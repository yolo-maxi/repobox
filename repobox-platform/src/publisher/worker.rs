//! The deploy worker (`repobox-platform publisher-worker`) and the runtime
//! query worker (`repobox-platform publisher-query`).
//!
//! Both run as root systemd oneshots started by `.path` units when a file
//! lands in the spool. They are the only components with Docker and Caddy
//! authority for published apps; the HTTP service has none of it. The
//! boundary is narrow by construction rather than by input paranoia:
//!
//! * they never open the registry database; a job is a validated manifest
//!   plus an app name, release id and uploaded archive, re-validated here;
//! * an uploaded `docker save` archive is imported under the platform's own
//!   name `repobox-pub/<app>:<release>` only (the names it carried are
//!   rewritten before `docker load`, so no upload can retag another image);
//! * every external command is an argv (no shell), with values that cannot
//!   become options, and a fixed `docker run` template: a dedicated bridge
//!   network without inter-container traffic, the port published on
//!   127.0.0.1 only, no privileged mode, host network, host mounts or
//!   devices, one named `/data` volume per app, memory/CPU/pids limits,
//!   `no-new-privileges`;
//! * the published routes are rendered from the worker's own per-app state
//!   (validated DNS labels and loopback ports only) with the standard gated
//!   route shape, and applied through `caddy-apply.py apply-published`
//!   (backup, validate the whole config, reload, restore on failure);
//! * a release goes live only after its container answers its health check;
//!   the previous container keeps serving until the route has switched
//!   (blue/green by port), and the last `keep` release images stay for
//!   rollback.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::archive;
use super::manifest::{self, Manifest};
use super::spool::{self, Job, JobResult, Op, Query, QueryResult, Spool};
use crate::model::{AiPolicy, App, AppKind, IdentityContract, Visibility, validate_app_name};
use crate::render::{self, RenderConfig};

/// Jobs older than this are refused (the API gives up at `JOB_TIMEOUT_SECS`).
pub const MAX_JOB_AGE_SECS: i64 = 1800;
pub const DEFAULT_KEEP: usize = 3;
pub const PUBLISHED_HEADER: &str =
    "# PUBLISHED by `repobox-platform publisher-worker`. Do not edit by hand.";
pub const IMAGE_REPO: &str = "repobox-pub";

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Docker network name, e.g. `repobox-published`.
    pub name: String,
    /// Linux bridge name (for the firewall rules), e.g. `rbpub0`.
    pub bridge: String,
    pub subnet: String,
    /// Install the iptables rules (NEW connections from the bridge to the
    /// host and to private/link-local ranges are dropped).
    pub firewall: bool,
}

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub spool: Spool,
    /// Root-owned per-app state tree, e.g. /srv/repobox-platform/published
    pub root: PathBuf,
    /// Installed published routes, e.g. /etc/caddy/repobox-platform/published.caddy
    pub routes_file: PathBuf,
    /// Program + leading args; the worker appends `apply-published <candidate>`.
    pub apply_cmd: Vec<String>,
    pub render: RenderConfig,
    /// Owner every job/query file must have (the API's service user).
    pub service_uid: Option<u32>,
    pub keep: usize,
    pub docker: String,
    pub network: Option<NetworkConfig>,
    /// Host ports handed to app containers (127.0.0.1 only).
    pub ports: std::ops::Range<u16>,
    /// Sum of `memory_mb` over all running published apps.
    pub memory_budget_mb: u32,

    pub health_timeout: Duration,
    /// e.g. https://auth.repo.box (for the env the platform sets)
    pub public_base: String,
    pub now: fn() -> i64,
}

fn now_system() -> i64 {
    chrono::Utc::now().timestamp()
}

impl WorkerConfig {
    pub fn new(spool: Spool, root: PathBuf, routes_file: PathBuf, render: RenderConfig) -> Self {
        Self {
            spool,
            root,
            routes_file,
            apply_cmd: vec![
                "python3".into(),
                "/srv/repobox-platform/caddy-apply.py".into(),
            ],
            render,
            service_uid: None,
            keep: DEFAULT_KEEP,
            docker: "docker".into(),
            network: None,
            ports: 4600..5000,
            memory_budget_mb: 3072,

            health_timeout: Duration::from_secs(120),
            public_base: "https://auth.repo.box".into(),
            now: now_system,
        }
    }
}

