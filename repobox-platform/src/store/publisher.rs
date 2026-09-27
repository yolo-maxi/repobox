//! Registry side of the external publisher: publisher principals, their
//! tokens and their releases. A publisher only ever reaches apps whose
//! `publisher_id` is its own; every query below is keyed on it.

use rusqlite::{OptionalExtension, Row, params};

use super::{APP_SELECT, Result, Store, StoreError, row_app, row_user};
use crate::model::{AiPolicy, App, AppKind, IdentityContract, Role, User, Visibility};
use crate::publisher::manifest::Manifest;
use crate::publisher::spool::JobResult;
use crate::publisher::{
    JOB_TIMEOUT_SECS, MAX_APPS_PER_PUBLISHER, MAX_RELEASES_PER_DAY, TOKEN_MAX_TTL, TOKEN_MIN_TTL,
    TOKEN_PREFIX,
};
use crate::tokens;

/// Target recorded for publisher apps: the real loopback port belongs to
/// the deploy worker (blue/green), so the registry never routes them.
pub const WORKER_MANAGED_TARGET: &str = "managed-by-deploy-worker";

#[derive(Debug, Clone)]
pub struct Publisher {
    pub id: i64,
    /// Immutable external id (`pub_<16 hex>`), returned on every record.
    pub public_id: String,
    pub handle: String,
    pub display_name: String,
    /// Canonical platform owner record of every app this publisher creates.
    pub owner_id: i64,
    pub enabled: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct PublisherToken {
    pub id: i64,
    pub name: String,
    pub publisher_id: i64,
    pub created_at: i64,
    pub expires_at: i64,
    pub revoked_at: Option<i64>,
    pub last_used_at: Option<i64>,
}

impl PublisherToken {
    pub fn status(&self, now: i64) -> &'static str {
        if self.revoked_at.is_some() {
            "revoked"
        } else if self.expires_at <= now {
            "expired"
        } else {
            "active"
        }
    }
}

#[derive(Debug, Clone)]
pub struct Release {
    pub id: String,
    pub publisher_id: i64,
    pub app_name: String,
    pub app_id: Option<i64>,
    pub version: i64,
    /// deploy | rollback | restart
    pub op: String,
    pub rollback_of: Option<String>,
    /// The manifest as deployed (JSON).
    pub manifest: String,
    /// queued | building | starting | live | superseded | failed | done
    pub status: String,
    pub failure_code: String,
    pub failure: String,
    pub commit: String,
    pub build_mode: String,
    pub image_id: String,
    pub retained: bool,
    pub token_name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub finished_at: Option<i64>,
    /// `sha256:<hex>` and size of the uploaded image archive.
    pub artifact_sha256: String,
    pub artifact_bytes: i64,
}

impl Release {
    pub fn in_progress(&self) -> bool {
        matches!(self.status.as_str(), "queued" | "building" | "starting")
    }
    pub fn manifest(&self) -> Option<Manifest> {
        serde_json::from_str(&self.manifest).ok()
    }
}

const PUBLISHER_SELECT: &str =
    "SELECT id, public_id, handle, display_name, owner_id, enabled, created_at FROM publishers";
const PTOKEN_SELECT: &str = "SELECT id, name, publisher_id, created_at, expires_at, revoked_at, last_used_at FROM publisher_tokens";
const RELEASE_SELECT: &str = "SELECT id, publisher_id, app_name, app_id, version, op, rollback_of, manifest, status, failure_code, failure, commit_sha, build_mode, image_id, retained, token_name, created_at, updated_at, finished_at, artifact_sha256, artifact_bytes FROM publisher_releases";

fn row_publisher(r: &Row<'_>) -> rusqlite::Result<Publisher> {
    Ok(Publisher {
        id: r.get(0)?,
        public_id: r.get(1)?,
        handle: r.get(2)?,
        display_name: r.get(3)?,
        owner_id: r.get(4)?,
        enabled: r.get::<_, i64>(5)? != 0,
        created_at: r.get(6)?,
    })
}

