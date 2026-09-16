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

use model::{AppKind, Role, Visibility};
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
        /// If the app exists, update title/description/kind/target only
        #[arg(long)]
        replace_route: bool,
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
    /// Remove an app from the registry (re-render routes afterwards)
    Remove {
        name: String,
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
        #[arg(long, default_value = "/srv/repobox-platform/apps")]
        apps_root: String,
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
            replace_route,
        } => {
            let kind = AppKind::parse(&kind).ok_or("bad kind")?;
            let vis = Visibility::parse(&visibility).ok_or("bad visibility")?;
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
                let a = store.create_app(&name, &title, &description, o.id, kind, &target, vis)?;
                store.audit(
                    None,
                    "app.register",
                    &a.name,
                    &format!("{} {} owner={}", a.kind.as_str(), a.target, o.name),
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
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                println!(
                    "NAME                   KIND    TARGET                               VISIBILITY       STATE    TITLE"
                );
                for a in apps {
                    println!(
                        "{:<22} {:<7} {:<36} {:<16} {:<8} {}",
                        a.name,
                        a.kind.as_str(),
                        a.target,
                        a.visibility.as_str(),
                        if a.enabled { "on" } else { "off" },
                        a.title
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
        AppCmd::Remove { name } => {
            let a = need_app(store, &name)?;
            store.delete_app(&a.name)?;
            store.audit(None, "app.remove", &a.name, "cli");
            println!("removed '{}'; re-render routes and reload Caddy", a.name);
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
            apps_root,
            check_roots,
        } => {
            let apps = store.list_apps()?;
            let cfg = render::RenderConfig {
                domain,
                gate,
                apps_root,
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