/// One deployed release of an app, as the worker knows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deployed {
    pub release: String,
    /// Local image reference `repobox-pub/<app>:<release>` (a rollback
    /// reuses the image of the release it restores).
    pub image: String,
    pub image_id: String,
    pub container_port: u16,
    pub memory_mb: u32,
    pub health_path: String,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub commit: String,
    pub build_mode: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppState {
    pub app: String,
    /// Live release, its container and host port.
    #[serde(default)]
    pub current: Option<Live>,
    /// Retained releases, newest first (current included).
    #[serde(default)]
    pub releases: Vec<Deployed>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Live {
    pub release: String,
    pub container: String,
    pub host_port: u16,
}

struct Fail {
    code: &'static str,
    message: String,
}

fn fail(code: &'static str, message: impl Into<String>) -> Fail {
    Fail {
        code,
        message: message.into(),
    }
}

// ------------------------------------------------------------------ helpers

fn lock(work: &Path, name: &str) -> std::io::Result<File> {
    let f = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(work.join(name))?;
    use std::os::fd::AsRawFd;
    // SAFETY: flock on a valid, owned descriptor.
    if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(f)
}

fn state_path(cfg: &WorkerConfig, app: &str) -> PathBuf {
    cfg.root.join(app).join("state.json")
}

pub fn load_state(cfg: &WorkerConfig, app: &str) -> AppState {
    std::fs::read(state_path(cfg, app))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_else(|| AppState {
            app: app.into(),
            ..Default::default()
        })
}

fn save_state(cfg: &WorkerConfig, st: &AppState) -> std::io::Result<()> {
    mkdir_0755(&cfg.root)?;
    let dir = cfg.root.join(&st.app);
    mkdir_0755(&dir)?;
    let tmp = dir.join("state.json.next");
    std::fs::write(
        &tmp,
        serde_json::to_vec_pretty(st).map_err(std::io::Error::other)?,
    )?;
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o644))?;
    std::fs::rename(&tmp, dir.join("state.json"))
}

fn mkdir_0755(p: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(p) {
        Ok(m) if m.is_dir() => Ok(()),
        Ok(_) => Err(std::io::Error::other(format!(
            "{} exists and is not a directory",
            p.display()
        ))),
        Err(_) => {
            std::fs::create_dir(p)?;
            std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755))
        }
    }
}

/// Apps with worker state, sorted.
pub fn published_apps(cfg: &WorkerConfig) -> Vec<AppState> {
    let mut names: Vec<String> = std::fs::read_dir(&cfg.root)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| validate_app_name(n).is_ok())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names.into_iter().map(|n| load_state(cfg, &n)).collect()
}

/// Build log of a job: world-readable, appended by every step.
struct Log {
    file: File,
}

impl Log {
    fn open(spool: &Spool, id: &str) -> std::io::Result<Self> {
        let path = spool.results().join(format!("{id}.log"));
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o644)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&path)?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))?;
        Ok(Self { file })
    }
    fn line(&mut self, s: &str) {
        let _ = writeln!(self.file, "{s}");
    }
    fn stdio(&self) -> Stdio {
        self.file
            .try_clone()
            .map(Stdio::from)
            .unwrap_or_else(|_| Stdio::null())
    }
}

/// Run to completion with a deadline, output appended to the log.
fn run_logged(cmd: &mut Command, log: &mut Log, timeout: Duration, what: &str) -> Result<(), Fail> {
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(log.stdio())
        .stderr(log.stdio())
        .spawn()
        .map_err(|e| fail("internal", format!("cannot run {what} ({e})")))?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(st)) if st.success() => return Ok(()),
            Ok(Some(st)) => {
                return Err(fail(
                    "step_failed",
                    format!("{what} failed ({st}); see the build log"),
                ));
            }
            Ok(None) if Instant::now() > deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(fail(
                    "timeout",
                    format!("{what} did not finish within {}s", timeout.as_secs()),
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(250)),
            Err(e) => return Err(fail("internal", format!("{what}: {e}"))),
        }
    }
}

