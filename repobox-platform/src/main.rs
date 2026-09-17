//! `repobox-platform`: the repo.box platform control plane.
//!
//! * `serve`        — the auth.repo.box UI plus the loopback gate for Caddy.
//! * `demo-origin`  — the loopback origin behind the private demo app.
//! * operator commands (users, apps, grants, route rendering, backups). These
//!   are the only way to register an app or produce a route: the web UI has no
//!   deploy or route controls by design.
//!
//! Secrets policy: the only time a raw token leaves this program is when an
//! operator command writes a single-use link to a 0600 file it was asked to
//! create. Nothing is ever printed to stdout or logged.

use repobox_platform::{demo_origin, model, render, store, web};

use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::{Parser, Subcommand};

use model::{AppKind, IdentityContract, Role, Visibility};
use store::{Store, TokenKind};

const DEFAULT_DB: &str = "/var/lib/repobox-platform/platform.db";

#[derive(Parser)]
#[command(
    name = "repobox-platform",
    version,
    about = "repo.box platform control plane"
)]
struct Cli {
    /// SQLite database path
    #[arg(long, global = true, env = "REPOBOX_PLATFORM_DB", default_value = DEFAULT_DB)]
    db: PathBuf,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the control plane (UI + gate) on a loopback address
    Serve {
        #[arg(long, default_value = "127.0.0.1:3230")]
        bind: SocketAddr,
        /// Public origin of this service
        #[arg(
            long,
            env = "REPOBOX_PLATFORM_PUBLIC_BASE",
            default_value = "https://auth.repo.box"
        )]
        public_base: String,
        /// Apex domain managed apps hang off
        #[arg(long, env = "REPOBOX_PLATFORM_DOMAIN", default_value = "repo.box")]
        domain: String,
    },
    /// Run the private demo app origin (loopback only)
    DemoOrigin {
        #[arg(long, default_value = "127.0.0.1:3231")]
        bind: SocketAddr,
        #[arg(long, default_value = "demo-private")]
        app: String,
        #[arg(long, default_value = "Private demo")]
        title: String,
    },
    /// Create the first admin (if none exists) and write a single-use device
    /// link for them to a 0600 file. Never prints the link.
    BootstrapAdmin {
        #[arg(long)]
        name: String,
        #[arg(long)]
        display_name: Option<String>,
        /// File to write the enrolment link to (created 0600)
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 7 * 24)]
        ttl_hours: i64,
    },
    /// Manage users
    User {
        #[command(subcommand)]
        cmd: UserCmd,
    },
    /// Manage apps and grants
    App {
        #[command(subcommand)]
        cmd: AppCmd,
    },
    /// Render or check the managed Caddy routes
    Routes {
        #[command(subcommand)]
        cmd: RoutesCmd,
    },
    /// Online backup of the database to a file
    Backup {
        #[arg(long)]
        out: PathBuf,
    },
    /// Print the most recent audit entries
    Audit {
        #[arg(long, default_value_t = 50)]
        limit: i64,
    },
}

#[derive(Subcommand)]
enum UserCmd {
    Create {
        name: String,
        #[arg(long)]
        display_name: Option<String>,
        #[arg(long)]
        admin: bool,
    },
    List,
    Enable {
        name: String,
    },
    Disable {
        name: String,
    },
    /// Write a single-use device link for a user to a 0600 file
    Enrol {
        name: String,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 7 * 24)]
        ttl_hours: i64,
    },
    /// Revoke every signed-in device and app session of a user
    Logout {
        name: String,
    },
    /// List a user's live device and app sessions (no secrets)
    Sessions {
        name: String,
    },
    /// Fold one user into another. The kept user keeps its id and therefore
    /// every device/app session; it takes over the retired user's apps and
    /// grants and may be renamed. The retired user is disabled, renamed
    /// `retired-<id>-<name>`, and all of its sessions and unused links are
    /// revoked. One transaction, one audit row.
    Consolidate {
        /// User to keep (must be enabled)
        #[arg(long)]
        keep: String,
        /// User to retire into it (must not be an admin)
        #[arg(long)]
        retire: String,
        /// New name for the kept user (defaults to its current name)
        #[arg(long)]
        name: Option<String>,
    },
}

