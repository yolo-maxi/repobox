//! SQLite-backed registry. One connection behind a mutex is plenty for a
//! control plane whose hot path is one indexed lookup per gated request.
//!
//! Invariants enforced here rather than in handlers:
//! * raw tokens and session secrets are never written, only their SHA-256;
//! * single-use tokens are consumed with a conditional UPDATE so a replay
//!   races cannot redeem twice;
//! * access checks always re-read user/app state so revocation, disabling and
//!   visibility changes take effect on the next request;
//! * access counting stores counters only: per app per UTC day, plus the user
//!   id of signed-in visitors of *private* apps for unique-user dedup. No URL,
//!   query, cookie, IP, user agent, token or body is ever written, and daily
//!   rows are deleted after [`ACCESS_RETENTION_DAYS`].

use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::model::{App, AppKind, Role, User, Visibility};
use crate::tokens;

pub type Clock = Arc<dyn Fn() -> i64 + Send + Sync>;

pub fn system_clock() -> Clock {
    Arc::new(|| chrono::Utc::now().timestamp())
}

#[derive(Debug)]
pub enum StoreError {
    NotFound,
    Conflict(String),
    Invalid(String),
    Db(rusqlite::Error),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::NotFound => write!(f, "not found"),
            StoreError::Conflict(m) => write!(f, "conflict: {m}"),
            StoreError::Invalid(m) => write!(f, "invalid: {m}"),
            StoreError::Db(e) => write!(f, "database error: {e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        match &e {
            rusqlite::Error::SqliteFailure(err, msg)
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                StoreError::Conflict(msg.clone().unwrap_or_else(|| "constraint violation".into()))
            }
            rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound,
            _ => StoreError::Db(e),
        }
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Launch,
    Enrol,
    Invite,
}

impl TokenKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TokenKind::Launch => "launch",
            TokenKind::Enrol => "enrol",
            TokenKind::Invite => "invite",
        }
    }
    fn parse(s: &str) -> Self {
        match s {
            "launch" => TokenKind::Launch,
            "enrol" => TokenKind::Enrol,
            _ => TokenKind::Invite,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    Auth,
    App,
}

impl SessionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SessionKind::Auth => "auth",
            SessionKind::App => "app",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub id: i64,
    pub kind: TokenKind,
    pub user_id: Option<i64>,
    pub app_id: Option<i64>,
    pub created_by: Option<i64>,
    pub created_at: i64,
    pub expires_at: i64,
    pub used_at: Option<i64>,
    pub used_by: Option<i64>,
    pub revoked_at: Option<i64>,
    pub note: String,
    /// Launch codes only: the device (auth) session that minted the code.
    pub session_id: Option<i64>,
}