/// Quick command, stdout captured.
fn capture(cmd: &mut Command) -> Option<String> {
    let out = cmd
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn docker(cfg: &WorkerConfig) -> Command {
    let mut c = Command::new(&cfg.docker);
    // Anonymous registry access only: never the host's own credentials.
    let conf = cfg.spool.work().join("docker-config");
    let _ = std::fs::create_dir_all(&conf);
    c.env("DOCKER_CONFIG", conf).env("DOCKER_BUILDKIT", "1");
    c
}

fn container_name(app: &str, release: &str) -> String {
    format!("rbpub-{app}-{}", &release[release.len() - 8..])
}

fn volume_name(app: &str) -> String {
    format!("rbpub-{app}-data")
}

// ------------------------------------------------------------- deploy worker

/// Process every queued job once. Returns the number of jobs handled.
pub fn run_once(cfg: &WorkerConfig) -> std::io::Result<usize> {
    let _lock = lock(&cfg.spool.work(), "deploy.lock")?;
    if let Some(net) = &cfg.network {
        ensure_network(cfg, net);
    }
    let mut names: Vec<String> = Vec::new();
    for e in std::fs::read_dir(cfg.spool.jobs())?.filter_map(|e| e.ok()) {
        let n = e.file_name().to_string_lossy().into_owned();
        if n.ends_with(".json") && !n.starts_with('.') {
            names.push(n);
        } else {
            // Nothing else belongs here; leaving it would re-fire the trigger.
            let _ = std::fs::remove_file(e.path());
        }
    }
    names.sort();
    let mut handled = 0;
    for name in names {
        let path = cfg.spool.jobs().join(&name);
        let job = spool::read_untrusted(&path, spool::MAX_JOB_BYTES, cfg.service_uid)
            .ok()
            .and_then(|b| serde_json::from_slice::<Job>(&b).ok());
        // The job file goes first: a job is attempted exactly once.
        let _ = std::fs::remove_file(&path);
        let Some(job) = job else {
            eprintln!("publisher-worker: dropped unreadable job {name:?}");
            continue;
        };
        let id_ok = match job.op {
            Op::Remove => spool::valid_id(&job.id, "rm"),
            _ => spool::valid_id(&job.id, "rel"),
        };
        if !id_ok || name != format!("{}.json", job.id) || validate_app_name(&job.app).is_err() {
            eprintln!("publisher-worker: dropped malformed job {name:?}");
            continue;
        }
        handled += 1;
        let mut log = Log::open(&cfg.spool, &job.id)?;
        let mut progress = JobResult {
            id: job.id.clone(),
            app: job.app.clone(),
            state: "building".into(),
            updated_at: (cfg.now)(),
            ..Default::default()
        };
        let _ = cfg.spool.write_json(&job.id, &progress);
        let outcome = process(cfg, &job, &mut log, &mut progress);
        let st = load_state(cfg, &job.app);
        progress.retained = st.releases.iter().map(|d| d.release.clone()).collect();
        progress.current = st.current.map(|c| c.release).unwrap_or_default();
        progress.updated_at = (cfg.now)();
        progress.finished_at = Some((cfg.now)());
        match outcome {
            Ok(()) => {
                progress.state = "live".into();
                log.line(&format!("== {:?} {} is live", job.op, job.id));
            }
            Err(f) => {
                progress.state = "failed".into();
                progress.code = f.code.into();
                progress.failure = f.message.clone();
                log.line(&format!("== FAILED ({}): {}", f.code, f.message));
            }
        }
        eprintln!(
            "publisher-worker: {:?} {} app={} {} {}",
            job.op, job.id, job.app, progress.state, progress.code
        );
        drop(log);
        trim_log(&cfg.spool.results().join(format!("{}.log", job.id)));
        cfg.spool.write_json(&job.id, &progress)?;
    }
    prune_spool(cfg);
    Ok(handled)
}

fn process(
    cfg: &WorkerConfig,
    job: &Job,
    log: &mut Log,
    progress: &mut JobResult,
) -> Result<(), Fail> {
    if (cfg.now)() - job.created_at > MAX_JOB_AGE_SECS {
        return Err(fail(
            "expired",
            "the release waited too long in the queue; deploy again",
        ));
    }
    match job.op {
        Op::Deploy => {
            let m = job
                .manifest
                .as_ref()
                .ok_or_else(|| fail("internal", "deploy job without a manifest"))?;
            // Re-validate: the job came through a less trusted hop.
            let m = manifest::from_value(&serde_json::to_value(m).unwrap_or_default())
                .map_err(|e| fail(e.code, e.message))?;
            if m.name != job.app {
                return Err(fail("internal", "manifest/app mismatch"));
            }
            deploy(cfg, job, &m, log, progress)
        }
        Op::Rollback => rollback(cfg, job, log, progress),
        Op::Restart => restart(cfg, job, log),
        Op::Remove => remove(cfg, job, log),
    }
}

fn deploy(
    cfg: &WorkerConfig,
    job: &Job,
    m: &Manifest,
    log: &mut Log,
    progress: &mut JobResult,
) -> Result<(), Fail> {
    let up = job
        .upload
        .as_ref()
        .ok_or_else(|| fail("internal", "deploy job without an uploaded image"))?;
    let image = format!("{IMAGE_REPO}/{}:{}", job.app, job.id);
    let r = load_upload(cfg, job, up, &image, log);
    let _ = std::fs::remove_file(cfg.spool.upload_path(&job.id));
    let format = r?;
    let inspect = capture(docker(cfg).args([
        "image",
        "inspect",
        "-f",
        "{{.Id}}|{{.Os}}|{{.Architecture}}|{{json .Config.ExposedPorts}}",
        &image,
    ]))
    .ok_or_else(|| fail("internal", "the loaded image is not visible"))?;
    let mut it = inspect.splitn(4, '|');
    let image_id = it.next().unwrap_or("").to_string();
    let os = it.next().unwrap_or("");
    let arch = it.next().unwrap_or("");
    let exposed = it.next().unwrap_or("null");
    if os != "linux" || arch != "amd64" {
        let _ = capture(docker(cfg).args(["image", "rm", &image]));
        return Err(fail(
            "wrong_platform",
            format!(
                "the image is {os}/{arch}; repo.box runs linux/amd64 (build with `docker build --platform linux/amd64`)"
            ),
        ));
    }
    let container_port = m
        .runtime
        .port
        .unwrap_or_else(|| single_exposed_port(exposed));
    log.line(&format!(
        "== image {image_id} ({os}/{arch}), container port {container_port}"
    ));
    progress.image_id = image_id.clone();
    progress.build_mode = format.clone();
    progress.state = "starting".into();
    progress.updated_at = (cfg.now)();
    let _ = cfg.spool.write_json(&job.id, progress);
    let d = Deployed {
        release: job.id.clone(),
        image,
        image_id,
        container_port,
        memory_mb: m.runtime.memory_mb,
        health_path: m.runtime.health_path.clone(),
        env: m.runtime.env.clone(),
        commit: m.provenance.commit.clone(),
        build_mode: format,
        created_at: (cfg.now)(),
    };
    let r = go_live(cfg, &job.app, d.clone(), log);
    if r.is_err() {
        let _ = capture(docker(cfg).args(["image", "rm", &d.image]));
    }
    r
}

/// `{"8080/tcp":{}}` -> 8080; anything but exactly one TCP port -> 8080.
fn single_exposed_port(json: &str) -> u16 {
    let v: serde_json::Value = serde_json::from_str(json).unwrap_or_default();
    let ports: Vec<u16> = v
        .as_object()
        .map(|m| {
            m.keys()
                .filter_map(|k| k.strip_suffix("/tcp").and_then(|p| p.parse().ok()))
                .collect()
        })
        .unwrap_or_default();
    match ports.as_slice() {
        [p] => *p,
        _ => manifest::DEFAULT_PORT,
    }
}

/// Stream the uploaded archive (verifying its hash) into a copy whose image
/// names are replaced by `image`, then `docker load` it. Whatever names the
/// archive carried are never applied on the host, so an upload can never
/// retag another app's or the host's images.
fn load_upload(
    cfg: &WorkerConfig,
    job: &Job,
    up: &spool::Upload,
    image: &str,
    log: &mut Log,
) -> Result<String, Fail> {
    let src = spool::open_untrusted(
        &cfg.spool.upload_path(&job.id),
        archive::MAX_UPLOAD_BYTES,
        cfg.service_uid,
    )
    .map_err(|e| {
        fail(
            "internal",
            format!("upload unavailable to the worker ({e})"),
        )
    })?;
    let dir = cfg.spool.work().join("load");
    std::fs::create_dir_all(&dir).map_err(io_fail)?;
    let out_path = dir.join(format!("{}.tar", job.id));
    let _ = std::fs::remove_file(&out_path);
    let r = (|| {
        let out = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&out_path)
            .map_err(io_fail)?;
        log.line(&format!(
            "== import {} bytes ({})",
            up.bytes,
            if up.gzip {
                "docker save | gzip"
            } else {
                "docker save"
            }
        ));
        let summary = archive::rename(src, std::io::BufWriter::new(out), image)
            .map_err(|e| fail(e.code, e.message))?;
        if summary.sha256 != up.sha256 {
            return Err(fail(
                "internal",
                "the queued upload does not match what the API received",
            ));
        }
        log.line(&format!("== archive format: {}", summary.format));
        let format = summary.format.clone();
        let out = std::process::Command::new(&cfg.docker)
            .env("DOCKER_CONFIG", cfg.spool.work().join("docker-config"))
            .args(["load", "-i"])
            .arg(&out_path)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| fail("internal", format!("cannot run docker load ({e})")))?;
        let text = String::from_utf8_lossy(&out.stdout).to_string()
            + &String::from_utf8_lossy(&out.stderr);
        log.line(text.trim_end());
        if !out.status.success() {
            return Err(fail(
                "load_failed",
                "docker could not load the archive; see the build log",
            ));
        }
        let loaded: Vec<&str> = text
            .lines()
            .filter_map(|l| l.strip_prefix("Loaded image: "))
            .map(str::trim)
            .collect();
        if loaded != [image] {
            return Err(fail(
                "load_failed",
                format!("expected exactly one image named {image}, docker reported {loaded:?}"),
            ));
        }
        Ok(format)
    })();
    let _ = std::fs::remove_file(&out_path);
    r
}