#[derive(Subcommand)]
enum AppCmd {
    /// Register an app (or update its route fields with --replace-route)
    Register {
        name: String,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "")]
        description: String,
        #[arg(long)]
        owner: String,
        #[arg(long, value_parser = ["static", "proxy"])]
        kind: String,
        /// static: absolute root directory; proxy: 127.0.0.1:PORT
        #[arg(long)]
        target: String,
        #[arg(long, default_value = "private", value_parser = ["private", "public_unlisted", "public_listed"])]
        visibility: String,
        /// Platform identity contract. Required for private apps: the app has
        /// no password/login/setup link/session of its own and scopes records
        /// by the identity the edge injects (X-RepoBox-*). The only value is
        /// `platform`; registering a private app without it is refused.
        #[arg(long, value_parser = ["platform"])]
        identity: Option<String>,
        /// If the app exists, update title/description/kind/target only
        #[arg(long)]
        replace_route: bool,
    },
    /// Attest, after review, that an app uses only the gate-injected identity
    /// (no app-level password, login, setup link or session). One-way, audited.
    Attest {
        name: String,
        /// What was reviewed/removed (recorded in the audit row)
        #[arg(long, default_value = "")]
        note: String,
    },
    List {
        #[arg(long)]
        json: bool,
    },
    Show {
        name: String,
    },
    Grant {
        name: String,
        #[arg(long)]
        user: String,
    },
    Revoke {
        name: String,
        #[arg(long)]
        user: String,
    },
    Visibility {
        name: String,
        #[arg(value_parser = ["private", "public_unlisted", "public_listed"])]
        visibility: String,
    },
    Enable {
        name: String,
    },
    Disable {
        name: String,
    },
    /// Move an app to another (enabled) user. Route, visibility, enabled
    /// state and grants are kept; the transition is audited.
    TransferOwner {
        name: String,
        #[arg(long)]
        owner: String,
    },
    /// Remove an app from the registry (re-render routes afterwards)
    Remove {
        name: String,
    },
    /// Print the access counters of an app (allowed requests only, no identity)
    Stats {
        name: String,
        #[arg(long, default_value_t = 14)]
        days: i64,
    },
    /// Print opens by signed-in people (first page load per 30-minute visit)
    Visits {
        name: String,
        /// Range in UTC days, today included (1-90)
        #[arg(long, default_value_t = 30)]
        days: i64,
    },
}

#[derive(Subcommand)]
enum RoutesCmd {
    /// Render the Caddy snippet for every registered app
    Render {
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value = "repo.box")]
        domain: String,
        #[arg(long, default_value = "127.0.0.1:3230")]
        gate: String,
        /// Directory static roots must live under; repeat to allow several
        #[arg(long = "apps-root", default_value = "/srv/repobox-platform/apps")]
        apps_roots: Vec<String>,
        /// Fail if a static root directory does not exist on this machine
        #[arg(long)]
        check_roots: bool,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Serve {
            bind,
            public_base,
            domain,
        } => {
            init_tracing();
            if !bind.ip().is_loopback() {
                return Err(format!("serve must bind to loopback, got {bind}").into());
            }
            let store = Store::open(&cli.db)?;
            let state = Arc::new(web::AppState {
                store,
                cfg: web::Config::defaults(&public_base, &domain),
            });
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async move {
                let app = web::router(state);
                let listener = tokio::net::TcpListener::bind(bind).await?;
                tracing::info!(
                    "control plane listening on {} (db {})",
                    listener.local_addr()?,
                    cli.db.display()
                );
                axum::serve(listener, app).await?;
                Ok::<(), Box<dyn std::error::Error>>(())
            })
        }
        Cmd::DemoOrigin { bind, app, title } => {
            init_tracing();
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(demo_origin::serve(bind, app, title))
        }
        Cmd::BootstrapAdmin {
            name,
            display_name,
            out,
            ttl_hours,
        } => {
            let store = Store::open(&cli.db)?;
            let user = match store.user_by_name(&name)? {
                Some(u) if u.is_admin() => u,
                Some(u) => {
                    store.set_user_role(u.id, Role::Admin)?;
                    store.audit(None, "user.promote", &u.name, "bootstrap");
                    store.user_by_id(u.id)?
                }
                None => {
                    if store.count_admins()? > 0 {
                        return Err(
                            "an admin already exists; use `user create` / `user enrol` instead"
                                .into(),
                        );
                    }
                    let u = store.create_user(
                        &name,
                        display_name.as_deref().unwrap_or(&name),
                        Role::Admin,
                    )?;
                    store.audit(None, "user.create", &u.name, "bootstrap admin");
                    u
                }
            };
            if !user.enabled {
                return Err(format!("user '{}' is disabled", user.name).into());
            }
            write_link(&store, &user, &out, ttl_hours, "bootstrap")?;
            println!(
                "admin '{}' ready; single-use device link written to {} (expires in {}h)",
                user.name,
                out.display(),
                ttl_hours
            );
            Ok(())
        }
        Cmd::User { cmd } => user_cmd(&Store::open(&cli.db)?, cmd),
        Cmd::App { cmd } => app_cmd(&Store::open(&cli.db)?, cmd),
        Cmd::Routes { cmd } => routes_cmd(&Store::open(&cli.db)?, cmd),
        Cmd::Backup { out } => {
            let store = Store::open(&cli.db)?;
            if out.exists() {
                return Err(format!(
                    "{} already exists; refusing to overwrite a backup",
                    out.display()
                )
                .into());
            }
            store.backup_to(&out)?;
            let _ =
                std::fs::set_permissions(&out, std::os::unix::fs::PermissionsExt::from_mode(0o600));
            println!("backup written to {}", out.display());
            Ok(())
        }
        Cmd::Audit { limit } => {
            let store = Store::open(&cli.db)?;
            for e in store.list_audit(limit)? {
                println!(
                    "{}  {:<12} {:<16} {:<20} {}",
                    web::html::fmt_ts(e.at),
                    e.actor.as_deref().unwrap_or("-"),
                    e.action,
                    e.subject,
                    e.detail
                );
            }
            Ok(())
        }
    }
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

