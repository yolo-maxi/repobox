//! External publisher: trusted first-party agents deploy their own apps.
//!
//! * [`manifest`]: the one JSON document a publisher sends (Git source +
//!   optional build/runtime settings, or an image override).
//! * [`spool`]: the queue between the unprivileged API and the root workers.
//! * [`worker`]: the deploy worker (clone, build, run, health-gate, route,
//!   rollback) and the runtime query worker (logs, container state).
//!
//! Ownership lives in the registry (`publishers`, `publisher_tokens`,
//! `publisher_releases`, `apps.publisher_id`); the HTTP surface is
//! `web::publisher`.

pub mod archive;
pub mod manifest;
pub mod spool;
pub mod worker;

/// The API reports a queued/building release as failed after this long
/// without a verdict (the worker refuses jobs older than
/// `worker::MAX_JOB_AGE_SECS`, and builds are bounded by its timeouts).
pub const JOB_TIMEOUT_SECS: i64 = 3600;
/// Apps one publisher may own.
pub const MAX_APPS_PER_PUBLISHER: i64 = 10;
/// Releases one publisher may start per UTC day.
pub const MAX_RELEASES_PER_DAY: i64 = 100;
/// Raw publisher tokens are `rbpub_` + 43 base64url characters.
pub const TOKEN_PREFIX: &str = "rbpub_";
/// Publisher token lifetime bounds.
pub const TOKEN_MIN_TTL: i64 = 3600;
pub const TOKEN_MAX_TTL: i64 = 30 * 86400;