fn row_ptoken(r: &Row<'_>) -> rusqlite::Result<PublisherToken> {
    Ok(PublisherToken {
        id: r.get(0)?,
        name: r.get(1)?,
        publisher_id: r.get(2)?,
        created_at: r.get(3)?,
        expires_at: r.get(4)?,
        revoked_at: r.get(5)?,
        last_used_at: r.get(6)?,
    })
}

fn row_release(r: &Row<'_>) -> rusqlite::Result<Release> {
    Ok(Release {
        id: r.get(0)?,
        publisher_id: r.get(1)?,
        app_name: r.get(2)?,
        app_id: r.get(3)?,
        version: r.get(4)?,
        op: r.get(5)?,
        rollback_of: r.get(6)?,
        manifest: r.get(7)?,
        status: r.get(8)?,
        failure_code: r.get(9)?,
        failure: r.get(10)?,
        commit: r.get(11)?,
        build_mode: r.get(12)?,
        image_id: r.get(13)?,
        retained: r.get::<_, i64>(14)? != 0,
        token_name: r.get(15)?,
        created_at: r.get(16)?,
        updated_at: r.get(17)?,
        finished_at: r.get(18)?,
        artifact_sha256: r.get(19)?,
        artifact_bytes: r.get(20)?,
    })
}

fn conflict(code: &str, msg: impl Into<String>) -> StoreError {
    StoreError::Conflict(format!("{code}: {}", msg.into()))
}

/// The message used for every name collision, whoever owns the name.
pub const NAME_UNAVAILABLE: &str =
    "name_unavailable: that app name is not available; choose another";

impl Store {
    /// Create a publisher. Without `owner`, a dedicated member user named
    /// after the handle is created as the canonical owner record.
    pub fn create_publisher(
        &self,
        handle: &str,
        display_name: &str,
        owner: Option<&User>,
    ) -> Result<(Publisher, User)> {
        crate::model::validate_user_name(handle)
            .map_err(|e| StoreError::Invalid(format!("handle: {e}")))?;
        crate::model::validate_display_name(display_name).map_err(StoreError::Invalid)?;
        let owner = match owner {
            Some(u) => {
                if !u.enabled {
                    return Err(StoreError::Invalid(format!(
                        "user '{}' is disabled",
                        u.name
                    )));
                }
                u.clone()
            }
            None => {
                if self.user_by_name(handle)?.is_some() {
                    return Err(StoreError::Conflict(format!(
                        "a user named '{handle}' already exists; pass --owner to bind it explicitly"
                    )));
                }
                self.create_user(handle, display_name, Role::Member)?
            }
        };
        let public_id = {
            use rand::RngCore;
            let mut b = [0u8; 8];
            rand::rngs::OsRng.fill_bytes(&mut b);
            format!("pub_{:016x}", u64::from_be_bytes(b))
        };
        let now = self.now();
        let conn = self.lock();
        conn.execute(
            "INSERT INTO publishers (public_id, handle, display_name, owner_id, enabled, created_at) VALUES (?1, ?2, ?3, ?4, 1, ?5)",
            params![public_id, handle, display_name.trim(), owner.id, now],
        )
        .map_err(|e| match StoreError::from(e) {
            StoreError::Conflict(_) => StoreError::Conflict(format!("publisher '{handle}' already exists")),
            other => other,
        })?;
        let id = conn.last_insert_rowid();
        let p = conn.query_row(
            &format!("{PUBLISHER_SELECT} WHERE id = ?1"),
            params![id],
            row_publisher,
        )?;
        Ok((p, owner))
    }