fn write_link(
    store: &Store,
    user: &model::User,
    out: &Path,
    ttl_hours: i64,
    note: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::OpenOptionsExt;
    if out.exists() {
        return Err(format!("{} already exists; choose a new file so a stale link is never confused for a fresh one", out.display()).into());
    }
    let (raw, tok) = store.create_token(
        TokenKind::Enrol,
        Some(user.id),
        None,
        None,
        ttl_hours * 3600,
        note,
    )?;
    let public_base = std::env::var("REPOBOX_PLATFORM_PUBLIC_BASE")
        .unwrap_or_else(|_| "https://auth.repo.box".into());
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(out)?;
    writeln!(f, "{}/enrol/{}", public_base.trim_end_matches('/'), raw)?;
    writeln!(
        f,
        "# single-use device link for '{}'; expires {}",
        user.name,
        web::html::fmt_ts(tok.expires_at)
    )?;
    store.audit(None, "enrol.create", &user.name, "cli");
    Ok(())
}

fn user_cmd(store: &Store, cmd: UserCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        UserCmd::Create {
            name,
            display_name,
            admin,
        } => {
            let role = if admin { Role::Admin } else { Role::Member };
            let u = store.create_user(&name, display_name.as_deref().unwrap_or(&name), role)?;
            store.audit(None, "user.create", &u.name, "cli");
            println!("created user '{}' ({})", u.name, u.role.as_str());
        }
        UserCmd::List => {
            println!("NAME                 ROLE     STATUS    CREATED            DISPLAY");
            for u in store.list_users()? {
                println!(
                    "{:<20} {:<8} {:<9} {:<18} {}",
                    u.name,
                    u.role.as_str(),
                    if u.enabled { "active" } else { "disabled" },
                    web::html::fmt_ts(u.created_at),
                    u.display_name
                );
            }
        }
        UserCmd::Enable { name } => {
            let u = need_user(store, &name)?;
            store.set_user_enabled(u.id, true)?;
            store.audit(None, "user.enable", &u.name, "cli");
            println!("enabled '{}'", u.name);
        }
        UserCmd::Disable { name } => {
            let u = need_user(store, &name)?;
            store.set_user_enabled(u.id, false)?;
            let n = store.revoke_user_sessions(u.id)?;
            store.audit(None, "user.disable", &u.name, "cli");
            println!("disabled '{}' and revoked {n} session(s)", u.name);
        }
        UserCmd::Enrol {
            name,
            out,
            ttl_hours,
        } => {
            let u = need_user(store, &name)?;
            if !u.enabled {
                return Err(format!("user '{}' is disabled", u.name).into());
            }
            write_link(store, &u, &out, ttl_hours, "cli")?;
            println!(
                "single-use device link for '{}' written to {} (expires in {}h)",
                u.name,
                out.display(),
                ttl_hours
            );
        }
        UserCmd::Logout { name } => {
            let u = need_user(store, &name)?;
            let n = store.revoke_user_sessions(u.id)?;
            store.audit(None, "session.revoke_all", &u.name, "cli");
            println!("revoked {n} session(s) for '{}'", u.name);
        }
        UserCmd::Sessions { name } => {
            let u = need_user(store, &name)?;
            println!(
                "user {} (id {}, {}, {})",
                u.name,
                u.id,
                u.role.as_str(),
                if u.enabled { "active" } else { "disabled" }
            );
            println!(
                "ID     KIND  APP                    CREATED             EXPIRES             LAST SEEN           DEVICE  LABEL"
            );
            for kind in [store::SessionKind::Auth, store::SessionKind::App] {
                for sess in store.list_sessions(u.id, kind)? {
                    let app = match sess.app_id {
                        Some(id) => store
                            .app_by_id(id)
                            .map(|a| a.name)
                            .unwrap_or_else(|_| "?".into()),
                        None => "-".into(),
                    };
                    println!(
                        "{:<6} {:<5} {:<22} {:<19} {:<19} {:<19} {:<7} {}",
                        sess.id,
                        kind.as_str(),
                        app,
                        web::html::fmt_ts(sess.created_at),
                        web::html::fmt_ts(sess.expires_at),
                        web::html::fmt_ts(sess.last_seen_at),
                        sess.parent_id
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| "-".into()),
                        sess.label
                    );
                }
            }
        }
        UserCmd::Consolidate { keep, retire, name } => {
            let k = need_user(store, &keep)?;
            let r = need_user(store, &retire)?;
            let c = store.consolidate_users(&k, &r, name.as_deref(), "cli")?;
            println!(
                "consolidated '{}' into '{}' (id {}): now named '{}'; {} app(s) and {} grant(s) moved; retired user renamed '{}' and disabled, {} session(s) and {} unused link(s) revoked",
                r.name,
                k.name,
                k.id,
                c.kept_name,
                c.apps,
                c.grants,
                c.retired_name,
                c.sessions_revoked,
                c.tokens_revoked
            );
        }
    }
    Ok(())
}

