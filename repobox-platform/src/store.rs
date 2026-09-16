//! SQLite-backed registry. One connection behind a mutex is plenty for a
//! control plane whose hot path is one indexed lookup per gated request.
//!
//! Invariants enforced here rather than in handlers:
//! * raw tokens and session secrets are never written, only their SHA-256;
//! * single-use tokens are consumed with a conditional UPDATE so a replay
//!   races cannot redeem twice;
//! * access checks always re-read user/app state so revocation, disabling and
//!   visibility changes take effect on the next request.

use std::path::Path;
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
    note TEXT NOT NULL DEFAULT ''
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
    label TEXT NOT NULL DEFAULT ''
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
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
INSERT OR IGNORE INTO meta (key, value) VALUES ('schema_version', '1');
"#;

pub struct Store {
    conn: Mutex<Connection>,
    clock: Clock,
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
        Ok(Self {
            conn: Mutex::new(conn),
            clock,
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

    pub fn create_session(
        &self,
        kind: SessionKind,
        user_id: i64,
        app_id: Option<i64>,
        ttl_secs: i64,
        label: &str,
    ) -> Result<(String, Session)> {
        let raw = tokens::generate();
        let hash = tokens::hash(&raw);
        let now = self.now();
        let conn = self.lock();
        conn.execute(
            "INSERT INTO sessions (kind, token_hash, user_id, app_id, created_at, expires_at, last_seen_at, label)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?5, ?7)",
            params![kind.as_str(), hash, user_id, app_id, now, now + ttl_secs, label],
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

    pub fn revoke_session(&self, id: i64, user_id: i64) -> Result<bool> {
        let now = self.now();
        let conn = self.lock();
        let n = conn.execute(
            "UPDATE sessions SET revoked_at = ?3 WHERE id = ?1 AND user_id = ?2 AND revoked_at IS NULL",
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
const TOKEN_SELECT: &str = "SELECT id, kind, user_id, app_id, created_by, created_at, expires_at, used_at, used_by, revoked_at, note FROM tokens";
const SESSION_SELECT: &str = "SELECT id, kind, user_id, app_id, created_at, expires_at, last_seen_at, revoked_at, label FROM sessions";

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
            .create_session(SessionKind::Auth, u.id, None, 60, "")
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
            .create_session(SessionKind::App, u.id, None, 100, "")
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
            .create_session(SessionKind::App, u.id, None, 100, "")
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
}