    pub fn publisher_by_handle(&self, handle: &str) -> Result<Option<Publisher>> {
        let conn = self.lock();
        conn.query_row(
            &format!("{PUBLISHER_SELECT} WHERE handle = ?1"),
            params![handle],
            row_publisher,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn publisher_by_id(&self, id: i64) -> Result<Publisher> {
        let conn = self.lock();
        conn.query_row(
            &format!("{PUBLISHER_SELECT} WHERE id = ?1"),
            params![id],
            row_publisher,
        )
        .map_err(Into::into)
    }

    pub fn list_publishers(&self) -> Result<Vec<Publisher>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(&format!("{PUBLISHER_SELECT} ORDER BY id"))?;
        let rows = stmt.query_map([], row_publisher)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Disabling a publisher stops every one of its tokens at once; its apps
    /// keep running (disable them separately with `app disable`).
    pub fn set_publisher_enabled(&self, id: i64, enabled: bool) -> Result<()> {
        let conn = self.lock();
        let n = conn.execute(
            "UPDATE publishers SET enabled = ?2 WHERE id = ?1",
            params![id, enabled as i64],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    /// Issue a publisher token (`rbpub_…`). The raw value is returned once;
    /// only its hash is stored.
    pub fn create_publisher_token(
        &self,
        name: &str,
        publisher: &Publisher,
        ttl_secs: i64,
    ) -> Result<(String, PublisherToken)> {
        crate::model::validate_user_name(name)
            .map_err(|e| StoreError::Invalid(format!("token name: {e}")))?;
        if !publisher.enabled {
            return Err(StoreError::Invalid(format!(
                "publisher '{}' is disabled",
                publisher.handle
            )));
        }
        if !(TOKEN_MIN_TTL..=TOKEN_MAX_TTL).contains(&ttl_secs) {
            return Err(StoreError::Invalid(
                "publisher token lifetime must be 1 hour to 30 days".into(),
            ));
        }
        let raw = tokens::generate();
        let now = self.now();
        let conn = self.lock();
        conn.execute(
            "INSERT INTO publisher_tokens (name, token_hash, publisher_id, created_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, tokens::hash(&raw), publisher.id, now, now + ttl_secs],
        )
        .map_err(|e| match StoreError::from(e) {
            StoreError::Conflict(_) => StoreError::Conflict(format!("publisher token '{name}' already exists")),
            other => other,
        })?;
        let id = conn.last_insert_rowid();
        let tok = conn.query_row(
            &format!("{PTOKEN_SELECT} WHERE id = ?1"),
            params![id],
            row_ptoken,
        )?;
        Ok((format!("{TOKEN_PREFIX}{raw}"), tok))
    }

    /// Resolve a bearer value to a live publisher token, an enabled
    /// publisher and its enabled owner record.
    pub fn publisher_token_lookup(
        &self,
        bearer: &str,
    ) -> Result<Option<(PublisherToken, Publisher, User)>> {
        let Some(raw) = bearer.strip_prefix(TOKEN_PREFIX) else {
            return Ok(None);
        };
        if !tokens::looks_like_token(raw) {
            return Ok(None);
        }
        let now = self.now();
        let conn = self.lock();
        let Some(tok) = conn
            .query_row(
                &format!("{PTOKEN_SELECT} WHERE token_hash = ?1"),
                params![tokens::hash(raw)],
                row_ptoken,
            )
            .optional()?
        else {
            return Ok(None);
        };
        if tok.revoked_at.is_some() || tok.expires_at <= now {
            return Ok(None);
        }
        let p = conn.query_row(
            &format!("{PUBLISHER_SELECT} WHERE id = ?1"),
            params![tok.publisher_id],
            row_publisher,
        )?;
        if !p.enabled {
            return Ok(None);
        }
        let owner = conn.query_row(
            "SELECT id, name, display_name, role, enabled, created_at FROM users WHERE id = ?1",
            params![p.owner_id],
            row_user,
        )?;
        if !owner.enabled {
            return Ok(None);
        }
        if tok.last_used_at.is_none_or(|t| now - t > 60) {
            conn.execute(
                "UPDATE publisher_tokens SET last_used_at = ?2 WHERE id = ?1",
                params![tok.id, now],
            )?;
        }
        Ok(Some((tok, p, owner)))
    }

    pub fn list_publisher_tokens(&self) -> Result<Vec<PublisherToken>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(&format!("{PTOKEN_SELECT} ORDER BY id"))?;
        let rows = stmt.query_map([], row_ptoken)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub fn revoke_publisher_token(&self, name: &str) -> Result<bool> {
        let now = self.now();
        let conn = self.lock();
        let exists = conn
            .query_row(
                "SELECT 1 FROM publisher_tokens WHERE name = ?1",
                params![name],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if !exists {
            return Err(StoreError::NotFound);
        }
        let n = conn.execute(
            "UPDATE publisher_tokens SET revoked_at = ?2 WHERE name = ?1 AND revoked_at IS NULL",
            params![name, now],
        )?;
        Ok(n == 1)
    }

    /// Apps this publisher created.
    pub fn publisher_apps(&self, publisher_id: i64) -> Result<Vec<App>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(&format!(
            "{APP_SELECT} WHERE publisher_id = ?1 ORDER BY name"
        ))?;
        let rows = stmt.query_map(params![publisher_id], row_app)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// One app, only if this publisher created it.
    pub fn publisher_app(&self, publisher_id: i64, name: &str) -> Result<Option<App>> {
        let conn = self.lock();
        conn.query_row(
            &format!("{APP_SELECT} WHERE name = ?1 AND publisher_id = ?2"),
            params![name, publisher_id],
            row_app,
        )
        .optional()
        .map_err(Into::into)
    }

    /// Register (first deploy) or update (same publisher) an app and record
    /// a queued release, in one transaction. A name held by anyone else —
    /// another publisher, an operator-registered app — is refused with the
    /// same message.
    pub fn begin_deploy(
        &self,
        p: &Publisher,
        token_name: &str,
        m: &Manifest,
        release_id: &str,
        artifact: (&str, u64),
    ) -> Result<(App, Release)> {
        let manifest = serde_json::to_string(m).map_err(|e| StoreError::Invalid(e.to_string()))?;
        let now = self.now();
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        let existing = tx
            .query_row(
                &format!("{APP_SELECT} WHERE name = ?1"),
                params![m.name],
                row_app,
            )
            .optional()?;
        let app_id = match existing {
            Some(a) if a.publisher_id == Some(p.id) => {
                tx.execute(
                    "UPDATE apps SET title = ?2, description = ?3, updated_at = ?4 WHERE id = ?1",
                    params![a.id, m.title, m.description, now],
                )?;
                a.id
            }
            Some(_) => return Err(StoreError::Conflict(NAME_UNAVAILABLE.into())),
            None => {
                let count: i64 = tx.query_row(
                    "SELECT COUNT(*) FROM apps WHERE publisher_id = ?1",
                    params![p.id],
                    |r| r.get(0),
                )?;
                if count >= MAX_APPS_PER_PUBLISHER {
                    return Err(conflict(
                        "limit",
                        format!("a publisher may own at most {MAX_APPS_PER_PUBLISHER} apps"),
                    ));
                }
                let ai = if m.ai {
                    AiPolicy::registration_default(Visibility::Private, IdentityContract::Platform)
                } else {
                    AiPolicy::disabled()
                };
                tx.execute(
                    "INSERT INTO apps (name, title, description, owner_id, kind, target, visibility, enabled, created_at, updated_at, identity,
                                       ai_enabled, ai_provider, ai_default_model, ai_models, ai_max_input_chars, ai_max_output_tokens,
                                       ai_user_daily_requests, ai_app_daily_requests, ai_public_policy, publisher_id)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'private', 1, ?7, ?7, 'platform', ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                    params![
                        m.name, m.title, m.description, p.owner_id, AppKind::Proxy.as_str(), WORKER_MANAGED_TARGET, now,
                        ai.enabled as i64, ai.provider, ai.default_model, ai.models_csv(), ai.max_input_chars, ai.max_output_tokens,
                        ai.user_daily_requests, ai.app_daily_requests, ai.public_policy, p.id
                    ],
                )
                .map_err(|e| match StoreError::from(e) {
                    StoreError::Conflict(_) => StoreError::Conflict(NAME_UNAVAILABLE.into()),
                    other => other,
                })?;
                tx.last_insert_rowid()
            }
        };
        let release = insert_release(
            &tx, p, token_name, app_id, &m.name, "deploy", None, &manifest, release_id, now,
            artifact,
        )?;
        tx.commit()?;
        drop(conn);
        Ok((self.app_by_id(app_id)?, release))
    }

    /// Queue a rollback or restart of one of this publisher's apps.
    pub fn begin_app_op(
        &self,
        p: &Publisher,
        token_name: &str,
        app: &App,
        op: &str,
        rollback_of: Option<&Release>,
        release_id: &str,
    ) -> Result<Release> {
        if app.publisher_id != Some(p.id) {
            return Err(StoreError::NotFound);
        }
        // Read before taking the connection lock (it is not reentrant).
        let (manifest, artifact) = match rollback_of {
            Some(r) => (
                r.manifest.clone(),
                (r.artifact_sha256.clone(), r.artifact_bytes as u64),
            ),
            None => self
                .live_release(&app.name)?
                .map(|r| (r.manifest, (r.artifact_sha256, r.artifact_bytes as u64)))
                .ok_or_else(|| conflict("not_running", "the app has no live release"))?,
        };
        let now = self.now();
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        let r = insert_release(
            &tx,
            p,
            token_name,
            app.id,
            &app.name,
            op,
            rollback_of.map(|r| r.id.as_str()),
            &manifest,
            release_id,
            now,
            (&artifact.0, artifact.1),
        )?;
        tx.commit()?;
        Ok(r)
    }

    pub fn publisher_releases(
        &self,
        publisher_id: Option<i64>,
        app: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Release>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(&format!(
            "{RELEASE_SELECT} WHERE (?1 IS NULL OR publisher_id = ?1) AND (?2 IS NULL OR app_name = ?2) ORDER BY created_at DESC, id DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map(params![publisher_id, app, limit.clamp(1, 500)], row_release)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// One release; with `publisher_id`, only if that publisher made it.
    pub fn publisher_release(
        &self,
        publisher_id: Option<i64>,
        id: &str,
    ) -> Result<Option<Release>> {
        let conn = self.lock();
        conn.query_row(
            &format!("{RELEASE_SELECT} WHERE id = ?1 AND (?2 IS NULL OR publisher_id = ?2)"),
            params![id, publisher_id],
            row_release,
        )
        .optional()
        .map_err(Into::into)
    }

    /// The release currently serving an app, if any.
    pub fn live_release(&self, app: &str) -> Result<Option<Release>> {
        let conn = self.lock();
        conn.query_row(
            &format!("{RELEASE_SELECT} WHERE app_name = ?1 AND status = 'live' ORDER BY created_at DESC, id DESC LIMIT 1"),
            params![app],
            row_release,
        )
        .optional()
        .map_err(Into::into)
    }

    /// The in-progress release of an app, if any.
    pub fn release_in_progress(&self, app: &str) -> Result<Option<String>> {
        let conn = self.lock();
        conn.query_row(
            "SELECT id FROM publisher_releases WHERE app_name = ?1 AND status IN ('queued', 'building', 'starting')",
            params![app],
            |r| r.get(0),
        )
        .optional()
        .map_err(Into::into)
    }

    /// Releases the worker has not finished yet.
    pub fn releases_in_progress(&self) -> Result<Vec<Release>> {
        let conn = self.lock();
        let mut stmt = conn.prepare(&format!(
            "{RELEASE_SELECT} WHERE status IN ('queued', 'building', 'starting') ORDER BY created_at"
        ))?;
        let rows = stmt.query_map([], row_release)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Fold a worker progress/result file into the release record.
    /// Idempotent; a finished release is never changed again.
    pub fn apply_release_progress(&self, r: &JobResult) -> Result<()> {
        let now = self.now();
        let mut conn = self.lock();
        let tx = conn.transaction()?;
        let Some((op, app)): Option<(String, String)> = tx
            .query_row(
                "SELECT op, app_name FROM publisher_releases WHERE id = ?1 AND status IN ('queued', 'building', 'starting')",
                params![r.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
        else {
            return Ok(());
        };
        if r.app != app {
            return Ok(());
        }
        let status = match (r.state.as_str(), op.as_str()) {
            ("live", "restart") => "done",
            ("live", _) => "live",
            ("failed", _) => "failed",
            ("starting", _) => "starting",
            _ => "building",
        };
        let finished =
            matches!(status, "live" | "failed" | "done").then_some(r.finished_at.unwrap_or(now));
        tx.execute(
            "UPDATE publisher_releases SET status = ?2, failure_code = ?3, failure = ?4, commit_sha = CASE WHEN ?5 = '' THEN commit_sha ELSE ?5 END,
                 build_mode = CASE WHEN ?6 = '' THEN build_mode ELSE ?6 END, image_id = CASE WHEN ?7 = '' THEN image_id ELSE ?7 END,
                 updated_at = ?8, finished_at = ?9
             WHERE id = ?1",
            params![r.id, status, r.code, r.failure, r.commit, r.build_mode, r.image_id, now, finished],
        )?;
        if status == "live" {
            tx.execute(
                "UPDATE publisher_releases SET status = 'superseded', updated_at = ?3 WHERE app_name = ?1 AND status = 'live' AND id != ?2",
                params![app, r.id, now],
            )?;
        }
        if finished.is_some() {
            tx.execute(
                "UPDATE publisher_releases SET retained = 0 WHERE app_name = ?1",
                params![app],
            )?;
            for id in &r.retained {
                tx.execute(
                    "UPDATE publisher_releases SET retained = 1 WHERE app_name = ?1 AND id = ?2",
                    params![app, id],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Give up on releases the worker never answered.
    pub fn fail_stale_releases(&self) -> Result<usize> {
        let now = self.now();
        let conn = self.lock();
        conn.execute(
            "UPDATE publisher_releases SET status = 'failed', failure_code = 'timeout',
                 failure = 'the deploy worker did not finish this release in time; deploy again or ask an operator',
                 updated_at = ?1, finished_at = ?1
             WHERE status IN ('queued', 'building', 'starting') AND created_at < ?2",
            params![now, now - JOB_TIMEOUT_SECS],
        )
        .map_err(Into::into)
    }
}

#[allow(clippy::too_many_arguments)]
fn insert_release(
    tx: &rusqlite::Transaction<'_>,
    p: &Publisher,
    token_name: &str,
    app_id: i64,
    app: &str,
    op: &str,
    rollback_of: Option<&str>,
    manifest: &str,
    release_id: &str,
    now: i64,
    artifact: (&str, u64),
) -> Result<Release> {
    if let Some(busy) = tx
        .query_row(
            "SELECT id FROM publisher_releases WHERE app_name = ?1 AND status IN ('queued', 'building', 'starting')",
            params![app],
            |r| r.get::<_, String>(0),
        )
        .optional()?
    {
        return Err(conflict(
            "release_in_progress",
            format!("release {busy} of this app is still in progress; wait for it to finish"),
        ));
    }
    let today: i64 = tx.query_row(
        "SELECT COUNT(*) FROM publisher_releases WHERE publisher_id = ?1 AND created_at >= ?2",
        params![p.id, now - 86400],
        |r| r.get(0),
    )?;
    if today >= MAX_RELEASES_PER_DAY {
        return Err(conflict(
            "limit",
            format!("at most {MAX_RELEASES_PER_DAY} releases per publisher per 24 hours"),
        ));
    }
    let version: i64 = tx.query_row(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM publisher_releases WHERE app_name = ?1",
        params![app],
        |r| r.get(0),
    )?;
    tx.execute(
        "INSERT INTO publisher_releases (id, publisher_id, app_name, app_id, version, op, rollback_of, manifest, status, token_name, created_at, updated_at, artifact_sha256, artifact_bytes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'queued', ?9, ?10, ?10, ?11, ?12)",
        params![release_id, p.id, app, app_id, version, op, rollback_of, manifest, token_name, now, artifact.0, artifact.1 as i64],
    )?;
    tx.query_row(
        &format!("{RELEASE_SELECT} WHERE id = ?1"),
        params![release_id],
        row_release,
    )
    .map_err(Into::into)
}