fn need_user(store: &Store, name: &str) -> Result<model::User, Box<dyn std::error::Error>> {
    store
        .user_by_name(name)?
        .ok_or_else(|| format!("no user '{name}'").into())
}

fn need_app(store: &Store, name: &str) -> Result<model::App, Box<dyn std::error::Error>> {
    store
        .app_by_name(name)?
        .ok_or_else(|| format!("no app '{name}'").into())
}

fn app_cmd(store: &Store, cmd: AppCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        AppCmd::Register {
            name,
            title,
            description,
            owner,
            kind,
            target,
            visibility,
            identity,
            replace_route,
        } => {
            let kind = AppKind::parse(&kind).ok_or("bad kind")?;
            let vis = Visibility::parse(&visibility).ok_or("bad visibility")?;
            let identity = match identity.as_deref() {
                Some("platform") => IdentityContract::Platform,
                _ => IdentityContract::Pending,
            };
            if vis == Visibility::Private && !identity.is_platform() {
                return Err(store::PRIVATE_NEEDS_PLATFORM_IDENTITY.into());
            }
            if let Some(existing) = store.app_by_name(&name)? {
                if !replace_route {
                    return Err(format!(
                        "app '{}' already exists (use --replace-route to update its route fields)",
                        existing.name
                    )
                    .into());
                }
                let a = store.update_app_route(&name, &title, &description, kind, &target)?;
                store.audit(
                    None,
                    "app.update_route",
                    &a.name,
                    &format!("{} {}", a.kind.as_str(), a.target),
                );
                println!(
                    "updated route of '{}' ({} -> {}); visibility/enabled/owner untouched",
                    a.name,
                    a.kind.as_str(),
                    a.target
                );
            } else {
                let o = need_user(store, &owner)?;
                let a = store.create_app(
                    &name,
                    &title,
                    &description,
                    o.id,
                    kind,
                    &target,
                    vis,
                    identity,
                )?;
                store.audit(
                    None,
                    "app.register",
                    &a.name,
                    &format!(
                        "{} {} owner={} identity={}",
                        a.kind.as_str(),
                        a.target,
                        o.name,
                        a.identity.as_str()
                    ),
                );
                println!(
                    "registered '{}' ({} -> {}), owner {}, visibility {}",
                    a.name,
                    a.kind.as_str(),
                    a.target,
                    o.name,
                    a.visibility.as_str()
                );
            }
            println!("next: `repobox-platform routes render --out <snippet>` and reload Caddy");
        }
        AppCmd::List { json } => {
            let apps = store.list_apps()?;
            if json {
                let v: Vec<serde_json::Value> = apps
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "name": a.name, "title": a.title, "kind": a.kind.as_str(), "target": a.target,
                            "visibility": a.visibility.as_str(), "enabled": a.enabled, "owner_id": a.owner_id,
                            "identity": a.identity.as_str(),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                println!(
                    "NAME                   KIND    TARGET                               VISIBILITY       STATE    IDENTITY  TITLE"
                );
                for a in &apps {
                    println!(
                        "{:<22} {:<7} {:<36} {:<16} {:<8} {:<9} {}",
                        a.name,
                        a.kind.as_str(),
                        a.target,
                        a.visibility.as_str(),
                        if a.enabled { "on" } else { "off" },
                        a.identity.as_str(),
                        a.title
                    );
                }
                let pending = render::pending_private(&apps);
                if !pending.is_empty() {
                    eprintln!(
                        "WARNING: {} private app(s) without the platform identity contract (app-level login must be removed, then `app attest`): {}",
                        pending.len(),
                        pending
                            .iter()
                            .map(|a| a.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
        }
        AppCmd::Show { name } => {
            let a = need_app(store, &name)?;
            let owner = store.user_by_id(a.owner_id)?;
            println!("name:        {}", a.name);
            println!("title:       {}", a.title);
            println!("description: {}", a.description);
            println!("route:       {} -> {}", a.kind.as_str(), a.target);
            println!("owner:       {} ({})", owner.name, owner.display_name);
            println!("visibility:  {}", a.visibility.as_str());
            println!("enabled:     {}", a.enabled);
            println!(
                "identity:    {} — {}",
                a.identity.as_str(),
                a.identity.help()
            );
            let grants = store.list_grants(a.id)?;
            println!("grants:      {}", grants.len());
            for g in grants {
                println!(
                    "  - {} (since {})",
                    g.user.name,
                    web::html::fmt_ts(g.created_at)
                );
            }
        }
        AppCmd::Grant { name, user } => {
            let a = need_app(store, &name)?;
            let u = need_user(store, &user)?;
            let added = store.add_grant(a.id, u.id, None)?;
            store.audit(None, "grant.add", &a.name, &format!("{} cli", u.name));
            println!(
                "{} '{}' -> '{}'",
                if added { "granted" } else { "already granted" },
                u.name,
                a.name
            );
        }
        AppCmd::Revoke { name, user } => {
            let a = need_app(store, &name)?;
            let u = need_user(store, &user)?;
            let removed = store.remove_grant(a.id, u.id)?;
            store.audit(None, "grant.remove", &a.name, &format!("{} cli", u.name));
            println!(
                "{} '{}' on '{}'",
                if removed { "revoked" } else { "no grant for" },
                u.name,
                a.name
            );
        }
        AppCmd::Attest { name, note } => {
            let a = need_app(store, &name)?;
            let changed = store.attest_app(a.id)?;
            if changed {
                store.audit(None, "app.attest", &a.name, &format!("cli {note}"));
                println!(
                    "'{}' identity contract -> platform (no app login; gate-injected identity only)",
                    a.name
                );
            } else {
                println!(
                    "'{}' already carries the platform identity contract",
                    a.name
                );
            }
        }
        AppCmd::Visibility { name, visibility } => {
            let a = need_app(store, &name)?;
            let v = Visibility::parse(&visibility).ok_or("bad visibility")?;
            store.set_app_visibility(a.id, v)?;
            store.audit(
                None,
                "app.visibility",
                &a.name,
                &format!("{} cli", v.as_str()),
            );
            println!("'{}' visibility -> {}", a.name, v.as_str());
        }
        AppCmd::Enable { name } => {
            let a = need_app(store, &name)?;
            store.set_app_enabled(a.id, true)?;
            store.audit(None, "app.enable", &a.name, "cli");
            println!("'{}' enabled", a.name);
        }
        AppCmd::Disable { name } => {
            let a = need_app(store, &name)?;
            store.set_app_enabled(a.id, false)?;
            store.audit(None, "app.disable", &a.name, "cli");
            println!("'{}' disabled", a.name);
        }
        AppCmd::TransferOwner { name, owner } => {
            let a = need_app(store, &name)?;
            let from = store.user_by_id(a.owner_id)?;
            let to = need_user(store, &owner)?;
            if store.transfer_app_owner(&a, &to, "cli")? {
                println!(
                    "'{}' owner: {} -> {}; route, visibility, enabled state and grants unchanged",
                    a.name, from.name, to.name
                );
            } else {
                println!(
                    "'{}' is already owned by {}; nothing changed",
                    a.name, to.name
                );
            }
        }
        AppCmd::Remove { name } => {
            let a = need_app(store, &name)?;
            store.delete_app(&a.name)?;
            store.audit(None, "app.remove", &a.name, "cli");
            println!("removed '{}'; re-render routes and reload Caddy", a.name);
        }
        AppCmd::Stats { name, days } => {
            let a = need_app(store, &name)?;
            let st = store.app_analytics(a.id, days)?;
            let private = a.visibility == Visibility::Private;
            println!(
                "{}: {} allowed request(s) {}",
                a.name,
                st.total_requests,
                st.since_day
                    .map(|d| format!("since {}", web::html::fmt_day(d)))
                    .unwrap_or_else(|| "(nothing counted yet)".into())
            );
            println!(
                "last {} days: {} request(s), {} signed-in user(s){}",
                st.window_days,
                st.window_requests,
                if private {
                    st.window_users.to_string()
                } else {
                    "-".into()
                },
                if private {
                    ""
                } else {
                    " (public app: visitors are not identified)"
                }
            );
            println!("DAY         REQUESTS  USERS");
            for d in &st.recent {
                println!(
                    "{}  {:>8}  {}",
                    web::html::fmt_day(d.day),
                    d.requests,
                    if private {
                        d.users.to_string()
                    } else {
                        "-".into()
                    }
                );
            }
        }
        AppCmd::Visits { name, days } => {
            let a = need_app(store, &name)?;
            let v = store.app_visits(a.id, days)?;
            println!(
                "{}: {} open(s) by {} signed-in {} in the last {} day(s) (since {} UTC)",
                a.name,
                v.opens,
                v.people,
                if v.people == 1 { "person" } else { "people" },
                v.days,
                web::html::fmt_day(store::day_of(v.since))
            );
            println!(
                "an open is the first HTML page load of a visit; a visit ends after {} minutes without a page load",
                store::VISIT_WINDOW_SECS / 60
            );
            println!("DAY         OPENS  PEOPLE");
            for d in v.daily.iter().filter(|d| d.opens > 0) {
                println!(
                    "{}  {:>5}  {}",
                    web::html::fmt_day(d.day),
                    d.opens,
                    d.people
                );
            }
            if v.by_person.is_empty() {
                println!("(no opens in range)");
            } else {
                println!(
                    "HANDLE                            OPENS  LAST OPENED        DISPLAY NAME"
                );
                for p in &v.by_person {
                    println!(
                        "{:<32}  {:>5}  {}  {}{}",
                        p.name,
                        p.opens,
                        web::html::fmt_ts(p.last_opened_at),
                        p.display_name,
                        if p.enabled { "" } else { " (disabled)" }
                    );
                }
            }
        }
    }
    Ok(())
}

fn routes_cmd(store: &Store, cmd: RoutesCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        RoutesCmd::Render {
            out,
            domain,
            gate,
            apps_roots,
            check_roots,
        } => {
            let apps = store.list_apps()?;
            let cfg = render::RenderConfig {
                domain,
                gate,
                apps_roots,
            };
            if check_roots {
                for a in apps.iter().filter(|a| a.kind == AppKind::Static) {
                    if !Path::new(&a.target).is_dir() {
                        return Err(format!(
                            "static root {} for app '{}' is not a directory",
                            a.target, a.name
                        )
                        .into());
                    }
                }
            }
            let text = render::render(&apps, &cfg)?;
            for a in render::pending_private(&apps) {
                eprintln!(
                    "WARNING: private app '{}' is served without the platform identity contract (registered before the policy); remove its app-level login, then `app attest {}`",
                    a.name, a.name
                );
            }
            match out {
                Some(p) => {
                    let tmp = p.with_extension("tmp");
                    std::fs::write(&tmp, &text)?;
                    std::fs::rename(&tmp, &p)?;
                    eprintln!("rendered {} app route(s) to {}", apps.len(), p.display());
                }
                None => print!("{text}"),
            }
        }
    }
    Ok(())
}