fn io_fail(e: std::io::Error) -> Fail {
    fail("internal", format!("filesystem error ({e})"))
}

/// Start `d` next to the current release, health-gate it, switch the route,
/// then retire the old container.
fn go_live(cfg: &WorkerConfig, app: &str, d: Deployed, log: &mut Log) -> Result<(), Fail> {
    let mut st = load_state(cfg, app);
    let others: u32 = published_apps(cfg)
        .iter()
        .filter(|s| s.app != app)
        .filter_map(|s| {
            let live = s.current.as_ref()?;
            s.releases
                .iter()
                .find(|r| r.release == live.release)
                .map(|r| r.memory_mb)
        })
        .sum();
    if others + d.memory_mb > cfg.memory_budget_mb {
        return Err(fail(
            "capacity",
            format!(
                "not enough memory budget for {} MB (published apps use {others} of {} MB); lower runtime.memory_mb or ask an operator",
                d.memory_mb, cfg.memory_budget_mb
            ),
        ));
    }
    let port = pick_port(cfg)?;
    let name = container_name(app, &d.release);
    let _ = capture(docker(cfg).args(["rm", "-f", &name]));
    let _ = capture(docker(cfg).args([
        "volume",
        "create",
        "--label",
        &format!("repobox.app={app}"),
        &volume_name(app),
    ]));
    let mut run = docker(cfg);
    run.args(["run", "-d", "--name", &name])
        .args(["--label", "repobox.managed=publisher"])
        .arg("--label")
        .arg(format!("repobox.app={app}"))
        .arg("--label")
        .arg(format!("repobox.release={}", d.release))
        .arg("--network")
        .arg(
            cfg.network
                .as_ref()
                .map(|n| n.name.as_str())
                .unwrap_or("bridge"),
        )
        .arg("-p")
        .arg(format!("127.0.0.1:{port}:{}", d.container_port))
        .arg("--memory")
        .arg(format!("{}m", d.memory_mb))
        .arg("--memory-swap")
        .arg(format!("{}m", d.memory_mb))
        .args(["--cpus", "1", "--pids-limit", "512"])
        .args([
            "--security-opt",
            "no-new-privileges",
            "--cap-drop",
            "NET_RAW",
            "--cap-drop",
            "MKNOD",
        ])
        .args(["--restart", "unless-stopped"])
        .args(["--log-opt", "max-size=10m", "--log-opt", "max-file=3"])
        .arg("-v")
        .arg(format!("{}:/data", volume_name(app)));
    let domain = &cfg.render.domain;
    let platform_env = [
        ("PORT", d.container_port.to_string()),
        ("REPOBOX_APP", app.to_string()),
        ("REPOBOX_APP_URL", format!("https://{app}.{domain}/")),
        ("REPOBOX_LAUNCHER_URL", format!("{}/{app}", cfg.public_base)),
        (
            "REPOBOX_AI_CHAT_PATH",
            crate::model::AI_CHAT_PATH.to_string(),
        ),
        ("REPOBOX_RELEASE", d.release.clone()),
        ("REPOBOX_DATA_DIR", "/data".to_string()),
    ];
    for (k, v) in platform_env
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .chain(d.env.clone())
    {
        run.arg("-e").arg(format!("{k}={v}"));
    }
    run.arg("--").arg(&d.image);
    log.line(&format!(
        "== start {name} on 127.0.0.1:{port} (container port {}, {} MB)",
        d.container_port, d.memory_mb
    ));
    run_logged(&mut run, log, Duration::from_secs(120), "docker run")?;
    if let Err(f) = wait_healthy(cfg, &name, port, &d.health_path, log) {
        container_logs_to(cfg, &name, log);
        let _ = capture(docker(cfg).args(["rm", "-f", &name]));
        return Err(f);
    }
    let previous = st.current.clone();
    st.current = Some(Live {
        release: d.release.clone(),
        container: name.clone(),
        host_port: port,
    });
    st.releases.retain(|r| r.release != d.release);
    st.releases.insert(0, d);
    save_state(cfg, &st).map_err(io_fail)?;
    if let Err(f) = sync_routes(cfg, log) {
        let _ = capture(docker(cfg).args(["rm", "-f", &name]));
        st.current = previous;
        st.releases.remove(0);
        let _ = save_state(cfg, &st);
        let _ = sync_routes(cfg, log);
        return Err(f);
    }
    if let Some(prev) = previous
        && prev.container != name
    {
        log.line(&format!("== retire {}", prev.container));
        let _ = capture(docker(cfg).args(["rm", "-f", &prev.container]));
    }
    prune_releases(cfg, &mut st);
    let _ = save_state(cfg, &st);
    Ok(())
}

