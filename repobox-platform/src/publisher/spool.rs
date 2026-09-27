//! The release queue between the unprivileged API and the privileged
//! deploy worker.
//!
//! Layout (created by `scripts/deploy.sh`):
//!
//! ```text
//! /var/spool/repobox-publisher/            root:root 0755  (the service cannot swap the subdirectories)
//!   jobs/     repobox-platform 0700  <id>.json per queued deploy/rollback/restart/remove (deploy worker .path trigger)
//!   uploads/  repobox-platform 0700  <id>.image: uploaded image archives (docker/podman save or OCI layout tar, optionally gzip)
//!   queries/  repobox-platform 0700  <id>.json per runtime log/status query (query worker .path trigger)
//!   results/  root:root 0755         <id>.json progress + verdict, <id>.log build log (written by the workers)
//!   work/     root:root 0700         locks, checkouts, Caddy candidates
//! ```
//!
//! The API writes a job under a temporary name and renames it into place,
//! so a worker never sees half a job. Workers open the final path component
//! with `O_NOFOLLOW` and require a regular, singly-linked file owned by the
//! service user; everything in a job is re-validated before use.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::manifest::Manifest;

pub const MAX_JOB_BYTES: u64 = 64 * 1024;
pub const MAX_RESULT_BYTES: u64 = 256 * 1024;
/// Build logs kept per release (the tail is kept when a build is chattier).
pub const MAX_LOG_BYTES: u64 = 512 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Op {
    Deploy,
    Rollback,
    Restart,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub v: u32,
    pub op: Op,
    pub app: String,
    /// Release id (`rel-…`); `remove` uses an operation id (`rm-…`).
    pub id: String,
    /// Deploy: the validated manifest.
    #[serde(default)]
    pub manifest: Option<Manifest>,
    /// Deploy: an uploaded image archive `uploads/<id>.image`.
    #[serde(default)]
    pub upload: Option<Upload>,
    /// Rollback: the retained release to run again.
    #[serde(default)]
    pub target: String,
    /// Remove: also delete the app's persistent /data volume.
    #[serde(default)]
    pub purge_data: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Upload {
    /// `sha256:<hex>` of the uploaded bytes (checked again by the worker).
    pub sha256: String,
    pub bytes: u64,
    pub gzip: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobResult {
    pub id: String,
    pub app: String,
    /// building | starting | live | failed (progress is rewritten in place).
    pub state: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub failure: String,
    /// Resolved source commit.
    #[serde(default)]
    pub commit: String,
    /// dockerfile | node | node-static | static | image
    #[serde(default)]
    pub build_mode: String,
    /// Local image id (sha256:…) of the release.
    #[serde(default)]
    pub image_id: String,
    /// Release ids whose images are retained for rollback (current first).
    #[serde(default)]
    pub retained: Vec<String>,
    #[serde(default)]
    pub current: String,
    #[serde(default)]
    pub routes_applied: bool,
    pub updated_at: i64,
    #[serde(default)]
    pub finished_at: Option<i64>,
}

impl JobResult {
    pub fn is_final(&self) -> bool {
        matches!(self.state.as_str(), "live" | "failed")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub v: u32,
    pub id: String,
    pub app: String,
    pub tail: u32,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryResult {
    pub id: String,
    pub app: String,
    pub ok: bool,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub release: String,
    /// running | restarting | exited | … | absent
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub started_at: String,
    #[serde(default)]
    pub restarts: i64,
    #[serde(default)]
    pub logs: String,
    pub finished_at: i64,
}

/// `rel-YYYYMMDDHHMMSS-xxxxxxxx` (also `rm-`, `q-`): the only names a
/// worker will build a path from.
pub fn valid_id(id: &str, prefix: &str) -> bool {
    let Some(rest) = id.strip_prefix(prefix).and_then(|r| r.strip_prefix('-')) else {
        return false;
    };
    let b = rest.as_bytes();
    b.len() == 23
        && b[..14].iter().all(u8::is_ascii_digit)
        && b[14] == b'-'
        && b[15..]
            .iter()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(c))
}

pub fn new_id(prefix: &str, now: i64) -> String {
    use rand::RngCore;
    let ts = chrono::DateTime::from_timestamp(now, 0)
        .unwrap_or_default()
        .format("%Y%m%d%H%M%S");
    let mut b = [0u8; 4];
    rand::rngs::OsRng.fill_bytes(&mut b);
    format!("{prefix}-{ts}-{:08x}", u32::from_be_bytes(b))
}

#[derive(Debug, Clone)]
pub struct Spool {
    pub root: PathBuf,
}

impl Spool {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    pub fn jobs(&self) -> PathBuf {
        self.root.join("jobs")
    }
    pub fn uploads(&self) -> PathBuf {
        self.root.join("uploads")
    }
    pub fn upload_path(&self, id: &str) -> PathBuf {
        self.uploads().join(format!("{id}.image"))
    }
    pub fn queries(&self) -> PathBuf {
        self.root.join("queries")
    }
    pub fn results(&self) -> PathBuf {
        self.root.join("results")
    }
    pub fn work(&self) -> PathBuf {
        self.root.join("work")
    }

    /// Create the layout (tests and local runs; production uses deploy.sh).
    pub fn create_dirs(&self) -> std::io::Result<()> {
        for d in [
            self.jobs(),
            self.uploads(),
            self.queries(),
            self.results(),
            self.work(),
        ] {
            std::fs::create_dir_all(d)?;
        }
        Ok(())
    }

    /// Write under a temporary name in `uploads/` (same filesystem), then
    /// rename into `dir`: the trigger directories only ever hold complete
    /// entries, so a leftover part can never keep a `.path` unit firing.
    fn put(&self, dir: PathBuf, id: &str, bytes: &[u8]) -> std::io::Result<()> {
        let part = self.uploads().join(format!(".{id}.json.part"));
        write_new(&part, bytes)?;
        std::fs::rename(&part, dir.join(format!("{id}.json")))
    }

    /// API side: queue a job.
    pub fn enqueue(&self, job: &Job) -> std::io::Result<()> {
        self.put(
            self.jobs(),
            &job.id,
            &serde_json::to_vec(job).map_err(std::io::Error::other)?,
        )
    }

    /// API side: queue a runtime query.
    pub fn enqueue_query(&self, q: &Query) -> std::io::Result<()> {
        self.put(
            self.queries(),
            &q.id,
            &serde_json::to_vec(q).map_err(std::io::Error::other)?,
        )
    }

    fn read_json<T: for<'de> Deserialize<'de>>(&self, id: &str) -> Option<T> {
        if !["rel", "rm", "q"].iter().any(|p| valid_id(id, p)) {
            return None;
        }
        let bytes = read_untrusted(
            &self.results().join(format!("{id}.json")),
            MAX_RESULT_BYTES,
            None,
        )
        .ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// API side: the worker's progress/verdict for a job.
    pub fn result(&self, id: &str) -> Option<JobResult> {
        self.read_json::<JobResult>(id).filter(|r| r.id == id)
    }

    pub fn query_result(&self, id: &str) -> Option<QueryResult> {
        self.read_json::<QueryResult>(id).filter(|r| r.id == id)
    }

    /// API side: the build log of a release (may be partial while building).
    pub fn build_log(&self, id: &str) -> Option<String> {
        if !valid_id(id, "rel") {
            return None;
        }
        let bytes = read_untrusted(
            &self.results().join(format!("{id}.log")),
            MAX_LOG_BYTES * 2,
            None,
        )
        .ok()?;
        Some(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// Worker side: write a world-readable result file atomically.
    pub fn write_json<T: Serialize>(&self, id: &str, v: &T) -> std::io::Result<()> {
        let part = self.work().join(format!("{id}.result.part"));
        let _ = std::fs::remove_file(&part);
        write_new(
            &part,
            &serde_json::to_vec(v).map_err(std::io::Error::other)?,
        )?;
        std::fs::set_permissions(&part, std::fs::Permissions::from_mode(0o644))?;
        std::fs::rename(&part, self.results().join(format!("{id}.json")))
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    f.write_all(bytes)?;
    f.sync_all()
}

/// Open (without reading) a file the service placed: same checks as
/// `read_untrusted`, for inputs too large to hold in memory.
pub fn open_untrusted(path: &Path, max: u64, owner: Option<u32>) -> std::io::Result<File> {
    let f = File::options()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    let m = f.metadata()?;
    let ok = m.file_type().is_file()
        && m.nlink() == 1
        && owner.is_none_or(|uid| m.uid() == uid)
        && m.len() <= max;
    if !ok {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "{}: not a single-link regular file of the expected owner and size",
                path.display()
            ),
        ));
    }
    Ok(f)
}

/// Open a file an untrusted party may have placed: the final component must
/// not be a symlink, it must be a regular file with exactly one link, owned
/// by `owner` when given, and at most `max` bytes.
pub fn read_untrusted(path: &Path, max: u64, owner: Option<u32>) -> std::io::Result<Vec<u8>> {
    let f = File::options()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    let m = f.metadata()?;
    let refuse = |why: &str| {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!("{}: {why}", path.display()),
        ))
    };
    if !m.file_type().is_file() {
        return refuse("not a regular file");
    }
    if m.nlink() != 1 {
        return refuse("has more than one link");
    }
    if let Some(uid) = owner
        && m.uid() != uid
    {
        return refuse("unexpected owner");
    }
    if m.len() > max {
        return refuse("too large");
    }
    let mut out = Vec::with_capacity(m.len() as usize);
    f.take(max + 1).read_to_end(&mut out)?;
    if out.len() as u64 > max {
        return refuse("too large");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids() {
        let id = new_id("rel", 1_800_000_000);
        assert!(valid_id(&id, "rel"), "{id}");
        assert!(!valid_id(&id, "rm"));
        assert!(valid_id("rm-20270115080000-0a1b2c3d", "rm"));
        for bad in [
            "rel-2027011508000-0a1b2c3d",
            "rel-20270115080000-0A1B2C3D",
            "rel-20270115080000-0a1b2c3d/..",
            "rel-../../etc/passwd000000",
            "rel",
            "",
        ] {
            assert!(!valid_id(bad, "rel"), "{bad}");
        }
    }

    #[test]
    fn untrusted_reads_refuse_links() {
        let d = std::env::temp_dir().join(format!("rbp-spool-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("real"), b"hello").unwrap();
        std::os::unix::fs::symlink(d.join("real"), d.join("sym")).unwrap();
        std::fs::hard_link(d.join("real"), d.join("hard")).unwrap();
        assert!(read_untrusted(&d.join("sym"), 100, None).is_err());
        assert!(read_untrusted(&d.join("hard"), 100, None).is_err());
        std::fs::remove_file(d.join("hard")).unwrap();
        assert_eq!(
            read_untrusted(&d.join("real"), 100, None).unwrap(),
            b"hello"
        );
        assert!(read_untrusted(&d.join("real"), 3, None).is_err());
        assert!(read_untrusted(&d.join("real"), 100, Some(u32::MAX - 7)).is_err());
        assert!(read_untrusted(&d, 100, None).is_err());
        std::fs::remove_dir_all(&d).unwrap();
    }
}