impl Token {
    pub fn status(&self, now: i64) -> &'static str {
        if self.revoked_at.is_some() {
            "revoked"
        } else if self.used_at.is_some() {
            "used"
        } else if self.expires_at <= now {
            "expired"
        } else {
            "active"
        }
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: i64,
    pub kind: SessionKind,
    pub user_id: i64,
    pub app_id: Option<i64>,
    pub created_at: i64,
    pub expires_at: i64,
    pub last_seen_at: i64,
    pub revoked_at: Option<i64>,
    pub label: String,
    /// App sessions only: the device session that launched them. Revoking
    /// the device revokes these too.
    pub parent_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Grant {
    pub user: User,
    pub granted_by: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub at: i64,
    pub actor: Option<String>,
    pub action: String,
    pub subject: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedeemError {
    Unknown,
    Expired,
    Used,
    Revoked,
}

impl RedeemError {
    pub fn message(self) -> &'static str {
        match self {
            RedeemError::Unknown => "This link is not valid.",
            RedeemError::Expired => "This link has expired.",
            RedeemError::Used => "This link has already been used.",
            RedeemError::Revoked => "This link was revoked.",
        }
    }
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin', 'member')),
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS apps (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    owner_id INTEGER NOT NULL REFERENCES users(id),
    kind TEXT NOT NULL CHECK (kind IN ('static', 'proxy')),
    target TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'private'
        CHECK (visibility IN ('private', 'public_unlisted', 'public_listed')),
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS grants (
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id),
    granted_by INTEGER REFERENCES users(id),
    created_at INTEGER NOT NULL,
    PRIMARY KEY (app_id, user_id)
);
CREATE TABLE IF NOT EXISTS tokens (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('launch', 'enrol', 'invite')),
    token_hash TEXT NOT NULL UNIQUE,
    user_id INTEGER REFERENCES users(id),
    app_id INTEGER REFERENCES apps(id) ON DELETE CASCADE,
    created_by INTEGER REFERENCES users(id),
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    used_at INTEGER,
    used_by INTEGER REFERENCES users(id),
    revoked_at INTEGER,
    note TEXT NOT NULL DEFAULT '',
    session_id INTEGER REFERENCES sessions(id)
);
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('auth', 'app')),
    token_hash TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL REFERENCES users(id),
    app_id INTEGER REFERENCES apps(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    revoked_at INTEGER,
    label TEXT NOT NULL DEFAULT '',
    parent_id INTEGER REFERENCES sessions(id)
);
CREATE INDEX IF NOT EXISTS sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS tokens_app ON tokens(app_id);
CREATE INDEX IF NOT EXISTS tokens_user ON tokens(user_id);
CREATE TABLE IF NOT EXISTS audit (
    id INTEGER PRIMARY KEY,
    at INTEGER NOT NULL,
    actor_id INTEGER,
    action TEXT NOT NULL,
    subject TEXT NOT NULL,
    detail TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS access_daily (
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    day INTEGER NOT NULL,
    requests INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (app_id, day)
);
CREATE TABLE IF NOT EXISTS access_daily_users (
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    day INTEGER NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id),
    PRIMARY KEY (app_id, day, user_id)
);
CREATE TABLE IF NOT EXISTS access_totals (
    app_id INTEGER PRIMARY KEY REFERENCES apps(id) ON DELETE CASCADE,
    requests INTEGER NOT NULL DEFAULT 0,
    since_day INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', '3');
"#;

/// Columns added after the first release. `CREATE TABLE IF NOT EXISTS` does
/// not touch existing tables, so each is added with a guarded ALTER.
const ADDED_COLUMNS: &[(&str, &str, &str)] = &[
    ("tokens", "session_id", "INTEGER REFERENCES sessions(id)"),
    ("sessions", "parent_id", "INTEGER REFERENCES sessions(id)"),
];

/// Daily access counters (and the per-day user ids of private apps that
/// back the unique-user figure) are kept for this many UTC days, today
/// included. The all-time request total carries no identity and is kept.
pub const ACCESS_RETENTION_DAYS: i64 = 90;

/// UTC day number of a unix timestamp.
pub fn day_of(ts: i64) -> i64 {
    ts.div_euclid(86400)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayAccess {
    pub day: i64,
    pub requests: i64,
    /// Distinct signed-in users that day. Always 0 for public apps, whose
    /// visitors are never identified.
    pub users: i64,
}

#[derive(Debug, Clone)]
pub struct AppAnalytics {
    /// Allowed requests since `since_day` (all time, no identity).
    pub total_requests: i64,
    pub since_day: Option<i64>,
    pub window_days: i64,
    pub window_requests: i64,
    /// Distinct signed-in users over the retention window (private apps).
    pub window_users: i64,
    /// Newest first; one entry per day, zero days included.
    pub recent: Vec<DayAccess>,
}

/// What `Store::consolidate_users` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consolidation {
    pub kept_name: String,
    pub retired_name: String,
    pub apps: usize,
    pub grants: usize,
    pub sessions_revoked: usize,
    pub tokens_revoked: usize,
}

pub struct Store {
    conn: Mutex<Connection>,
    clock: Clock,
    /// Day on which expired access counters were last pruned (0 = never).
    access_pruned_day: AtomicI64,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| {
                StoreError::Invalid(format!("cannot create {}: {e}", parent.display()))
            })?;
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Self::init(conn, system_clock())
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?, system_clock())
    }

    pub fn open_in_memory_with_clock(clock: Clock) -> Result<Self> {
        Self::init(Connection::open_in_memory()?, clock)
    }

    fn init(conn: Connection, clock: Clock) -> Result<Self> {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        conn.execute_batch(SCHEMA)?;
        for (table, column, ddl) in ADDED_COLUMNS {
            let present = conn
                .prepare(&format!("PRAGMA table_info({table})"))?
                .query_map([], |r| r.get::<_, String>(1))?
                .any(|c| c.as_deref() == Ok(column));
            if !present {
                conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {ddl}"))?;
            }
        }
        Ok(Self {
            conn: Mutex::new(conn),
            clock,
            access_pruned_day: AtomicI64::new(0),
        })
    }

    pub fn now(&self) -> i64 {
        (self.clock)()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|p| p.into_inner())
    }

    // ----------------------------------------------------------------- users

    pub fn create_user(&self, name: &str, display_name: &str, role: Role) -> Result<User> {
        crate::model::validate_user_name(name).map_err(StoreError::Invalid)?;
        crate::model::validate_display_name(display_name).map_err(StoreError::Invalid)?;
        let now = self.now();
        let conn = self.lock();
        conn.execute(
            "INSERT INTO users (name, display_name, role, enabled, created_at) VALUES (?1, ?2, ?3, 1, ?4)",
            params![name, display_name.trim(), role.as_str(), now],
        )
        .map_err(|e| match StoreError::from(e) {
            StoreError::Conflict(_) => StoreError::Conflict(format!("user '{name}' already exists")),
            other => other,
        })?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.user_by_id(id)
    }

    pub fn user_by_id(&self, id: i64) -> Result<User> {
        let conn = self.lock();
        conn.query_row(
            "SELECT id, name, display_name, role, enabled, created_at FROM users WHERE id = ?1",
            params![id],
            row_user,
        )
        .map_err(Into::into)
    }

    pub fn user_by_name(&self, name: &str) -> Result<Option<User>> {
        let conn = self.lock();
        conn.query_row(
            "SELECT id, name, display_name, role, enabled, created_at FROM users WHERE name = ?1",
            params![name],
            row_user,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn list_users(&self) -> Result<Vec<User>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, display_name, role, enabled, created_at FROM users ORDER BY name",
        )?;
        let rows = stmt.query_map([], row_user)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn count_admins(&self) -> Result<i64> {
        let conn = self.lock();
        conn.query_row("SELECT COUNT(*) FROM users WHERE role = 'admin'", [], |r| {
            r.get(0)
        })
        .map_err(Into::into)
    }

    pub fn set_user_enabled(&self, id: i64, enabled: bool) -> Result<()> {
        let conn = self.lock();
        let n = conn.execute(
            "UPDATE users SET enabled = ?2 WHERE id = ?1",
            params![id, enabled as i64],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    pub fn set_user_role(&self, id: i64, role: Role) -> Result<()> {
        let conn = self.lock();
        let n = conn.execute(
            "UPDATE users SET role = ?2 WHERE id = ?1",
            params![id, role.as_str()],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    /// Fold `retire` into `keep` in one transaction. `keep` retains its row,
    /// id and therefore every device and app session it holds; it receives
    /// every app `retire` owned and every grant `retire` had, and is renamed
    /// to `new_name` when given. `retire` loses its name (it becomes
    /// `retired-<id>-<old name>`), is disabled, and every session and every
    /// unused token that pointed at it is revoked, so nothing can sign in as
    /// the retired identity afterwards. The audit row is written in the same
    /// transaction. `keep` must be enabled and must not be `retire`; an admin
    /// cannot be retired implicitly (demote first).
    pub fn consolidate_users(
        &self,
        keep: &User,
        retire: &User,
        new_name: Option<&str>,
        note: &str,
    ) -> Result<Consolidation> {
        if keep.id == retire.id {
            return Err(StoreError::Invalid(
                "keep and retire are the same user".into(),
            ));
        }
        if !keep.enabled {
            return Err(StoreError::Invalid(format!(
                "user '{}' is disabled; the kept identity must be enabled",
                keep.name
            )));
        }
        if retire.is_admin() {
            return Err(StoreError::Invalid(format!(
                "user '{}' is an admin; demote it before retiring it",
                retire.name
            )));
        }
        if let Some(n) = new_name {
            crate::model::validate_user_name(n).map_err(StoreError::Invalid)?;
        }
        let retired_name = {
            let prefix = format!("retired-{}-", retire.id);
            let room = 32 - prefix.len();
            let mut base = retire.name.clone();
            base.truncate(room);
            let base = base.trim_end_matches(['.', '_', '-']).to_string();
            format!("{prefix}{base}")
        };
        let now = self.now();
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        // the snapshots must still describe the rows we are about to change
        let cur: (String, i64) = tx.query_row(
            "SELECT name, enabled FROM users WHERE id = ?1",
            params![keep.id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if cur.0 != keep.name || cur.1 != 1 {
            return Err(StoreError::Conflict(format!(
                "user '{}' changed while consolidating; retry",
                keep.name
            )));
        }
        let cur: String = tx.query_row(
            "SELECT name FROM users WHERE id = ?1",
            params![retire.id],
            |r| r.get(0),
        )?;
        if cur != retire.name {
            return Err(StoreError::Conflict(format!(
                "user '{}' changed while consolidating; retry",
                retire.name
            )));
        }
        let apps = tx.execute(
            "UPDATE apps SET owner_id = ?2, updated_at = ?3 WHERE owner_id = ?1",
            params![retire.id, keep.id, now],
        )?;
        let grants = tx.execute(
            "INSERT OR IGNORE INTO grants (app_id, user_id, granted_by, created_at)
             SELECT app_id, ?2, granted_by, created_at FROM grants WHERE user_id = ?1",
            params![retire.id, keep.id],
        )?;
        tx.execute("DELETE FROM grants WHERE user_id = ?1", params![retire.id])?;
        let sessions = tx.execute(
            "UPDATE sessions SET revoked_at = ?2 WHERE user_id = ?1 AND revoked_at IS NULL",
            params![retire.id, now],
        )?;
        let tokens = tx.execute(
            "UPDATE tokens SET revoked_at = ?2
             WHERE (user_id = ?1 OR created_by = ?1) AND used_at IS NULL AND revoked_at IS NULL",
            params![retire.id, now],
        )?;
        // free the old name first so `keep` may take it
        tx.execute(
            "UPDATE users SET name = ?2, enabled = 0 WHERE id = ?1",
            params![retire.id, retired_name],
        )?;
        if let Some(n) = new_name {
            tx.execute(
                "UPDATE users SET name = ?2 WHERE id = ?1",
                params![keep.id, n],
            )
            .map_err(|e| match StoreError::from(e) {
                StoreError::Conflict(_) => {
                    StoreError::Conflict(format!("user name '{n}' is already taken"))
                }
                other => other,
            })?;
        }
        let final_name = new_name.unwrap_or(&keep.name).to_string();
        let detail = format!(
            "absorbed={} retired_as={} renamed_from={} apps={apps} grants={grants} sessions_revoked={sessions} tokens_revoked={tokens} {note}",
            retire.name, retired_name, keep.name
        );
        tx.execute(
            "INSERT INTO audit (at, actor_id, action, subject, detail) VALUES (?1, NULL, 'user.consolidate', ?2, ?3)",
            params![now, final_name, detail],
        )?;
        tx.commit()?;
        Ok(Consolidation {
            kept_name: final_name,
            retired_name,
            apps,
            grants,
            sessions_revoked: sessions,
            tokens_revoked: tokens,
        })
    }

    // ------------------------------------------------------------------ apps

    #[allow(clippy::too_many_arguments)]
    pub fn create_app(
        &self,
        name: &str,
        title: &str,
        description: &str,
        owner_id: i64,
        kind: AppKind,
        target: &str,
        visibility: Visibility,
    ) -> Result<App> {
        crate::model::validate_app_name(name).map_err(StoreError::Invalid)?;
        let target = crate::model::validate_target(kind, target).map_err(StoreError::Invalid)?;
        let title = title.trim();
        if title.is_empty() || title.chars().count() > 80 {
            return Err(StoreError::Invalid("title must be 1-80 characters".into()));
        }
        let now = self.now();
        let conn = self.lock();
        conn.execute(
            "INSERT INTO apps (name, title, description, owner_id, kind, target, visibility, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8)",
            params![name, title, description.trim(), owner_id, kind.as_str(), target, visibility.as_str(), now],
        )
        .map_err(|e| match StoreError::from(e) {
            StoreError::Conflict(_) => StoreError::Conflict(format!("app '{name}' already exists")),
            other => other,
        })?;
        let id = conn.last_insert_rowid();
        drop(conn);
        self.app_by_id(id)
    }

    /// Update the route-relevant fields of an existing app. Visibility,
    /// enabled state, owner and grants are deliberately untouched.
    pub fn update_app_route(
        &self,
        name: &str,
        title: &str,
        description: &str,
        kind: AppKind,
        target: &str,
    ) -> Result<App> {
        let target = crate::model::validate_target(kind, target).map_err(StoreError::Invalid)?;
        let now = self.now();
        let conn = self.lock();
        let n = conn.execute(
            "UPDATE apps SET title = ?2, description = ?3, kind = ?4, target = ?5, updated_at = ?6 WHERE name = ?1",
            params![name, title.trim(), description.trim(), kind.as_str(), target, now],
        )?;
        drop(conn);
        if n == 0 {
            return Err(StoreError::NotFound);
        }
        self.app_by_name(name)?.ok_or(StoreError::NotFound)
    }

    /// Move an app to a new owner in one transaction: the owner and
    /// `updated_at` change together with the audit row that records the
    /// transition, so the log can never disagree with the registry. Route,
    /// visibility, enabled state and grants are deliberately untouched. The
    /// new owner must exist and be enabled; transferring to the current owner
    /// is a no-op (`Ok(false)`) and writes no audit entry.
    pub fn transfer_app_owner(&self, app: &App, new_owner: &User, note: &str) -> Result<bool> {
        if !new_owner.enabled {
            return Err(StoreError::Invalid(format!(
                "user '{}' is disabled; enable them before transferring an app",
                new_owner.name
            )));
        }
        if app.owner_id == new_owner.id {
            return Ok(false);
        }
        let old_owner = self.user_by_id(app.owner_id)?;
        let now = self.now();
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        let n = tx.execute(
            "UPDATE apps SET owner_id = ?2, updated_at = ?3 WHERE id = ?1 AND owner_id = ?4",
            params![app.id, new_owner.id, now, app.owner_id],
        )?;
        if n == 0 {
            return Err(StoreError::Conflict(format!(
                "app '{}' changed while transferring; re-run `app show` and retry",
                app.name
            )));
        }
        tx.execute(
            "INSERT INTO audit (at, actor_id, action, subject, detail) VALUES (?1, NULL, 'app.transfer_owner', ?2, ?3)",
            params![
                now,
                app.name,
                format!("from={} to={} {}", old_owner.name, new_owner.name, note)
            ],
        )?;
        tx.commit()?;
        Ok(true)
    }

    pub fn app_by_id(&self, id: i64) -> Result<App> {
        let conn = self.lock();
        conn.query_row(&format!("{APP_SELECT} WHERE id = ?1"), params![id], row_app)
            .map_err(Into::into)
    }

    pub fn app_by_name(&self, name: &str) -> Result<Option<App>> {
        let conn = self.lock();
        conn.query_row(
            &format!("{APP_SELECT} WHERE name = ?1"),
            params![name],
            row_app,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn list_apps(&self) -> Result<Vec<App>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(&format!("{APP_SELECT} ORDER BY name"))?;
        let rows = stmt.query_map([], row_app)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn set_app_visibility(&self, id: i64, visibility: Visibility) -> Result<()> {
        let now = self.now();
        let conn = self.lock();
        let n = conn.execute(
            "UPDATE apps SET visibility = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, visibility.as_str(), now],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    pub fn set_app_enabled(&self, id: i64, enabled: bool) -> Result<()> {
        let now = self.now();
        let conn = self.lock();
        let n = conn.execute(
            "UPDATE apps SET enabled = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, enabled as i64, now],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    pub fn delete_app(&self, name: &str) -> Result<()> {
        let conn = self.lock();
        let n = conn.execute("DELETE FROM apps WHERE name = ?1", params![name])?;
        if n == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    // ---------------------------------------------------------------- grants

    pub fn add_grant(&self, app_id: i64, user_id: i64, granted_by: Option<i64>) -> Result<bool> {
        let now = self.now();
        let conn = self.lock();
        let n = conn.execute(
            "INSERT OR IGNORE INTO grants (app_id, user_id, granted_by, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![app_id, user_id, granted_by, now],
        )?;
        Ok(n == 1)
    }

    pub fn remove_grant(&self, app_id: i64, user_id: i64) -> Result<bool> {
        let conn = self.lock();
        let n = conn.execute(
            "DELETE FROM grants WHERE app_id = ?1 AND user_id = ?2",
            params![app_id, user_id],
        )?;
        Ok(n == 1)
    }

    pub fn has_grant(&self, app_id: i64, user_id: i64) -> Result<bool> {
        let conn = self.lock();
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM grants WHERE app_id = ?1 AND user_id = ?2",
            params![app_id, user_id],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn list_grants(&self, app_id: i64) -> Result<Vec<Grant>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT u.id, u.name, u.display_name, u.role, u.enabled, u.created_at, g.created_at, b.name
             FROM grants g
             JOIN users u ON u.id = g.user_id
             LEFT JOIN users b ON b.id = g.granted_by
             WHERE g.app_id = ?1 ORDER BY u.name",
        )?;
        let rows = stmt.query_map(params![app_id], |r| {
            Ok(Grant {
                user: row_user(r)?,
                created_at: r.get(6)?,
                granted_by: r.get(7)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Can this user open this app? Admins and owners always can; members
    /// need a grant. A disabled user never can. This does not consider app
    /// visibility or enabled state: callers check those separately so the
    /// reason for a denial stays explicit.
    pub fn has_access(&self, user: &User, app: &App) -> Result<bool> {
        if !user.enabled {
            return Ok(false);
        }
        if user.is_admin() || app.owner_id == user.id {
            return Ok(true);
        }
        self.has_grant(app.id, user.id)
    }

    pub fn can_manage(&self, user: &User, app: &App) -> bool {
        user.enabled && (user.is_admin() || app.owner_id == user.id)
    }

    // ---------------------------------------------------------------- tokens

    pub fn create_token(
        &self,
        kind: TokenKind,
        user_id: Option<i64>,
        app_id: Option<i64>,
        created_by: Option<i64>,
        ttl_secs: i64,
        note: &str,
    ) -> Result<(String, Token)> {
        let raw = tokens::generate();
        let hash = tokens::hash(&raw);
        let now = self.now();
        let conn = self.lock();
        conn.execute(
            "INSERT INTO tokens (kind, token_hash, user_id, app_id, created_by, created_at, expires_at, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![kind.as_str(), hash, user_id, app_id, created_by, now, now + ttl_secs, note],
        )?;
        let id = conn.last_insert_rowid();
        drop(conn);
        Ok((raw, self.token_by_id(id)?))
    }

    /// Mint a one-time launch code bound to a user, an app and the device
    /// session that asked for it, so the resulting app session can be tied
    /// back to (and revoked with) that device.
    pub fn create_launch_code(
        &self,
        user_id: i64,
        app_id: i64,
        session_id: i64,
        ttl_secs: i64,
    ) -> Result<(String, Token)> {
        let raw = tokens::generate();
        let hash = tokens::hash(&raw);
        let now = self.now();
        let conn = self.lock();
        conn.execute(
            "INSERT INTO tokens (kind, token_hash, user_id, app_id, created_by, created_at, expires_at, note, session_id)
             VALUES ('launch', ?1, ?2, ?3, ?2, ?4, ?5, '', ?6)",
            params![hash, user_id, app_id, now, now + ttl_secs, session_id],
        )?;
        let id = conn.last_insert_rowid();
        drop(conn);
        Ok((raw, self.token_by_id(id)?))
    }

    pub fn token_by_id(&self, id: i64) -> Result<Token> {
        let conn = self.lock();
        conn.query_row(
            &format!("{TOKEN_SELECT} WHERE id = ?1"),
            params![id],
            row_token,
        )
        .map_err(Into::into)
    }

    fn token_by_raw(&self, kind: TokenKind, raw: &str) -> std::result::Result<Token, RedeemError> {
        if !tokens::looks_like_token(raw) {
            return Err(RedeemError::Unknown);
        }
        let hash = tokens::hash(raw);
        let conn = self.lock();
        let tok = conn
            .query_row(
                &format!("{TOKEN_SELECT} WHERE token_hash = ?1"),
                params![hash],
                row_token,
            )
            .optional()
            .map_err(|_| RedeemError::Unknown)?
            .ok_or(RedeemError::Unknown)?;
        if tok.kind != kind {
            return Err(RedeemError::Unknown);
        }
        Ok(tok)
    }

    /// Validate a token without consuming it (used to render the confirmation
    /// page before the POST that actually redeems).
    pub fn peek_token(
        &self,
        kind: TokenKind,
        raw: &str,
    ) -> std::result::Result<Token, RedeemError> {
        let tok = self.token_by_raw(kind, raw)?;
        self.check_token(&tok)?;
        Ok(tok)
    }

    fn check_token(&self, tok: &Token) -> std::result::Result<(), RedeemError> {
        if tok.revoked_at.is_some() {
            return Err(RedeemError::Revoked);
        }
        if tok.used_at.is_some() {
            return Err(RedeemError::Used);
        }
        if tok.expires_at <= self.now() {
            return Err(RedeemError::Expired);
        }
        Ok(())
    }

    /// Atomically consume a single-use token. The conditional UPDATE means two
    /// concurrent redemptions cannot both succeed.
    pub fn consume_token(
        &self,
        kind: TokenKind,
        raw: &str,
        used_by: Option<i64>,
    ) -> std::result::Result<Token, RedeemError> {
        let tok = self.token_by_raw(kind, raw)?;
        self.check_token(&tok)?;
        let now = self.now();
        let conn = self.lock();
        let n = conn
            .execute(
                "UPDATE tokens SET used_at = ?2, used_by = ?3
                 WHERE id = ?1 AND used_at IS NULL AND revoked_at IS NULL AND expires_at > ?2",
                params![tok.id, now, used_by],
            )
            .map_err(|_| RedeemError::Unknown)?;
        if n != 1 {
            return Err(RedeemError::Used);
        }
        Ok(Token {
            used_at: Some(now),
            used_by,
            ..tok
        })
    }

    pub fn revoke_token(&self, id: i64) -> Result<()> {
        let now = self.now();
        let conn = self.lock();
        let n = conn.execute(
            "UPDATE tokens SET revoked_at = ?2 WHERE id = ?1 AND revoked_at IS NULL",
            params![id, now],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    pub fn list_app_tokens(&self, kind: TokenKind, app_id: i64) -> Result<Vec<Token>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(&format!(
            "{TOKEN_SELECT} WHERE kind = ?1 AND app_id = ?2 ORDER BY created_at DESC LIMIT 50"
        ))?;
        let rows = stmt.query_map(params![kind.as_str(), app_id], row_token)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn list_user_tokens(&self, kind: TokenKind, user_id: i64) -> Result<Vec<Token>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(&format!(
            "{TOKEN_SELECT} WHERE kind = ?1 AND user_id = ?2 ORDER BY created_at DESC LIMIT 50"
        ))?;
        let rows = stmt.query_map(params![kind.as_str(), user_id], row_token)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    // -------------------------------------------------------------- sessions

    /// `parent_id` is the device session an app session was launched from
    /// (None for device sessions themselves).
    pub fn create_session(
        &self,
        kind: SessionKind,
        user_id: i64,
        app_id: Option<i64>,
        parent_id: Option<i64>,
        ttl_secs: i64,
        label: &str,
    ) -> Result<(String, Session)> {
        let raw = tokens::generate();
        let hash = tokens::hash(&raw);
        let now = self.now();
        let conn = self.lock();
        conn.execute(
            "INSERT INTO sessions (kind, token_hash, user_id, app_id, created_at, expires_at, last_seen_at, label, parent_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?5, ?7, ?8)",
            params![kind.as_str(), hash, user_id, app_id, now, now + ttl_secs, label, parent_id],
        )?;
        let id = conn.last_insert_rowid();
        let sess = conn.query_row(
            &format!("{SESSION_SELECT} WHERE id = ?1"),
            params![id],
            row_session,
        )?;
        Ok((raw, sess))
    }

    /// Resolve a session secret to a live session and its (enabled) user.
    /// Returns None for anything not currently valid.
    pub fn session_lookup(&self, kind: SessionKind, raw: &str) -> Result<Option<(Session, User)>> {
        if !tokens::looks_like_token(raw) {
            return Ok(None);
        }
        let hash = tokens::hash(raw);
        let now = self.now();
        let conn = self.lock();
        let found = conn
            .query_row(
                &format!("{SESSION_SELECT} WHERE token_hash = ?1"),
                params![hash],
                row_session,
            )
            .optional()?;
        let Some(sess) = found else { return Ok(None) };
        if sess.kind != kind || sess.revoked_at.is_some() || sess.expires_at <= now {
            return Ok(None);
        }
        let user = conn.query_row(
            "SELECT id, name, display_name, role, enabled, created_at FROM users WHERE id = ?1",
            params![sess.user_id],
            row_user,
        )?;
        if !user.enabled {
            return Ok(None);
        }
        if now - sess.last_seen_at > 60 {
            conn.execute(
                "UPDATE sessions SET last_seen_at = ?2 WHERE id = ?1",
                params![sess.id, now],
            )?;
        }
        Ok(Some((sess, user)))
    }

    pub fn session_by_id(&self, id: i64) -> Result<Option<Session>> {
        let conn = self.lock();
        conn.query_row(
            &format!("{SESSION_SELECT} WHERE id = ?1"),
            params![id],
            row_session,
        )
        .optional()
        .map_err(Into::into)
    }

    /// Revoke one session of `user_id`, plus every app session launched from
    /// it if it is a device session. Scoped to the user so a caller can never
    /// revoke somebody else's session by id. Returns whether the session
    /// itself was live and is now revoked; the gate and the UI see the
    /// change on the next request because every lookup re-reads `revoked_at`.
    pub fn revoke_session(&self, id: i64, user_id: i64) -> Result<bool> {
        let now = self.now();
        let conn = self.lock();
        let n = conn.execute(
            "UPDATE sessions SET revoked_at = ?3 WHERE id = ?1 AND user_id = ?2 AND revoked_at IS NULL",
            params![id, user_id, now],
        )?;
        conn.execute(
            "UPDATE sessions SET revoked_at = ?3 WHERE parent_id = ?1 AND user_id = ?2 AND revoked_at IS NULL",
            params![id, user_id, now],
        )?;
        Ok(n == 1)
    }

    pub fn revoke_user_sessions(&self, user_id: i64) -> Result<usize> {
        let now = self.now();
        let conn = self.lock();
        let n = conn.execute(
            "UPDATE sessions SET revoked_at = ?2 WHERE user_id = ?1 AND revoked_at IS NULL",
            params![user_id, now],
        )?;
        Ok(n)
    }

    pub fn list_sessions(&self, user_id: i64, kind: SessionKind) -> Result<Vec<Session>> {
        let now = self.now();
        let conn = self.lock();
        let mut stmt = conn.prepare(&format!(
            "{SESSION_SELECT} WHERE user_id = ?1 AND kind = ?2 AND revoked_at IS NULL AND expires_at > ?3
             ORDER BY last_seen_at DESC LIMIT 50"
        ))?;
        let rows = stmt.query_map(params![user_id, kind.as_str(), now], row_session)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    // ----------------------------------------------------------------- audit

    pub fn audit(&self, actor_id: Option<i64>, action: &str, subject: &str, detail: &str) {
        let now = self.now();
        let conn = self.lock();
        let _ = conn.execute(
            "INSERT INTO audit (at, actor_id, action, subject, detail) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![now, actor_id, action, subject, detail],
        );
    }

    pub fn list_audit(&self, limit: i64) -> Result<Vec<AuditEntry>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(
            "SELECT a.at, u.name, a.action, a.subject, a.detail FROM audit a
             LEFT JOIN users u ON u.id = a.actor_id ORDER BY a.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |r| {
            Ok(AuditEntry {
                at: r.get(0)?,
                actor: r.get(1)?,
                action: r.get(2)?,
                subject: r.get(3)?,
                detail: r.get(4)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    // ---------------------------------------------------------------- access

    /// Count one request the gate allowed through for `app_id`. `user_id` is
    /// the signed-in visitor of a *private* app (callers pass `None` for
    /// public apps, so no identity is ever attached to public traffic). Only
    /// counters are written; nothing about the request itself is stored.
    pub fn record_access(&self, app_id: i64, user_id: Option<i64>) -> Result<()> {
        let day = day_of(self.now());
        let conn = self.lock();
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO access_totals (app_id, requests, since_day) VALUES (?1, 1, ?2)
             ON CONFLICT(app_id) DO UPDATE SET requests = requests + 1",
            params![app_id, day],
        )?;
        tx.execute(
            "INSERT INTO access_daily (app_id, day, requests) VALUES (?1, ?2, 1)
             ON CONFLICT(app_id, day) DO UPDATE SET requests = requests + 1",
            params![app_id, day],
        )?;
        if let Some(uid) = user_id {
            tx.execute(
                "INSERT OR IGNORE INTO access_daily_users (app_id, day, user_id) VALUES (?1, ?2, ?3)",
                params![app_id, day, uid],
            )?;
        }
        if self.access_pruned_day.swap(day, Ordering::Relaxed) != day {
            let cutoff = day - ACCESS_RETENTION_DAYS;
            tx.execute("DELETE FROM access_daily WHERE day <= ?1", params![cutoff])?;
            tx.execute(
                "DELETE FROM access_daily_users WHERE day <= ?1",
                params![cutoff],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Aggregate counters for one app: all-time total, the retention window,
    /// and `recent_days` per-day rows (today first).
    pub fn app_analytics(&self, app_id: i64, recent_days: i64) -> Result<AppAnalytics> {
        let today = day_of(self.now());
        let cutoff = today - ACCESS_RETENTION_DAYS;
        let conn = self.lock();
        let (total_requests, since_day) = conn
            .query_row(
                "SELECT requests, since_day FROM access_totals WHERE app_id = ?1",
                params![app_id],
                |r| Ok((r.get::<_, i64>(0)?, Some(r.get::<_, i64>(1)?))),
            )
            .optional()?
            .unwrap_or((0, None));
        let window_requests: i64 = conn.query_row(
            "SELECT COALESCE(SUM(requests), 0) FROM access_daily WHERE app_id = ?1 AND day > ?2",
            params![app_id, cutoff],
            |r| r.get(0),
        )?;
        let window_users: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT user_id) FROM access_daily_users WHERE app_id = ?1 AND day > ?2",
            params![app_id, cutoff],
            |r| r.get(0),
        )?;
        let mut by_day =
            conn.prepare("SELECT requests FROM access_daily WHERE app_id = ?1 AND day = ?2")?;
        let mut users_by_day =
            conn.prepare("SELECT COUNT(*) FROM access_daily_users WHERE app_id = ?1 AND day = ?2")?;
        let mut recent = Vec::with_capacity(recent_days.max(0) as usize);
        for i in 0..recent_days.max(0) {
            let day = today - i;
            let requests: i64 = by_day
                .query_row(params![app_id, day], |r| r.get(0))
                .optional()?
                .unwrap_or(0);
            let users: i64 = users_by_day.query_row(params![app_id, day], |r| r.get(0))?;
            recent.push(DayAccess {
                day,
                requests,
                users,
            });
        }
        Ok(AppAnalytics {
            total_requests,
            since_day,
            window_days: ACCESS_RETENTION_DAYS,
            window_requests,
            window_users,
            recent,
        })
    }

    // ---------------------------------------------------------------- backup

    /// Consistent online backup using SQLite's backup API (safe while the
    /// service is running and the database is in WAL mode).
    pub fn backup_to(&self, dest: &Path) -> Result<()> {
        let conn = self.lock();
        let mut dst = Connection::open(dest)?;
        let backup = rusqlite::backup::Backup::new(&conn, &mut dst)?;
        backup.run_to_completion(256, std::time::Duration::from_millis(20), None)?;
        Ok(())
    }
}

const APP_SELECT: &str = "SELECT id, name, title, description, owner_id, kind, target, visibility, enabled, created_at, updated_at FROM apps";
const TOKEN_SELECT: &str = "SELECT id, kind, user_id, app_id, created_by, created_at, expires_at, used_at, used_by, revoked_at, note, session_id FROM tokens";
const SESSION_SELECT: &str = "SELECT id, kind, user_id, app_id, created_at, expires_at, last_seen_at, revoked_at, label, parent_id FROM sessions";

fn row_user(r: &Row<'_>) -> rusqlite::Result<User> {
    let role: String = r.get(3)?;
    Ok(User {
        id: r.get(0)?,
        name: r.get(1)?,
        display_name: r.get(2)?,
        role: Role::parse(&role).unwrap_or(Role::Member),
        enabled: r.get::<_, i64>(4)? != 0,
        created_at: r.get(5)?,
    })
}

fn row_app(r: &Row<'_>) -> rusqlite::Result<App> {
    let kind: String = r.get(5)?;
    let vis: String = r.get(7)?;
    Ok(App {
        id: r.get(0)?,
        name: r.get(1)?,
        title: r.get(2)?,
        description: r.get(3)?,
        owner_id: r.get(4)?,
        kind: AppKind::parse(&kind).unwrap_or(AppKind::Proxy),
        target: r.get(6)?,
        visibility: Visibility::parse(&vis).unwrap_or(Visibility::Private),
        enabled: r.get::<_, i64>(8)? != 0,
        created_at: r.get(9)?,
        updated_at: r.get(10)?,
    })
}

fn row_token(r: &Row<'_>) -> rusqlite::Result<Token> {
    let kind: String = r.get(1)?;
    Ok(Token {
        id: r.get(0)?,
        kind: TokenKind::parse(&kind),
        user_id: r.get(2)?,
        app_id: r.get(3)?,
        created_by: r.get(4)?,
        created_at: r.get(5)?,
        expires_at: r.get(6)?,
        used_at: r.get(7)?,
        used_by: r.get(8)?,
        revoked_at: r.get(9)?,
        note: r.get(10)?,
        session_id: r.get(11)?,
    })
}

fn row_session(r: &Row<'_>) -> rusqlite::Result<Session> {
    let kind: String = r.get(1)?;
    Ok(Session {
        id: r.get(0)?,
        kind: if kind == "auth" {
            SessionKind::Auth
        } else {
            SessionKind::App
        },
        user_id: r.get(2)?,
        app_id: r.get(3)?,
        created_at: r.get(4)?,
        expires_at: r.get(5)?,
        last_seen_at: r.get(6)?,
        revoked_at: r.get(7)?,
        label: r.get(8)?,
        parent_id: r.get(9)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI64, Ordering};

    fn store_with_clock() -> (Store, Arc<AtomicI64>) {
        let t = Arc::new(AtomicI64::new(1_000_000));
        let tc = t.clone();
        let s =
            Store::open_in_memory_with_clock(Arc::new(move || tc.load(Ordering::SeqCst))).unwrap();
        (s, t)
    }

    #[test]
    fn tokens_are_single_use_and_expire() {
        let (s, clock) = store_with_clock();
        let u = s.create_user("fran", "Fran", Role::Admin).unwrap();
        let (raw, tok) = s
            .create_token(TokenKind::Enrol, Some(u.id), None, None, 60, "")
            .unwrap();
        assert_eq!(tok.status(s.now()), "active");
        assert!(s.peek_token(TokenKind::Enrol, &raw).is_ok());
        // wrong kind is not distinguishable from unknown
        assert_eq!(
            s.peek_token(TokenKind::Invite, &raw).unwrap_err(),
            RedeemError::Unknown
        );
        let used = s.consume_token(TokenKind::Enrol, &raw, Some(u.id)).unwrap();
        assert!(used.used_at.is_some());
        assert_eq!(
            s.consume_token(TokenKind::Enrol, &raw, Some(u.id))
                .unwrap_err(),
            RedeemError::Used
        );

        let (raw2, _) = s
            .create_token(TokenKind::Enrol, Some(u.id), None, None, 60, "")
            .unwrap();
        clock.fetch_add(61, Ordering::SeqCst);
        assert_eq!(
            s.consume_token(TokenKind::Enrol, &raw2, None).unwrap_err(),
            RedeemError::Expired
        );

        let (raw3, t3) = s
            .create_token(TokenKind::Enrol, Some(u.id), None, None, 60, "")
            .unwrap();
        s.revoke_token(t3.id).unwrap();
        assert_eq!(
            s.consume_token(TokenKind::Enrol, &raw3, None).unwrap_err(),
            RedeemError::Revoked
        );
        assert_eq!(
            s.consume_token(TokenKind::Enrol, "nope", None).unwrap_err(),
            RedeemError::Unknown
        );
    }

    #[test]
    fn raw_secrets_never_stored() {
        let (s, _) = store_with_clock();
        let u = s.create_user("fran", "Fran", Role::Admin).unwrap();
        let (raw_t, _) = s
            .create_token(TokenKind::Launch, Some(u.id), None, None, 60, "")
            .unwrap();
        let (raw_s, _) = s
            .create_session(SessionKind::Auth, u.id, None, None, 60, "")
            .unwrap();
        let conn = s.lock();
        let mut stmt = conn
            .prepare("SELECT token_hash FROM tokens UNION ALL SELECT token_hash FROM sessions")
            .unwrap();
        let hashes: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(hashes.len(), 2);
        for h in hashes {
            assert_ne!(h, raw_t);
            assert_ne!(h, raw_s);
            assert_eq!(h.len(), 64);
        }
    }

    #[test]
    fn sessions_respect_revocation_expiry_and_user_state() {
        let (s, clock) = store_with_clock();
        let u = s.create_user("bob", "Bob", Role::Member).unwrap();
        let (raw, sess) = s
            .create_session(SessionKind::App, u.id, None, None, 100, "")
            .unwrap();
        assert!(s.session_lookup(SessionKind::App, &raw).unwrap().is_some());
        assert!(
            s.session_lookup(SessionKind::Auth, &raw).unwrap().is_none(),
            "kind must match"
        );
        s.set_user_enabled(u.id, false).unwrap();
        assert!(
            s.session_lookup(SessionKind::App, &raw).unwrap().is_none(),
            "disabled user"
        );
        s.set_user_enabled(u.id, true).unwrap();
        assert!(s.revoke_session(sess.id, u.id).unwrap());
        assert!(
            s.session_lookup(SessionKind::App, &raw).unwrap().is_none(),
            "revoked"
        );
        let (raw2, _) = s
            .create_session(SessionKind::App, u.id, None, None, 100, "")
            .unwrap();
        clock.fetch_add(101, Ordering::SeqCst);
        assert!(
            s.session_lookup(SessionKind::App, &raw2).unwrap().is_none(),
            "expired"
        );
    }

    #[test]
    fn access_rules() {
        let (s, _) = store_with_clock();
        let admin = s.create_user("fran", "Fran", Role::Admin).unwrap();
        let owner = s.create_user("owner", "Owner", Role::Member).unwrap();
        let bob = s.create_user("bob", "Bob", Role::Member).unwrap();
        let app = s
            .create_app(
                "demo",
                "Demo",
                "",
                owner.id,
                AppKind::Proxy,
                "127.0.0.1:3231",
                Visibility::Private,
            )
            .unwrap();
        assert!(s.has_access(&admin, &app).unwrap());
        assert!(s.has_access(&owner, &app).unwrap());
        assert!(!s.has_access(&bob, &app).unwrap());
        assert!(s.add_grant(app.id, bob.id, Some(owner.id)).unwrap());
        assert!(
            !s.add_grant(app.id, bob.id, Some(owner.id)).unwrap(),
            "idempotent"
        );
        assert!(s.has_access(&bob, &app).unwrap());
        assert!(s.remove_grant(app.id, bob.id).unwrap());
        assert!(!s.has_access(&bob, &app).unwrap());
        assert!(
            s.create_app(
                "demo",
                "Dup",
                "",
                owner.id,
                AppKind::Proxy,
                "127.0.0.1:1",
                Visibility::Private
            )
            .is_err()
        );
        assert!(
            s.create_app(
                "evil",
                "E",
                "",
                owner.id,
                AppKind::Proxy,
                "10.0.0.1:80",
                Visibility::Private
            )
            .is_err()
        );
    }

    #[test]
    fn transfer_owner_moves_ownership_and_keeps_everything_else() {
        let (s, clock) = store_with_clock();
        let dup = s.create_user("ellie-dup", "Ellie", Role::Member).unwrap();
        let ellie = s.create_user("ellie", "Ellie", Role::Member).unwrap();
        let bob = s.create_user("bob", "Bob", Role::Member).unwrap();
        let app = s
            .create_app(
                "diary",
                "Diary",
                "desc",
                dup.id,
                AppKind::Proxy,
                "127.0.0.1:3025",
                Visibility::PublicUnlisted,
            )
            .unwrap();
        s.set_app_enabled(app.id, false).unwrap();
        let app = s.app_by_id(app.id).unwrap();
        assert!(s.add_grant(app.id, bob.id, Some(dup.id)).unwrap());
        assert!(s.add_grant(app.id, dup.id, None).unwrap());
        let before = s.list_audit(10).unwrap().len();

        clock.fetch_add(100, Ordering::SeqCst);
        assert!(s.transfer_app_owner(&app, &ellie, "cli").unwrap());
        let after = s.app_by_id(app.id).unwrap();
        assert_eq!(after.owner_id, ellie.id);
        assert_eq!(after.updated_at, app.updated_at + 100);
        // untouched: route, title, description, visibility, enabled, grants
        assert_eq!(after.kind, app.kind);
        assert_eq!(after.target, app.target);
        assert_eq!(after.title, app.title);
        assert_eq!(after.description, app.description);
        assert_eq!(after.visibility, Visibility::PublicUnlisted);
        assert!(!after.enabled);
        assert_eq!(after.created_at, app.created_at);
        let grants: Vec<String> = s
            .list_grants(app.id)
            .unwrap()
            .into_iter()
            .map(|g| g.user.name)
            .collect();
        assert!(grants.contains(&"bob".to_string()));
        assert!(grants.contains(&"ellie-dup".to_string()));
        // access follows ownership
        assert!(s.has_access(&ellie, &after).unwrap());
        assert!(s.can_manage(&ellie, &after));
        assert!(!s.can_manage(&s.user_by_id(dup.id).unwrap(), &after));
        assert!(
            s.has_access(&s.user_by_id(dup.id).unwrap(), &after)
                .unwrap(),
            "the former owner keeps only its explicit grant"
        );
        // exactly one audit row, describing the transition
        let audit = s.list_audit(10).unwrap();
        assert_eq!(audit.len(), before + 1);
        let e = &audit[0];
        assert_eq!(e.action, "app.transfer_owner");
        assert_eq!(e.subject, "diary");
        assert_eq!(e.detail, "from=ellie-dup to=ellie cli");
        assert_eq!(e.at, app.updated_at + 100);

        // same owner: no-op, no audit
        assert!(!s.transfer_app_owner(&after, &ellie, "cli").unwrap());
        assert_eq!(s.list_audit(10).unwrap().len(), before + 1);
    }

    #[test]
    fn transfer_owner_refuses_disabled_owner_and_stale_app() {
        let (s, _) = store_with_clock();
        let a = s.create_user("a", "A", Role::Member).unwrap();
        let b = s.create_user("b", "B", Role::Member).unwrap();
        let c = s.create_user("c", "C", Role::Member).unwrap();
        let d = s.create_user("d", "D", Role::Member).unwrap();
        let app = s
            .create_app(
                "x",
                "X",
                "",
                a.id,
                AppKind::Static,
                "/srv/repobox-platform/apps/x",
                Visibility::Private,
            )
            .unwrap();
        s.set_user_enabled(b.id, false).unwrap();
        let b = s.user_by_id(b.id).unwrap();
        let err = s.transfer_app_owner(&app, &b, "cli").unwrap_err();
        assert!(matches!(err, StoreError::Invalid(_)), "{err}");
        assert_eq!(s.app_by_id(app.id).unwrap().owner_id, a.id);
        assert!(
            s.list_audit(10)
                .unwrap()
                .iter()
                .all(|e| e.action != "app.transfer_owner")
        );

        // a stale App snapshot (owner already changed underneath) is rejected
        assert!(s.transfer_app_owner(&app, &c, "cli").unwrap());
        let err = s.transfer_app_owner(&app, &d, "cli").unwrap_err();
        assert!(matches!(err, StoreError::Conflict(_)), "{err}");
        assert_eq!(s.app_by_id(app.id).unwrap().owner_id, c.id);
        // a deleted app is rejected the same way
        let fresh = s.app_by_id(app.id).unwrap();
        s.delete_app("x").unwrap();
        assert!(s.transfer_app_owner(&fresh, &a, "cli").is_err());
    }

    #[test]
    fn consolidate_keeps_sessions_of_kept_user_and_retires_the_other() {
        let (s, clock) = store_with_clock();
        // the enrolled identity (kept) and the unused duplicate (retired)
        let keep = s
            .create_user("ellie-beaumont-study-diary", "Ellie Beaumont", Role::Member)
            .unwrap();
        let retire = s
            .create_user("ellie-beaumont", "Ellie Beaumont", Role::Member)
            .unwrap();
        let fran = s.create_user("fran", "Fran", Role::Admin).unwrap();
        let diary = s
            .create_app(
                "study-diary",
                "Study Diary",
                "",
                keep.id,
                AppKind::Proxy,
                "127.0.0.1:3025",
                Visibility::Private,
            )
            .unwrap();
        let kitchen = s
            .create_app(
                "uni-kitchen",
                "Uni Kitchen",
                "",
                retire.id,
                AppKind::Proxy,
                "127.0.0.1:4184",
                Visibility::Private,
            )
            .unwrap();
        let shared = s
            .create_app(
                "shared",
                "Shared",
                "",
                fran.id,
                AppKind::Proxy,
                "127.0.0.1:1",
                Visibility::Private,
            )
            .unwrap();
        // grants: retire has one on `shared` and one on `diary`; keep already has `shared`
        assert!(s.add_grant(shared.id, retire.id, Some(fran.id)).unwrap());
        assert!(s.add_grant(diary.id, retire.id, None).unwrap());
        assert!(s.add_grant(shared.id, keep.id, Some(fran.id)).unwrap());
        // sessions: keep has a device + a launched app session, retire has a device session
        let (dev_raw, dev) = s
            .create_session(SessionKind::Auth, keep.id, None, None, 3600, "Chrome")
            .unwrap();
        let (app_raw, _) = s
            .create_session(
                SessionKind::App,
                keep.id,
                Some(diary.id),
                Some(dev.id),
                3600,
                "Chrome",
            )
            .unwrap();
        let (r_raw, _) = s
            .create_session(SessionKind::Auth, retire.id, None, None, 3600, "Other")
            .unwrap();
        // tokens: an unredeemed enrol link for retire, a used one for keep, a link issued by retire
        let (r_link, _) = s
            .create_token(TokenKind::Enrol, Some(retire.id), None, None, 3600, "")
            .unwrap();
        let (k_link, _) = s
            .create_token(TokenKind::Enrol, Some(keep.id), None, None, 3600, "")
            .unwrap();
        s.consume_token(TokenKind::Enrol, &k_link, Some(keep.id))
            .unwrap();
        let (issued, _) = s
            .create_token(
                TokenKind::Invite,
                None,
                Some(shared.id),
                Some(retire.id),
                3600,
                "",
            )
            .unwrap();
        let before = s.list_audit(50).unwrap().len();

        clock.fetch_add(10, Ordering::SeqCst);
        let c = s
            .consolidate_users(&keep, &retire, Some("ellie-beaumont"), "cli")
            .unwrap();
        assert_eq!(c.kept_name, "ellie-beaumont");
        assert_eq!(c.retired_name, "retired-2-ellie-beaumont");
        assert_eq!(
            (c.apps, c.grants, c.sessions_revoked, c.tokens_revoked),
            (1, 1, 1, 2)
        );

        // one identity named ellie-beaumont, same id as before, still enabled
        let k = s.user_by_name("ellie-beaumont").unwrap().unwrap();
        assert_eq!(k.id, keep.id);
        assert!(k.enabled);
        assert_eq!(k.display_name, "Ellie Beaumont");
        assert!(
            s.user_by_name("ellie-beaumont-study-diary")
                .unwrap()
                .is_none()
        );
        let r = s.user_by_id(retire.id).unwrap();
        assert_eq!(r.name, "retired-2-ellie-beaumont");
        assert!(!r.enabled);

        // ownership and grants unified under the kept id
        assert_eq!(s.app_by_id(diary.id).unwrap().owner_id, keep.id);
        assert_eq!(s.app_by_id(kitchen.id).unwrap().owner_id, keep.id);
        assert_eq!(
            s.app_by_id(kitchen.id).unwrap().updated_at,
            kitchen.updated_at + 10
        );
        assert!(s.has_grant(diary.id, keep.id).unwrap(), "grant moved");
        assert!(
            s.has_grant(shared.id, keep.id).unwrap(),
            "existing grant kept"
        );
        assert!(!s.has_grant(shared.id, retire.id).unwrap());
        assert!(!s.has_grant(diary.id, retire.id).unwrap());
        assert_eq!(s.list_grants(shared.id).unwrap().len(), 1);

        // the kept user's device and app sessions still work, the other's do not
        let (dsess, duser) = s
            .session_lookup(SessionKind::Auth, &dev_raw)
            .unwrap()
            .unwrap();
        assert_eq!(
            (dsess.id, duser.id, duser.name.as_str()),
            (dev.id, keep.id, "ellie-beaumont")
        );
        let (asess, _) = s
            .session_lookup(SessionKind::App, &app_raw)
            .unwrap()
            .unwrap();
        assert_eq!(asess.parent_id, Some(dev.id));
        assert!(
            s.session_lookup(SessionKind::Auth, &r_raw)
                .unwrap()
                .is_none()
        );

        // the retired identity's links are dead; the kept user's history is intact
        assert_eq!(
            s.peek_token(TokenKind::Enrol, &r_link).unwrap_err(),
            RedeemError::Revoked
        );
        assert_eq!(
            s.peek_token(TokenKind::Invite, &issued).unwrap_err(),
            RedeemError::Revoked
        );
        assert_eq!(
            s.peek_token(TokenKind::Enrol, &k_link).unwrap_err(),
            RedeemError::Used
        );

        // exactly one audit row, in the same transaction
        let audit = s.list_audit(50).unwrap();
        assert_eq!(audit.len(), before + 1);
        assert_eq!(audit[0].action, "user.consolidate");
        assert_eq!(audit[0].subject, "ellie-beaumont");
        assert_eq!(
            audit[0].detail,
            "absorbed=ellie-beaumont retired_as=retired-2-ellie-beaumont renamed_from=ellie-beaumont-study-diary apps=1 grants=1 sessions_revoked=1 tokens_revoked=2 cli"
        );
    }

    #[test]
    fn consolidate_refuses_unsafe_requests_and_leaves_no_trace() {
        let (s, _) = store_with_clock();
        let keep = s.create_user("keep", "Keep", Role::Member).unwrap();
        let retire = s.create_user("gone", "Gone", Role::Member).unwrap();
        let admin = s.create_user("boss", "Boss", Role::Admin).unwrap();
        let other = s.create_user("taken", "Taken", Role::Member).unwrap();
        let snapshot = |s: &Store| {
            let conn = s.lock();
            let mut st = conn
                .prepare("SELECT id, name, enabled FROM users ORDER BY id")
                .unwrap();
            st.query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect::<Vec<_>>()
        };
        let before = snapshot(&s);
        let audit_before = s.list_audit(50).unwrap().len();
        // same user
        assert!(matches!(
            s.consolidate_users(&keep, &keep, None, "cli").unwrap_err(),
            StoreError::Invalid(_)
        ));
        // retiring an admin implicitly
        assert!(matches!(
            s.consolidate_users(&keep, &admin, None, "cli").unwrap_err(),
            StoreError::Invalid(_)
        ));
        // kept user disabled
        s.set_user_enabled(keep.id, false).unwrap();
        let disabled = s.user_by_id(keep.id).unwrap();
        assert!(matches!(
            s.consolidate_users(&disabled, &retire, None, "cli")
                .unwrap_err(),
            StoreError::Invalid(_)
        ));
        s.set_user_enabled(keep.id, true).unwrap();
        // bad new name
        assert!(matches!(
            s.consolidate_users(&keep, &retire, Some("Bad Name"), "cli")
                .unwrap_err(),
            StoreError::Invalid(_)
        ));
        // new name taken by a third user: the transaction rolls back completely
        let err = s
            .consolidate_users(&keep, &retire, Some("taken"), "cli")
            .unwrap_err();
        assert!(matches!(err, StoreError::Conflict(_)), "{err}");
        assert_eq!(
            snapshot(&s),
            before,
            "rolled back: names and enabled flags untouched"
        );
        assert_eq!(s.user_by_id(other.id).unwrap().name, "taken");
        // stale snapshot (retire renamed underneath) is refused
        let stale = retire.clone();
        s.consolidate_users(&other, &retire, None, "cli").unwrap();
        assert!(matches!(
            s.consolidate_users(&keep, &stale, None, "cli").unwrap_err(),
            StoreError::Conflict(_) | StoreError::NotFound
        ));
        assert_eq!(
            s.list_audit(50).unwrap().len(),
            audit_before + 1,
            "only the successful call is audited"
        );
    }

    #[test]
    fn access_counters_dedupe_users_and_expire() {
        let (s, clock) = store_with_clock();
        let owner = s.create_user("owner", "Owner", Role::Member).unwrap();
        let bob = s.create_user("bob", "Bob", Role::Member).unwrap();
        let app = s
            .create_app(
                "demo",
                "Demo",
                "",
                owner.id,
                AppKind::Proxy,
                "127.0.0.1:3231",
                Visibility::Private,
            )
            .unwrap();
        let empty = s.app_analytics(app.id, 3).unwrap();
        assert_eq!(empty.total_requests, 0);
        assert_eq!(empty.since_day, None);
        assert_eq!(empty.recent.len(), 3);
        assert!(empty.recent.iter().all(|d| d.requests == 0 && d.users == 0));

        let day0 = day_of(s.now());
        s.record_access(app.id, Some(bob.id)).unwrap();
        s.record_access(app.id, Some(bob.id)).unwrap();
        s.record_access(app.id, Some(owner.id)).unwrap();
        s.record_access(app.id, None).unwrap();
        let a = s.app_analytics(app.id, 2).unwrap();
        assert_eq!(a.total_requests, 4);
        assert_eq!(a.since_day, Some(day0));
        assert_eq!(a.window_requests, 4);
        assert_eq!(a.window_users, 2, "bob counted once");
        assert_eq!(
            a.recent[0],
            DayAccess {
                day: day0,
                requests: 4,
                users: 2
            }
        );
        assert_eq!(a.recent[1].requests, 0);

        // Next day: bob again is a new unique for that day but still one user overall.
        clock.fetch_add(86400, Ordering::SeqCst);
        s.record_access(app.id, Some(bob.id)).unwrap();
        let a = s.app_analytics(app.id, 2).unwrap();
        assert_eq!(a.total_requests, 5);
        assert_eq!(a.window_users, 2);
        assert_eq!(a.recent[0].users, 1);
        assert_eq!(a.recent[1].users, 2);

        // Past the retention window the daily rows (and user ids) are gone,
        // while the all-time total survives.
        clock.fetch_add(86400 * ACCESS_RETENTION_DAYS, Ordering::SeqCst);
        s.record_access(app.id, None).unwrap();
        let a = s.app_analytics(app.id, 1).unwrap();
        assert_eq!(a.total_requests, 6);
        assert_eq!(a.window_requests, 1);
        assert_eq!(a.window_users, 0);
        let conn = s.lock();
        let users_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM access_daily_users", [], |r| r.get(0))
            .unwrap();
        assert_eq!(users_rows, 0, "identity rows pruned");
        let daily_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM access_daily", [], |r| r.get(0))
            .unwrap();
        assert_eq!(daily_rows, 1);
    }

    #[test]
    fn revoking_a_device_session_cascades_and_is_user_scoped() {
        let (s, _) = store_with_clock();
        let bob = s.create_user("bob", "Bob", Role::Member).unwrap();
        let eve = s.create_user("eve", "Eve", Role::Member).unwrap();
        let (dev_raw, dev) = s
            .create_session(SessionKind::Auth, bob.id, None, None, 100, "laptop")
            .unwrap();
        let (dev2_raw, _) = s
            .create_session(SessionKind::Auth, bob.id, None, None, 100, "phone")
            .unwrap();
        let (app_raw, app) = s
            .create_session(SessionKind::App, bob.id, None, Some(dev.id), 100, "laptop")
            .unwrap();
        let (app2_raw, _) = s
            .create_session(SessionKind::App, bob.id, None, None, 100, "phone")
            .unwrap();
        assert_eq!(
            s.session_by_id(app.id).unwrap().unwrap().parent_id,
            Some(dev.id)
        );
        // Another user cannot revoke bob's session, even with the right id.
        assert!(!s.revoke_session(dev.id, eve.id).unwrap());
        assert!(
            s.session_lookup(SessionKind::Auth, &dev_raw)
                .unwrap()
                .is_some()
        );
        // Revoking the device kills its child app session, nothing else.
        assert!(s.revoke_session(dev.id, bob.id).unwrap());
        assert!(
            s.session_lookup(SessionKind::Auth, &dev_raw)
                .unwrap()
                .is_none()
        );
        assert!(
            s.session_lookup(SessionKind::App, &app_raw)
                .unwrap()
                .is_none()
        );
        assert!(
            s.session_lookup(SessionKind::Auth, &dev2_raw)
                .unwrap()
                .is_some()
        );
        assert!(
            s.session_lookup(SessionKind::App, &app2_raw)
                .unwrap()
                .is_some()
        );
        assert!(
            !s.revoke_session(dev.id, bob.id).unwrap(),
            "already revoked"
        );
        assert_eq!(s.list_sessions(bob.id, SessionKind::Auth).unwrap().len(), 1);
        assert_eq!(s.list_sessions(bob.id, SessionKind::App).unwrap().len(), 1);
    }

    #[test]
    fn older_databases_gain_the_session_columns() {
        // A registry created before schema 3 has no parent_id / session_id.
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            &SCHEMA
                .replace(",\n    session_id INTEGER REFERENCES sessions(id)", "")
                .replace(",\n    parent_id INTEGER REFERENCES sessions(id)", "")
                .replace("'schema_version', '3'", "'schema_version', '2'"),
        )
        .unwrap();
        let cols = |t: &str| -> Vec<String> {
            conn.prepare(&format!("PRAGMA table_info({t})"))
                .unwrap()
                .query_map([], |r| r.get::<_, String>(1))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert!(!cols("sessions").contains(&"parent_id".to_string()));
        assert!(!cols("tokens").contains(&"session_id".to_string()));
        let s = Store::init(conn, system_clock()).unwrap();
        {
            let conn = s.lock();
            let has = |t: &str, c: &str| {
                conn.prepare(&format!("PRAGMA table_info({t})"))
                    .unwrap()
                    .query_map([], |r| r.get::<_, String>(1))
                    .unwrap()
                    .any(|r| r.unwrap() == c)
            };
            assert!(has("sessions", "parent_id") && has("tokens", "session_id"));
            let v: String = conn
                .query_row(
                    "SELECT value FROM meta WHERE key = 'schema_version'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(v, "3");
        }
        // And the upgraded registry works end to end.
        let u = s.create_user("bob", "Bob", Role::Member).unwrap();
        let app = s
            .create_app(
                "a",
                "A",
                "",
                u.id,
                AppKind::Proxy,
                "127.0.0.1:1",
                Visibility::Private,
            )
            .unwrap();
        let (_, dev) = s
            .create_session(SessionKind::Auth, u.id, None, None, 10, "")
            .unwrap();
        let (_, tok) = s.create_launch_code(u.id, app.id, dev.id, 10).unwrap();
        assert_eq!(tok.session_id, Some(dev.id));
        assert_eq!(tok.user_id, Some(u.id));
        assert_eq!(tok.kind, TokenKind::Launch);
    }

    #[test]
    fn access_tables_hold_only_counters() {
        let (s, _) = store_with_clock();
        let conn = s.lock();
        for table in ["access_daily", "access_daily_users", "access_totals"] {
            let mut stmt = conn
                .prepare(&format!("PRAGMA table_info({table})"))
                .unwrap();
            let cols: Vec<(String, String)> = stmt
                .query_map([], |r| Ok((r.get(1)?, r.get(2)?)))
                .unwrap()
                .map(|r| r.unwrap())
                .collect();
            for (name, ty) in &cols {
                assert_eq!(ty, "INTEGER", "{table}.{name} must be a counter or id");
                assert!(
                    ["app_id", "day", "requests", "user_id", "since_day"].contains(&name.as_str()),
                    "unexpected column {table}.{name}"
                );
            }
        }
    }
}