fn pick_port(cfg: &WorkerConfig) -> Result<u16, Fail> {
    let used: Vec<u16> = published_apps(cfg)
        .iter()
        .filter_map(|s| s.current.as_ref().map(|c| c.host_port))
        .collect();
    for p in cfg.ports.clone() {
        if used.contains(&p) {
            continue;
        }
        if std::net::TcpListener::bind(("127.0.0.1", p)).is_ok() {
            return Ok(p);
        }
    }
    Err(fail("capacity", "no free loopback port for the app"))
}

fn wait_healthy(
    cfg: &WorkerConfig,
    container: &str,
    port: u16,
    path: &str,
    log: &mut Log,
) -> Result<(), Fail> {
    let deadline = Instant::now() + cfg.health_timeout;
    // An explicit health path must answer 2xx/3xx; the default "/" only has
    // to answer HTTP at all (API-only servers may 404 there).
    let strict = path != "/";
    let mut last = String::from("no answer yet");
    while Instant::now() < deadline {
        let status = capture(docker(cfg).args(["inspect", "-f", "{{.State.Status}}", container]))
            .unwrap_or_default();
        if status == "exited" || status == "dead" {
            return Err(fail(
                "unhealthy",
                "the app exited during startup; see the deploy log for its output".to_string(),
            ));
        }
        match http_status(port, path) {
            Some(code) if (strict && code < 400) || (!strict && code < 500) => {
                log.line(&format!("== healthy: GET {path} -> {code}"));
                return Ok(());
            }
            Some(code) => last = format!("GET {path} -> {code}"),
            None => {}
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(fail(
        "unhealthy",
        format!(
            "the app did not become healthy within {}s ({last}); it must listen on 0.0.0.0:$PORT (runtime.port) and answer runtime.health_path",
            cfg.health_timeout.as_secs()
        ),
    ))
}

/// Minimal HTTP/1.1 probe on loopback: the status code, if any.
pub fn http_status(port: u16, path: &str) -> Option<u16> {
    use std::net::TcpStream;
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).ok()?;
    s.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    write!(s, "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nUser-Agent: repobox-health\r\nConnection: close\r\n\r\n").ok()?;
    let mut buf = [0u8; 64];
    let n = s.read(&mut buf).ok()?;
    let line = std::str::from_utf8(&buf[..n]).ok()?;
    line.strip_prefix("HTTP/1.")?.get(2..5)?.parse().ok()
}

fn container_logs_to(cfg: &WorkerConfig, container: &str, log: &mut Log) {
    log.line(&format!("== last output of {container}"));
    let _ = docker(cfg)
        .args(["logs", "--tail", "80", container])
        .stdin(Stdio::null())
        .stdout(log.stdio())
        .stderr(log.stdio())
        .status();
}

fn prune_releases(cfg: &WorkerConfig, st: &mut AppState) {
    let keep = cfg.keep.max(1);
    if st.releases.len() <= keep {
        return;
    }
    let dropped: Vec<Deployed> = st.releases.split_off(keep);
    for d in dropped {
        if !st.releases.iter().any(|r| r.image == d.image) {
            let _ = capture(docker(cfg).args(["image", "rm", &d.image]));
        }
    }
}

fn rollback(
    cfg: &WorkerConfig,
    job: &Job,
    log: &mut Log,
    progress: &mut JobResult,
) -> Result<(), Fail> {
    let st = load_state(cfg, &job.app);
    let target = st
        .releases
        .iter()
        .find(|r| r.release == job.target)
        .cloned()
        .ok_or_else(|| {
            fail(
                "release_not_retained",
                "that release is no longer retained; deploy it again",
            )
        })?;
    log.line(&format!(
        "== roll back to {} ({})",
        target.release, target.image
    ));
    progress.commit = target.commit.clone();
    progress.build_mode = target.build_mode.clone();
    progress.image_id = target.image_id.clone();
    let d = Deployed {
        release: job.id.clone(),
        created_at: (cfg.now)(),
        ..target
    };
    go_live(cfg, &job.app, d, log)
}

fn restart(cfg: &WorkerConfig, job: &Job, log: &mut Log) -> Result<(), Fail> {
    let st = load_state(cfg, &job.app);
    let live = st
        .current
        .clone()
        .ok_or_else(|| fail("not_running", "the app has no live release"))?;
    let d = st
        .releases
        .iter()
        .find(|r| r.release == live.release)
        .cloned()
        .ok_or_else(|| fail("internal", "live release missing from state"))?;
    log.line(&format!("== restart {}", live.container));
    run_logged(
        docker(cfg).args(["restart", &live.container]),
        log,
        Duration::from_secs(120),
        "docker restart",
    )?;
    wait_healthy(cfg, &live.container, live.host_port, &d.health_path, log)
}

fn remove(cfg: &WorkerConfig, job: &Job, log: &mut Log) -> Result<(), Fail> {
    let app = &job.app;
    if let Some(ids) = capture(docker(cfg).args([
        "ps",
        "-aq",
        "--filter",
        &format!("label=repobox.app={app}"),
        "--filter",
        "label=repobox.managed=publisher",
    ])) {
        for id in ids.split_whitespace() {
            let _ = capture(docker(cfg).args(["rm", "-f", id]));
        }
    }
    let st = load_state(cfg, app);
    for d in &st.releases {
        let _ = capture(docker(cfg).args(["image", "rm", &d.image]));
    }
    if let Some(tags) =
        capture(docker(cfg).args(["image", "ls", "-q", &format!("{IMAGE_REPO}/{app}")]))
    {
        for t in tags.split_whitespace() {
            let _ = capture(docker(cfg).args(["image", "rm", "-f", t]));
        }
    }
    if job.purge_data {
        let _ = capture(docker(cfg).args(["volume", "rm", &volume_name(app)]));
        log.line("== data volume removed");
    }
    let _ = std::fs::remove_dir_all(cfg.root.join(app));
    log.line(&format!("== removed {app}"));
    sync_routes(cfg, log)
}

fn trim_log(path: &Path) {
    let Ok(mut f) = std::fs::OpenOptions::new().read(true).open(path) else {
        return;
    };
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    if len <= spool::MAX_LOG_BYTES {
        return;
    }
    let mut tail = Vec::new();
    if f.seek(SeekFrom::Start(len - spool::MAX_LOG_BYTES)).is_ok()
        && f.read_to_end(&mut tail).is_ok()
    {
        let mut out = b"[... earlier output trimmed ...]\n".to_vec();
        out.extend_from_slice(&tail);
        let _ = std::fs::write(path, out);
    }
}

fn prune_spool(cfg: &WorkerConfig) {
    let now = std::time::SystemTime::now();
    if let Ok(rd) = std::fs::read_dir(cfg.spool.results()) {
        for e in rd.filter_map(|e| e.ok()) {
            let old = e
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| now.duration_since(t).ok())
                .is_some_and(|d| d.as_secs() > 30 * 86400);
            if old {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
}

fn ensure_network(cfg: &WorkerConfig, net: &NetworkConfig) {
    if capture(docker(cfg).args(["network", "inspect", "-f", "{{.Name}}", &net.name])).is_none() {
        let ok = capture(docker(cfg).args([
            "network",
            "create",
            "--driver",
            "bridge",
            "--subnet",
            &net.subnet,
            "-o",
            "com.docker.network.bridge.enable_icc=false",
            "-o",
            &format!("com.docker.network.bridge.name={}", net.bridge),
            "--label",
            "repobox.managed=publisher",
            &net.name,
        ]));
        eprintln!(
            "publisher-worker: created network {} ({})",
            net.name,
            if ok.is_some() { "ok" } else { "FAILED" }
        );
    }
    if !net.firewall {
        return;
    }
    let mut rules: Vec<Vec<String>> = [
        "10.0.0.0/8",
        "172.16.0.0/12",
        "192.168.0.0/16",
        "169.254.0.0/16",
        "100.64.0.0/10",
    ]
    .iter()
    .map(|d| {
        [
            "DOCKER-USER",
            "-i",
            &net.bridge,
            "-d",
            d,
            "-m",
            "conntrack",
            "--ctstate",
            "NEW",
            "-j",
            "DROP",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    })
    .collect();
    rules.push(
        [
            "INPUT",
            "-i",
            &net.bridge,
            "-m",
            "conntrack",
            "--ctstate",
            "NEW",
            "-j",
            "DROP",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
    );
    for r in rules {
        let present = Command::new("iptables")
            .arg("-C")
            .args(&r)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        if !present {
            let ok = Command::new("iptables")
                .arg("-I")
                .args(&r)
                .status()
                .is_ok_and(|s| s.success());
            eprintln!(
                "publisher-worker: firewall {} {}",
                r.join(" "),
                if ok { "installed" } else { "FAILED" }
            );
        }
    }
}

/// The published routes: the standard gated route of every managed app
/// (strip identity headers, forward_auth, the reserved `/_repo_box/*`
/// paths incl. the AI endpoint, then `reverse_proxy` to the loopback port).
pub fn render_published(states: &[AppState], cfg: &RenderConfig) -> Result<String, String> {
    let apps: Vec<App> = states
        .iter()
        .filter_map(|s| {
            let live = s.current.as_ref()?;
            Some(App {
                id: 0,
                name: s.app.clone(),
                title: s.app.clone(),
                description: String::new(),
                owner_id: 0,
                kind: AppKind::Proxy,
                target: format!("127.0.0.1:{}", live.host_port),
                visibility: Visibility::Private,
                enabled: true,
                created_at: 0,
                updated_at: 0,
                identity: IdentityContract::Platform,
                ai: AiPolicy::private_default(),
                publisher_id: Some(0),
            })
        })
        .collect();
    let body = render::render(&apps, cfg)?;
    let body = body.split_once('\n').map(|x| x.1).unwrap_or("");
    Ok(format!("{PUBLISHED_HEADER}\n{body}"))
}

/// Make the installed published routes match the worker state.
fn sync_routes(cfg: &WorkerConfig, log: &mut Log) -> Result<(), Fail> {
    let text = render_published(&published_apps(cfg), &cfg.render)
        .map_err(|e| fail("internal", format!("route render failed ({e})")))?;
    let current = std::fs::read_to_string(&cfg.routes_file).unwrap_or_default();
    if current == text {
        return Ok(());
    }
    let candidate = cfg.spool.work().join("published.caddy.candidate");
    let _ = std::fs::remove_file(&candidate);
    std::fs::write(&candidate, &text).map_err(io_fail)?;
    let (prog, args) = cfg
        .apply_cmd
        .split_first()
        .ok_or_else(|| fail("internal", "no apply command"))?;
    log.line("== apply route (validate + reload)");
    let r = run_logged(
        Command::new(prog)
            .args(args)
            .arg("apply-published")
            .arg(&candidate),
        log,
        Duration::from_secs(120),
        "route apply",
    );
    let _ = std::fs::remove_file(&candidate);
    r.map_err(|f| {
        fail(
            "route_apply_failed",
            format!(
                "the edge refused the new route; nothing changed ({})",
                f.message
            ),
        )
    })
}

// -------------------------------------------------------------- query worker

/// Answer every queued runtime query once (logs + container state of the
/// app's *current* container; the query names only the app).
pub fn run_queries(cfg: &WorkerConfig) -> std::io::Result<usize> {
    let _lock = lock(&cfg.spool.work(), "query.lock")?;
    let mut n = 0;
    let mut names: Vec<String> = Vec::new();
    for e in std::fs::read_dir(cfg.spool.queries())?.filter_map(|e| e.ok()) {
        let n = e.file_name().to_string_lossy().into_owned();
        if n.ends_with(".json") && !n.starts_with('.') {
            names.push(n);
        } else {
            // Nothing else belongs here; leaving it would re-fire the trigger.
            let _ = std::fs::remove_file(e.path());
        }
    }
    names.sort();
    for name in names {
        let path = cfg.spool.queries().join(&name);
        let q = spool::read_untrusted(&path, spool::MAX_JOB_BYTES, cfg.service_uid)
            .ok()
            .and_then(|b| serde_json::from_slice::<Query>(&b).ok());
        let _ = std::fs::remove_file(&path);
        let Some(q) = q else { continue };
        if !spool::valid_id(&q.id, "q")
            || name != format!("{}.json", q.id)
            || validate_app_name(&q.app).is_err()
        {
            continue;
        }
        n += 1;
        let mut r = QueryResult {
            id: q.id.clone(),
            app: q.app.clone(),
            finished_at: (cfg.now)(),
            ..Default::default()
        };
        let st = load_state(cfg, &q.app);
        match st.current {
            None => {
                r.ok = true;
                r.state = "absent".into();
            }
            Some(live) => {
                r.release = live.release.clone();
                let info = capture(docker(cfg).args([
                    "inspect",
                    "-f",
                    "{{.State.Status}}|{{.State.StartedAt}}|{{.RestartCount}}",
                    &live.container,
                ]))
                .unwrap_or_default();
                let mut it = info.split('|');
                r.state = it
                    .next()
                    .filter(|s| !s.is_empty())
                    .unwrap_or("absent")
                    .to_string();
                r.started_at = it.next().unwrap_or("").to_string();
                r.restarts = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                let tmp = cfg.spool.work().join(format!("{}.logs", q.id));
                let _ = std::fs::remove_file(&tmp);
                if let Ok(f) = std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .mode(0o600)
                    .open(&tmp)
                {
                    let tail = q.tail.clamp(1, 2000).to_string();
                    let _ = docker(cfg)
                        .args(["logs", "--timestamps", "--tail", &tail, &live.container])
                        .stdin(Stdio::null())
                        .stdout(f.try_clone().map(Stdio::from).unwrap_or(Stdio::null()))
                        .stderr(Stdio::from(f))
                        .status();
                    let mut bytes = std::fs::read(&tmp).unwrap_or_default();
                    let max = 200 * 1024;
                    if bytes.len() > max {
                        bytes = bytes.split_off(bytes.len() - max);
                    }
                    r.logs = String::from_utf8_lossy(&bytes).into_owned();
                    let _ = std::fs::remove_file(&tmp);
                }
                r.ok = true;
            }
        }
        cfg.spool.write_json(&q.id, &r)?;
    }
    Ok(n)
}
