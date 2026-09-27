//! Operator commands for external publishers and the two root workers.

use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};

use repobox_platform::publisher::spool::{self, Job, Op, Spool};
use repobox_platform::publisher::worker::{self, NetworkConfig, WorkerConfig};
use repobox_platform::publisher::{TOKEN_MAX_TTL, TOKEN_MIN_TTL};
use repobox_platform::render::RenderConfig;
use repobox_platform::store::Store;
use repobox_platform::web;

pub const DEFAULT_SPOOL: &str = "/var/spool/repobox-publisher";

type R = Result<(), Box<dyn std::error::Error>>;

#[derive(Subcommand)]
pub enum PublisherCmd {
    /// Create a publisher principal (an external agent). Without --owner a
    /// dedicated member user named after the handle becomes the canonical
    /// owner record of every app it creates.
    Create {
        /// Handle (a-z, 0-9, '.', '_', '-'), e.g. muse
        #[arg(long)]
        handle: String,
        #[arg(long)]
        display_name: Option<String>,
        /// Bind an existing user as the owner record instead
        #[arg(long)]
        owner: Option<String>,
    },
    /// List publishers
    List,
    /// Show a publisher, its tokens and apps
    Show { handle: String },
    /// Disable a publisher: every token stops working at once (apps keep running)
    Disable { handle: String },
    /// Re-enable a publisher
    Enable { handle: String },
    /// Publisher tokens (rbpub_…; not OAuth)
    Token {
        #[command(subcommand)]
        cmd: TokenCmd,
    },
    /// Releases (newest first), optionally of one publisher or app
    Releases {
        #[arg(long)]
        publisher: Option<String>,
        #[arg(long)]
        app: Option<String>,
        #[arg(long, default_value_t = 30)]
        limit: i64,
    },
    /// One release in full, with its deploy log
    Release {
        id: String,
        #[arg(long, default_value = DEFAULT_SPOOL)]
        spool: PathBuf,
    },
    /// Remove a publisher-created app: registry row now, container/images/
    /// route via the deploy worker
    RemoveApp {
        name: String,
        /// Also delete the app's persistent /data volume
        #[arg(long)]
        purge_data: bool,
        #[arg(long, default_value = DEFAULT_SPOOL)]
        spool: PathBuf,
        /// Required: this cannot be undone
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum TokenCmd {
    /// Issue a publisher token; the raw value is written once to a new 0600
    /// file and never printed
    Create {
        #[arg(long)]
        publisher: String,
        /// Token name (a-z, 0-9, '.', '_', '-')
        #[arg(long)]
        name: String,
        /// Lifetime in days (1-30)
        #[arg(long, default_value_t = 30)]
        ttl_days: i64,
        /// File to write the token to (created 0600, must not exist)
        #[arg(long)]
        out: PathBuf,
    },
    /// List publisher tokens (never shows values)
    List,
    /// Revoke a token by name (effective on the next request)
    Revoke { name: String },
}

/// Shared by `publisher-worker` and `publisher-query`.
#[derive(Args, Clone)]
pub struct WorkerArgs {
    #[arg(long, default_value = DEFAULT_SPOOL)]
    pub spool: PathBuf,
    /// Root-owned per-app state tree
    #[arg(long, default_value = "/srv/repobox-platform/published")]
    pub root: PathBuf,
    /// Installed published routes (imported by the Caddyfile)
    #[arg(long, default_value = "/etc/caddy/repobox-platform/published.caddy")]
    pub routes_file: PathBuf,
    /// Route apply command; the worker appends `apply-published <candidate>`
    #[arg(long, num_args = 1.., default_values = ["python3", "/srv/repobox-platform/caddy-apply.py"])]
    pub apply_cmd: Vec<String>,
    /// Owner every spool file must have
    #[arg(long, default_value = "repobox-platform")]
    pub service_user: String,
    #[arg(long, default_value = "repo.box")]
    pub domain: String,
    #[arg(long, default_value = "127.0.0.1:3230")]
    pub gate: String,
    #[arg(long, default_value = "https://auth.repo.box")]
    pub public_base: String,
    /// Dedicated Docker network for published apps ("" = default bridge)
    #[arg(long, default_value = "repobox-published")]
    pub network: String,
    #[arg(long, default_value = "rbpub0")]
    pub bridge: String,
    #[arg(long, default_value = "172.31.240.0/24")]
    pub subnet: String,
    /// Skip the iptables rules for the published network (local tests)
    #[arg(long)]
    pub no_firewall: bool,
    #[arg(long, default_value_t = 3072)]
    pub memory_budget_mb: u32,
    /// Loopback host ports for app containers, e.g. 4600-5000
    #[arg(long, default_value = "4600-5000")]
    pub ports: String,
    #[arg(long, default_value = "docker")]
    pub docker: String,
    #[arg(long, default_value_t = 120)]
    pub health_timeout_secs: u64,
}

fn uid_of(user: &str) -> Option<u32> {
    let c = std::ffi::CString::new(user).ok()?;
    // SAFETY: getpwnam with a valid C string; the record is read at once.
    let pw = unsafe { libc::getpwnam(c.as_ptr()) };
    if pw.is_null() {
        None
    } else {
        Some(unsafe { (*pw).pw_uid })
    }
}

impl WorkerArgs {
    pub fn config(&self) -> Result<WorkerConfig, Box<dyn std::error::Error>> {
        let mut cfg = WorkerConfig::new(
            Spool::new(&self.spool),
            self.root.clone(),
            self.routes_file.clone(),
            RenderConfig {
                domain: self.domain.clone(),
                gate: self.gate.clone(),
                apps_roots: vec![],
            },
        );
        cfg.apply_cmd = self.apply_cmd.clone();
        cfg.service_uid = if self.service_user.is_empty() {
            None
        } else {
            Some(
                uid_of(&self.service_user)
                    .ok_or_else(|| format!("no user '{}'", self.service_user))?,
            )
        };
        cfg.public_base = self.public_base.clone();
        cfg.network = (!self.network.is_empty()).then(|| NetworkConfig {
            name: self.network.clone(),
            bridge: self.bridge.clone(),
            subnet: self.subnet.clone(),
            firewall: !self.no_firewall,
        });
        cfg.memory_budget_mb = self.memory_budget_mb;
        let (a, b) = self
            .ports
            .split_once('-')
            .and_then(|(a, b)| Some((a.parse::<u16>().ok()?, b.parse::<u16>().ok()?)))
            .filter(|(a, b)| a < b && *a >= 1024)
            .ok_or("--ports must look like 4600-5000")?;
        cfg.ports = a..b;
        cfg.docker = self.docker.clone();
        cfg.health_timeout = std::time::Duration::from_secs(self.health_timeout_secs);
        Ok(cfg)
    }
}

fn need_publisher(
    store: &Store,
    handle: &str,
) -> Result<repobox_platform::store::Publisher, Box<dyn std::error::Error>> {
    store
        .publisher_by_handle(handle)?
        .ok_or_else(|| format!("no publisher '{handle}'").into())
}

fn shown(out: &Path) -> String {
    std::env::var("REPOBOX_PLATFORM_OUT_DISPLAY").unwrap_or_else(|_| out.display().to_string())
}

pub fn run(store: &Store, cmd: PublisherCmd) -> R {
    match cmd {
        PublisherCmd::Create {
            handle,
            display_name,
            owner,
        } => {
            let owner = match owner {
                Some(o) => Some(
                    store
                        .user_by_name(&o)?
                        .ok_or_else(|| format!("no user '{o}'"))?,
                ),
                None => None,
            };
            let (p, o) = store.create_publisher(
                &handle,
                display_name.as_deref().unwrap_or(&handle),
                owner.as_ref(),
            )?;
            store.audit(
                None,
                "publisher.create",
                &p.handle,
                &format!("id={} owner={}", p.public_id, o.name),
            );
            println!(
                "publisher '{}' created: id {} (immutable), owner record '{}'",
                p.handle, p.public_id, o.name
            );
            println!(
                "issue its credential: repobox-platform publisher token create --publisher {} --name {}-1 --out FILE",
                p.handle, p.handle
            );
        }
        PublisherCmd::List => {
            println!(
                "{:<18} {:<22} {:<16} {:<8} APPS",
                "HANDLE", "ID", "OWNER", "STATE"
            );
            for p in store.list_publishers()? {
                let owner = store.user_by_id(p.owner_id)?.name;
                let apps = store.publisher_apps(p.id)?.len();
                println!(
                    "{:<18} {:<22} {:<16} {:<8} {apps}",
                    p.handle,
                    p.public_id,
                    owner,
                    if p.enabled { "on" } else { "off" }
                );
            }
        }
        PublisherCmd::Show { handle } => {
            let p = need_publisher(store, &handle)?;
            let now = store.now();
            println!(
                "handle:   {}\nid:       {}\nname:     {}\nowner:    {}\nstate:    {}\ncreated:  {}",
                p.handle,
                p.public_id,
                p.display_name,
                store.user_by_id(p.owner_id)?.name,
                if p.enabled { "enabled" } else { "disabled" },
                web::html::fmt_ts(p.created_at)
            );
            println!("tokens:");
            for t in store
                .list_publisher_tokens()?
                .into_iter()
                .filter(|t| t.publisher_id == p.id)
            {
                println!(
                    "  {:<24} {:<8} expires {}  last used {}",
                    t.name,
                    t.status(now),
                    web::html::fmt_ts(t.expires_at),
                    t.last_used_at
                        .map(web::html::fmt_ts)
                        .unwrap_or_else(|| "never".into())
                );
            }
            println!("apps:");
            for a in store.publisher_apps(p.id)? {
                let live = store.live_release(&a.name)?;
                println!(
                    "  {:<24} {:<8} live release: {}",
                    a.name,
                    if a.enabled { "on" } else { "off" },
                    live.map(|r| format!("{} (v{})", r.id, r.version))
                        .unwrap_or_else(|| "none".into())
                );
            }
        }
        PublisherCmd::Disable { handle } => {
            let p = need_publisher(store, &handle)?;
            store.set_publisher_enabled(p.id, false)?;
            store.audit(None, "publisher.disable", &p.handle, "cli");
            println!(
                "publisher '{}' disabled: all of its tokens stop working now",
                p.handle
            );
        }
        PublisherCmd::Enable { handle } => {
            let p = need_publisher(store, &handle)?;
            store.set_publisher_enabled(p.id, true)?;
            store.audit(None, "publisher.enable", &p.handle, "cli");
            println!("publisher '{}' enabled", p.handle);
        }
        PublisherCmd::Token { cmd } => token(store, cmd)?,
        PublisherCmd::Releases {
            publisher,
            app,
            limit,
        } => {
            let pid = match publisher {
                Some(h) => Some(need_publisher(store, &h)?.id),
                None => None,
            };
            println!(
                "{:<30} {:<22} {:>3} {:<9} {:<11} {:<19} FAILURE",
                "RELEASE", "APP", "V", "OP", "STATUS", "CREATED"
            );
            for r in store.publisher_releases(pid, app.as_deref(), limit)? {
                println!(
                    "{:<30} {:<22} {:>3} {:<9} {:<11} {:<19} {}",
                    r.id,
                    r.app_name,
                    r.version,
                    r.op,
                    r.status,
                    web::html::fmt_ts(r.created_at),
                    if r.failure_code.is_empty() {
                        String::new()
                    } else {
                        format!("{}: {}", r.failure_code, r.failure)
                    }
                );
            }
        }
        PublisherCmd::Release { id, spool } => {
            let r = store
                .publisher_release(None, &id)?
                .ok_or_else(|| format!("no release '{id}'"))?;
            let p = store.publisher_by_id(r.publisher_id)?;
            println!(
                "release:   {}\napp:       {} (v{})\npublisher: {} ({})\nop:        {}{}\nstatus:    {}\nartifact:  {} ({} bytes, {})\nimage:     {}\nretained:  {}\ncreated:   {}\nfinished:  {}",
                r.id,
                r.app_name,
                r.version,
                p.handle,
                p.public_id,
                r.op,
                r.rollback_of
                    .as_ref()
                    .map(|t| format!(" of {t}"))
                    .unwrap_or_default(),
                r.status,
                r.artifact_sha256,
                r.artifact_bytes,
                r.build_mode,
                r.image_id,
                r.retained,
                web::html::fmt_ts(r.created_at),
                r.finished_at
                    .map(web::html::fmt_ts)
                    .unwrap_or_else(|| "-".into())
            );
            if !r.failure_code.is_empty() {
                println!("failure:   {}: {}", r.failure_code, r.failure);
            }
            println!("manifest:  {}", r.manifest);
            if let Some(log) = Spool::new(spool).build_log(&r.id) {
                println!("--- deploy log ---\n{log}");
            }
        }
        PublisherCmd::RemoveApp {
            name,
            purge_data,
            spool,
            yes,
        } => {
            let app = store
                .app_by_name(&name)?
                .ok_or_else(|| format!("no app '{name}'"))?;
            if app.publisher_id.is_none() {
                return Err(
                    format!("'{name}' was not created by a publisher; use `app remove`").into(),
                );
            }
            if !yes {
                return Err("removing an app cannot be undone; pass --yes".into());
            }
            let id = spool::new_id("rm", store.now());
            Spool::new(spool).enqueue(&Job {
                v: 1,
                op: Op::Remove,
                app: name.clone(),
                id: id.clone(),
                manifest: None,
                upload: None,
                target: String::new(),
                purge_data,
                created_at: store.now(),
            })?;
            store.delete_app(&name)?;
            store.audit(
                None,
                "publisher.remove_app",
                &name,
                &format!("job={id} purge_data={purge_data}"),
            );
            println!(
                "'{name}' removed from the registry (the gate answers 404 now); the deploy worker removes its container, images{} and route (job {id})",
                if purge_data { ", data volume" } else { "" }
            );
        }
    }
    Ok(())
}

fn token(store: &Store, cmd: TokenCmd) -> R {
    match cmd {
        TokenCmd::Create {
            publisher,
            name,
            ttl_days,
            out,
        } => {
            use std::os::unix::fs::OpenOptionsExt;
            if out.exists() {
                return Err(format!("{} already exists; choose a new file", out.display()).into());
            }
            let ttl = ttl_days * 86400;
            if !(TOKEN_MIN_TTL..=TOKEN_MAX_TTL).contains(&ttl) {
                return Err("--ttl-days must be 1-30".into());
            }
            let p = need_publisher(store, &publisher)?;
            // Open the file first so a failure never leaves a live token nobody holds.
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&out)?;
            let (raw, tok) = match store.create_publisher_token(&name, &p, ttl) {
                Ok(v) => v,
                Err(e) => {
                    drop(f);
                    let _ = std::fs::remove_file(&out);
                    return Err(e.into());
                }
            };
            writeln!(f, "{raw}")?;
            writeln!(
                f,
                "# repo.box publisher token '{}' for publisher '{}' ({}); expires {}; use as Authorization: Bearer <first line>; start at https://auth.repo.box/api/platform/v1",
                tok.name,
                p.handle,
                p.public_id,
                web::html::fmt_ts(tok.expires_at)
            )?;
            store.audit(
                None,
                "publisher_token.create",
                &tok.name,
                &format!("publisher={} ttl_days={ttl_days}", p.public_id),
            );
            println!(
                "publisher token '{}' written to {} (publisher {}, expires {})",
                tok.name,
                shown(&out),
                p.handle,
                web::html::fmt_ts(tok.expires_at)
            );
        }
        TokenCmd::List => {
            let now = store.now();
            println!(
                "{:<24} {:<16} {:<8} {:<19} LAST USED",
                "NAME", "PUBLISHER", "STATE", "EXPIRES"
            );
            for t in store.list_publisher_tokens()? {
                let p = store.publisher_by_id(t.publisher_id)?;
                println!(
                    "{:<24} {:<16} {:<8} {:<19} {}",
                    t.name,
                    p.handle,
                    t.status(now),
                    web::html::fmt_ts(t.expires_at),
                    t.last_used_at
                        .map(web::html::fmt_ts)
                        .unwrap_or_else(|| "never".into())
                );
            }
        }
        TokenCmd::Revoke { name } => {
            if store.revoke_publisher_token(&name)? {
                store.audit(None, "publisher_token.revoke", &name, "cli");
                println!("publisher token '{name}' revoked");
            } else {
                println!("publisher token '{name}' was already revoked");
            }
        }
    }
    Ok(())
}

pub fn run_worker(args: &WorkerArgs) -> R {
    let cfg = args.config()?;
    let n = worker::run_once(&cfg)?;
    eprintln!("publisher-worker: {n} job(s)");
    Ok(())
}

pub fn run_query(args: &WorkerArgs) -> R {
    let cfg = args.config()?;
    let n = worker::run_queries(&cfg)?;
    eprintln!("publisher-query: {n} quer(ies)");
    Ok(())
}
