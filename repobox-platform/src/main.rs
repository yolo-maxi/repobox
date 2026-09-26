//! `repobox-platform`: the repo.box platform control plane.
//!
//! * `serve`        — the auth.repo.box UI plus the loopback gate for Caddy.
//! * `demo-origin`  — the loopback origin behind the private demo app.
//! * `ai-broker`    — the ChatMock broker (runs next to ChatMock on Hetzner).
//! * operator commands (users, apps, grants, AI policy, service tokens,
//!   registration requests, route rendering, backups). These
//!   are the only way to register an app or produce a route: the web UI has no
//!   deploy or route controls by design.
//!
//! Secrets policy: the only time a raw token leaves this program is when an
//! operator command writes a single-use link to a 0600 file it was asked to
//! create. Nothing is ever printed to stdout or logged.

use repobox_platform::{ai, demo_origin, model, render, store, web};

use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::{Parser, Subcommand};

use model::{AiPolicy, AppKind, IdentityContract, Role, Visibility};
use store::{Store, TokenKind};

const DEFAULT_DB: &str = "/var/lib/repobox-platform/platform.db";

#[derive(Parser)]
#[command(
    name = "repobox-platform",
    version,
    about = "repo.box platform control plane",
    long_about = "repo.box platform control plane: named users, app grants, one-time launch codes, the Caddy edge gate, the per-app AI capability and operator route rendering.\n\nThis CLI is the canonical mutating interface. Agents without host access use the scoped machine API instead: discovery at https://auth.repo.box/api/platform/v1 (OpenAPI at /api/platform/v1/openapi.json, MCP at /api/platform/v1/mcp) with an operator-issued service token (`service-token create`). The full agent contract is printed by `repobox-platform skill`.",
    after_help = "Common flows:\n  app register NAME --title T --owner U --kind proxy --target 127.0.0.1:PORT --identity platform\n      (private + platform identity => AI capability on by default; --no-ai to opt out)\n  app ai show NAME | app ai set NAME --models gpt-5.6-terra,gpt-5.6-luna --max-output-tokens 1024\n  service-token create --name agent-x --owner U --scope apps:read --scope ai:read --out FILE\n  app requests list | app requests approve ID\n  routes render --out FILE   (then the guarded Caddy apply in scripts/deploy.sh)\n  skill                      (print the agent SKILL.md)"
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
        /// Loopback URL of the AI broker (the repo.box end of the tunnel),
        /// e.g. http://127.0.0.1:3232. Unset: the AI endpoint answers 503.
        #[arg(long, env = "REPOBOX_PLATFORM_AI_UPSTREAM")]
        ai_upstream: Option<String>,
        /// Bridge secret file. Default: the systemd credential
        /// `ai-bridge-secret` ($CREDENTIALS_DIRECTORY, LoadCredential=).
        #[arg(long, env = "REPOBOX_PLATFORM_AI_SECRET_FILE")]
        ai_secret_file: Option<PathBuf>,
        /// Concurrent AI requests this control plane forwards
        #[arg(long, default_value_t = 8)]
        ai_max_concurrency: usize,
    },
    /// Run the ChatMock broker: loopback only, bridge-secret authenticated,
    /// model allowlist intersected with ChatMock's live models, hard request
    /// ceilings, no body logging. Reached from repo.box only through the
    /// private reverse tunnel.
    AiBroker {
        #[arg(long, default_value = "127.0.0.1:8127")]
        bind: SocketAddr,
        /// ChatMock base URL (loopback)
        #[arg(long, default_value = "http://127.0.0.1:8111")]
        upstream: String,
        /// Bridge secret file. Default: the systemd credential `ai-bridge-secret`.
        #[arg(long, env = "REPOBOX_AI_BROKER_SECRET_FILE")]
        secret_file: Option<PathBuf>,
        /// Models to route (comma separated; must be known platform models)
        #[arg(long, default_value = "gpt-5.6-terra,gpt-5.6-luna,gpt-5.6-sol")]
        models: String,
        #[arg(long, default_value_t = 4)]
        max_concurrency: usize,
        /// Seconds to wait for ChatMock
        #[arg(long, default_value_t = 85)]
        timeout_secs: u64,
    },
    /// Print the agent skill (SKILL.md): endpoint contract, policy, access model
    Skill,
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
    /// Scoped machine credentials for the platform API / MCP (not OAuth)
    #[command(name = "service-token")]
    ServiceToken {
        #[command(subcommand)]
        cmd: ServiceTokenCmd,
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
        /// Opt a private app out of the default AI capability
        #[arg(long)]
        no_ai: bool,
        /// Enable AI on a *public* app: the only policy is `signed-in-quota`
        /// (only signed-in platform users with access; explicit quotas)
        #[arg(long, value_parser = ["signed-in-quota"], requires_all = ["ai_user_daily", "ai_app_daily"])]
        ai_public_policy: Option<String>,
        /// Explicit per-user daily AI requests (public apps)
        #[arg(long)]
        ai_user_daily: Option<i64>,
        /// Explicit per-app daily AI requests (public apps)
        #[arg(long)]
        ai_app_daily: Option<i64>,
    },
    /// Inspect or change an app's AI capability policy
    Ai {
        #[command(subcommand)]
        cmd: AiCmd,
    },
    /// Review app registration requests filed through the machine API
    Requests {
        #[command(subcommand)]
        cmd: RequestsCmd,
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
enum AiCmd {
    /// Show the AI policy, endpoint and today's usage
    Show {
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// Turn the AI capability on (policy must validate: platform identity;
    /// public apps need --public-policy via `set` first)
    Enable { name: String },
    /// Turn the AI capability off (policy values are kept)
    Disable { name: String },
    /// Change policy fields; unspecified fields are kept
    Set {
        name: String,
        /// Default model when a request names none
        #[arg(long)]
        default_model: Option<String>,
        /// Allowed models, comma separated
        #[arg(long)]
        models: Option<String>,
        /// Max total characters across a request's messages
        #[arg(long)]
        max_input_chars: Option<i64>,
        /// Max output tokens (max_tokens default and ceiling)
        #[arg(long)]
        max_output_tokens: Option<i64>,
        /// Requests per user per UTC day
        #[arg(long)]
        user_daily: Option<i64>,
        /// Requests per app per UTC day
        #[arg(long)]
        app_daily: Option<i64>,
        /// Public abuse/quota policy: `signed-in-quota` or `none`
        #[arg(long, value_parser = ["signed-in-quota", "none"])]
        public_policy: Option<String>,
        /// Also enable
        #[arg(long, conflicts_with = "disable")]
        enable: bool,
        /// Also disable
        #[arg(long)]
        disable: bool,
    },
}

#[derive(Subcommand)]
enum RequestsCmd {
    /// List registration requests (newest first)
    List {
        #[arg(long)]
        json: bool,
    },
    /// Register the requested app (same checks as `app register`), then
    /// render routes and apply them with the guarded Caddy path
    Approve {
        id: i64,
        #[arg(long, default_value = "")]
        note: String,
    },
    /// Reject a request
    Reject {
        id: i64,
        #[arg(long)]
        note: String,
    },
}

#[derive(Subcommand)]
enum ServiceTokenCmd {
    /// Issue a token bound to one owner; the raw value is written once to a
    /// new 0600 file and never printed
    Create {
        /// Token name (a-z, 0-9, '.', '_', '-')
        #[arg(long)]
        name: String,
        /// Owner user; the token only reaches apps this user owns
        #[arg(long)]
        owner: String,
        /// Scope (repeat): apps:read apps:request ai:read ai:write routes:read release:read
        #[arg(long = "scope", required = true)]
        scopes: Vec<String>,
        /// Restrict to these apps (repeat; default: all apps the owner owns)
        #[arg(long = "app")]
        apps: Vec<String>,
        /// Lifetime in days (1-90)
        #[arg(long, default_value_t = 30)]
        ttl_days: i64,
        /// File to write the token to (created 0600, must not exist)
        #[arg(long)]
        out: PathBuf,
    },
    /// List tokens (never shows values)
    List,
    /// Revoke a token by name (effective on the next request)
    Revoke { name: String },
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
            ai_upstream,
            ai_secret_file,
            ai_max_concurrency,
        } => {
            init_tracing();
            if !bind.ip().is_loopback() {
                return Err(format!("serve must bind to loopback, got {bind}").into());
            }
            let store = open_store(&cli.db)?;
            let mut cfg = web::Config::defaults(&public_base, &domain);
            cfg.routes.gate = bind.to_string();
            if let Some(url) = ai_upstream {
                let secret = ai::Secret::load(ai_secret_file.as_deref(), "ai-bridge-secret")?;
                cfg.ai = Some(Arc::new(ai::Bridge::new(&url, secret, ai_max_concurrency)?));
                tracing::info!("AI endpoint enabled via broker {url}");
            } else {
                tracing::info!("AI endpoint not configured (no --ai-upstream): it answers 503");
            }
            let state = Arc::new(web::AppState { store, cfg });
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
        Cmd::AiBroker {
            bind,
            upstream,
            secret_file,
            models,
            max_concurrency,
            timeout_secs,
        } => {
            init_tracing();
            let cfg = ai::BrokerConfig {
                upstream,
                secret: ai::Secret::load(secret_file.as_deref(), "ai-bridge-secret")?,
                models: AiPolicy::parse_models(&models),
                max_concurrency,
                timeout: std::time::Duration::from_secs(timeout_secs),
            };
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(ai::serve_broker(bind, cfg))
        }
        Cmd::Skill => {
            print!("{}", web::api::SKILL_MD);
            Ok(())
        }
        Cmd::ServiceToken { cmd } => service_token_cmd(&open_store(&cli.db)?, cmd),
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
            let store = open_store(&cli.db)?;
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
                shown(&out),
                ttl_hours
            );
            Ok(())
        }
        Cmd::User { cmd } => user_cmd(&open_store(&cli.db)?, cmd),
        Cmd::App { cmd } => app_cmd(&open_store(&cli.db)?, cmd),
        Cmd::Routes { cmd } => routes_cmd(&open_store(&cli.db)?, cmd),
        Cmd::Backup { out } => {
            let store = open_store(&cli.db)?;
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
            println!("backup written to {}", shown(&out));
            Ok(())
        }
        Cmd::Audit { limit } => {
            let store = open_store(&cli.db)?;
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

/// `RUST_LOG` tunes this crate's logging, but HTTP/transport crates are
/// capped at `info` whatever the environment says: at debug/trace they can
/// log request lines and headers (URIs with launch codes, cookies, the bridge
/// secret). Our own log lines never contain secrets, URIs or bodies.
/// How an `--out` path is shown to the operator. The host wrapper
/// (/usr/local/bin/repobox-platform) has the service user write into a
/// private spool and passes the path the operator asked for here.
fn shown(out: &Path) -> String {
    std::env::var("REPOBOX_PLATFORM_OUT_DISPLAY").unwrap_or_else(|_| out.display().to_string())
}

/// Open the registry, with a pointer to the wrapper when the caller lacks
/// access (on the host the registry belongs to the `repobox-platform` user).
fn open_store(db: &Path) -> Result<Store, Box<dyn std::error::Error>> {
    Store::open(db).map_err(|e| {
        let denied = std::fs::File::open(db)
            .err()
            .is_some_and(|io| io.kind() == std::io::ErrorKind::PermissionDenied)
            || db.parent().is_some_and(|d| {
                std::fs::read_dir(d)
                    .err()
                    .is_some_and(|io| io.kind() == std::io::ErrorKind::PermissionDenied)
            });
        if denied {
            format!("{e}\nhint: the registry belongs to the `repobox-platform` service user; run the CLI as `/usr/local/bin/repobox-platform …` (it uses sudo + runuser)").into()
        } else {
            e.into()
        }
    })
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let mut filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    for target in [
        "hyper",
        "hyper_util",
        "h2",
        "axum",
        "tower",
        "tower_http",
        "rustls",
    ] {
        if let Ok(d) = format!("{target}=info").parse() {
            filter = filter.add_directive(d);
        }
    }
    tracing_subscriber::fmt().with_env_filter(filter).init();
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
                shown(&out),
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
            no_ai,
            ai_public_policy,
            ai_user_daily,
            ai_app_daily,
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
                let a = if no_ai && a.ai.enabled {
                    let mut p = a.ai.clone();
                    p.enabled = false;
                    store.set_app_ai(a.id, &p)?
                } else if let Some(policy) = ai_public_policy {
                    let mut p = AiPolicy::private_default();
                    p.public_policy = Some(policy);
                    p.user_daily_requests = ai_user_daily.ok_or("--ai-user-daily is required")?;
                    p.app_daily_requests = ai_app_daily.ok_or("--ai-app-daily is required")?;
                    match store.set_app_ai(a.id, &p) {
                        Ok(a) => a,
                        Err(e) => {
                            // Keep registration atomic from the operator's view.
                            store.delete_app(&a.name)?;
                            return Err(e.into());
                        }
                    }
                } else {
                    a
                };
                store.audit(None, "app.ai", &a.name, &ai_audit_detail(&a.ai, "register"));
                println!(
                    "registered '{}' ({} -> {}), owner {}, visibility {}, AI {}",
                    a.name,
                    a.kind.as_str(),
                    a.target,
                    o.name,
                    a.visibility.as_str(),
                    if a.ai.enabled { "on" } else { "off" }
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
                            "ai": a.ai.to_json(),
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
            println!(
                "ai:          {} (default {}, models {}, in {} chars, out {} tokens, {}/user/day, {}/app/day{})",
                if a.ai.enabled { "on" } else { "off" },
                a.ai.default_model,
                a.ai.models_csv(),
                a.ai.max_input_chars,
                a.ai.max_output_tokens,
                a.ai.user_daily_requests,
                a.ai.app_daily_requests,
                a.ai.public_policy
                    .as_deref()
                    .map(|p| format!(", public policy {p}"))
                    .unwrap_or_default()
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
            if store.set_app_visibility(a.id, v)? {
                store.audit(
                    None,
                    "app.ai",
                    &a.name,
                    "enabled=false (made public without a public AI policy)",
                );
                println!(
                    "note: the AI capability was switched off (public apps need `app ai set {} --public-policy signed-in-quota --user-daily N --app-daily N --enable`)",
                    a.name
                );
            }
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
        AppCmd::Ai { cmd } => ai_cmd(store, cmd)?,
        AppCmd::Requests { cmd } => requests_cmd(store, cmd)?,
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

fn ai_audit_detail(p: &AiPolicy, via: &str) -> String {
    format!(
        "{via} enabled={} default={} models={} in={} out={} user/day={} app/day={} public={}",
        p.enabled,
        p.default_model,
        p.models_csv(),
        p.max_input_chars,
        p.max_output_tokens,
        p.user_daily_requests,
        p.app_daily_requests,
        p.public_policy.as_deref().unwrap_or("-")
    )
}

fn ai_cmd(store: &Store, cmd: AiCmd) -> Result<(), Box<dyn std::error::Error>> {
    let update =
        |name: &str, f: &dyn Fn(&mut AiPolicy)| -> Result<model::App, Box<dyn std::error::Error>> {
            let a = need_app(store, name)?;
            let mut p = a.ai.clone();
            f(&mut p);
            let a = store.set_app_ai(a.id, &p)?;
            store.audit(None, "app.ai", &a.name, &ai_audit_detail(&a.ai, "cli"));
            Ok(a)
        };
    match cmd {
        AiCmd::Show { name, json } => {
            let a = need_app(store, &name)?;
            let (requests, users) = store.ai_usage_today(a.id)?;
            let endpoint = format!("https://{}.repo.box{}", a.name, model::AI_CHAT_PATH);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "app": a.name, "policy": a.ai.to_json(), "endpoint": endpoint,
                        "usage_today_utc": {"requests": requests, "users": users},
                    }))?
                );
            } else {
                println!("app:           {}", a.name);
                println!("enabled:       {}", a.ai.enabled);
                println!("provider:      {}", a.ai.provider);
                println!("default model: {}", a.ai.default_model);
                println!("models:        {}", a.ai.models_csv());
                println!("max input:     {} characters", a.ai.max_input_chars);
                println!("max output:    {} tokens", a.ai.max_output_tokens);
                println!(
                    "quota:         {}/user/day, {}/app/day (UTC)",
                    a.ai.user_daily_requests, a.ai.app_daily_requests
                );
                println!(
                    "public policy: {}",
                    a.ai.public_policy.as_deref().unwrap_or("none")
                );
                println!("streaming:     no (v1 is non-streaming)");
                println!("endpoint:      POST {endpoint}");
                println!("today:         {requests} request(s) by {users} user(s)");
            }
        }
        AiCmd::Enable { name } => {
            let a = update(&name, &|p| p.enabled = true)?;
            println!(
                "'{}' AI capability on (default {})",
                a.name, a.ai.default_model
            );
        }
        AiCmd::Disable { name } => {
            let a = update(&name, &|p| p.enabled = false)?;
            println!("'{}' AI capability off", a.name);
        }
        AiCmd::Set {
            name,
            default_model,
            models,
            max_input_chars,
            max_output_tokens,
            user_daily,
            app_daily,
            public_policy,
            enable,
            disable,
        } => {
            let a = update(&name, &|p| {
                if let Some(m) = &models {
                    p.models = AiPolicy::parse_models(m);
                }
                if let Some(m) = &default_model {
                    p.default_model = m.clone();
                }
                if let Some(v) = max_input_chars {
                    p.max_input_chars = v;
                }
                if let Some(v) = max_output_tokens {
                    p.max_output_tokens = v;
                }
                if let Some(v) = user_daily {
                    p.user_daily_requests = v;
                }
                if let Some(v) = app_daily {
                    p.app_daily_requests = v;
                }
                match public_policy.as_deref() {
                    Some("none") => p.public_policy = None,
                    Some(v) => p.public_policy = Some(v.to_string()),
                    None => {}
                }
                if enable {
                    p.enabled = true;
                }
                if disable {
                    p.enabled = false;
                }
            })?;
            println!(
                "'{}' AI policy updated: {}",
                a.name,
                ai_audit_detail(&a.ai, "now")
            );
        }
    }
    Ok(())
}

fn requests_cmd(store: &Store, cmd: RequestsCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        RequestsCmd::List { json } => {
            let list = store.list_app_requests(None)?;
            if json {
                let v: Vec<serde_json::Value> = list
                    .iter()
                    .map(|r| serde_json::json!({
                        "id": r.id, "name": r.name, "title": r.title, "kind": r.kind.as_str(),
                        "target": r.target, "visibility": r.visibility.as_str(), "owner_id": r.owner_id,
                        "status": r.status, "note": r.note, "created_at": r.created_at,
                    }))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                println!(
                    "ID    STATUS    NAME                   KIND    TARGET                               VISIBILITY       OWNER         NOTE"
                );
                for r in list {
                    let owner = store
                        .user_by_id(r.owner_id)
                        .map(|u| u.name)
                        .unwrap_or_else(|_| "?".into());
                    println!(
                        "{:<5} {:<9} {:<22} {:<7} {:<36} {:<16} {:<13} {}",
                        r.id,
                        r.status,
                        r.name,
                        r.kind.as_str(),
                        r.target,
                        r.visibility.as_str(),
                        owner,
                        r.note
                    );
                }
            }
        }
        RequestsCmd::Approve { id, note } => {
            let r = store.app_request(id)?;
            if r.status != "pending" {
                return Err(format!("request {id} is already {}", r.status).into());
            }
            let owner = store.user_by_id(r.owner_id)?;
            // Private requests were accepted only with the platform identity
            // declaration, so they register with it (and AI on by default).
            let identity = if r.visibility == Visibility::Private {
                IdentityContract::Platform
            } else {
                IdentityContract::Pending
            };
            let a = store.create_app(
                &r.name,
                &r.title,
                &r.description,
                owner.id,
                r.kind,
                &r.target,
                r.visibility,
                identity,
            )?;
            store.decide_app_request(id, true, &note)?;
            store.audit(
                None,
                "app.register",
                &a.name,
                &format!(
                    "{} {} owner={} identity={} request={id}",
                    a.kind.as_str(),
                    a.target,
                    owner.name,
                    a.identity.as_str()
                ),
            );
            println!(
                "approved request {id}: registered '{}' ({} -> {}), AI {}",
                a.name,
                a.kind.as_str(),
                a.target,
                if a.ai.enabled { "on" } else { "off" }
            );
            println!(
                "next: render routes and apply them with the guarded Caddy path (scripts/deploy.sh)"
            );
        }
        RequestsCmd::Reject { id, note } => {
            if !store.decide_app_request(id, false, &note)? {
                return Err(format!("request {id} is not pending").into());
            }
            store.audit(None, "app.request_reject", &id.to_string(), &note);
            println!("rejected request {id}");
        }
    }
    Ok(())
}

fn service_token_cmd(
    store: &Store,
    cmd: ServiceTokenCmd,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        ServiceTokenCmd::Create {
            name,
            owner,
            scopes,
            apps,
            ttl_days,
            out,
        } => {
            use std::os::unix::fs::OpenOptionsExt;
            if out.exists() {
                return Err(format!("{} already exists; choose a new file", out.display()).into());
            }
            let o = need_user(store, &owner)?;
            // Open the file first so a failure never leaves a live token nobody holds.
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&out)?;
            let (raw, tok) =
                match store.create_service_token(&name, &o, &scopes, &apps, ttl_days * 86400) {
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
                "# repo.box platform service token '{}' for owner '{}'; scopes: {}; expires {}; use as Authorization: Bearer <first line>",
                tok.name,
                o.name,
                tok.scopes.join(" "),
                web::html::fmt_ts(tok.expires_at)
            )?;
            store.audit(
                None,
                "service_token.create",
                &tok.name,
                &format!(
                    "owner={} scopes={} apps={}",
                    o.name,
                    tok.scopes.join(","),
                    tok.apps.join(",")
                ),
            );
            println!(
                "service token '{}' written to {} (owner {}, expires {})",
                tok.name,
                shown(&out),
                o.name,
                web::html::fmt_ts(tok.expires_at)
            );
        }
        ServiceTokenCmd::List => {
            let now = store.now();
            println!(
                "NAME                 OWNER        STATUS   EXPIRES            LAST USED          SCOPES / APPS"
            );
            for t in store.list_service_tokens()? {
                let owner = store
                    .user_by_id(t.owner_id)
                    .map(|u| u.name)
                    .unwrap_or_else(|_| "?".into());
                println!(
                    "{:<20} {:<12} {:<8} {:<18} {:<18} {} / {}",
                    t.name,
                    owner,
                    t.status(now),
                    web::html::fmt_ts(t.expires_at),
                    t.last_used_at
                        .map(web::html::fmt_ts)
                        .unwrap_or_else(|| "-".into()),
                    t.scopes.join(" "),
                    if t.apps.is_empty() {
                        "all owned apps".to_string()
                    } else {
                        t.apps.join(",")
                    }
                );
            }
        }
        ServiceTokenCmd::Revoke { name } => {
            let changed = store.revoke_service_token(&name)?;
            store.audit(None, "service_token.revoke", &name, "cli");
            println!(
                "{} '{name}'",
                if changed {
                    "revoked"
                } else {
                    "already revoked:"
                }
            );
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
                    eprintln!("rendered {} app route(s) to {}", apps.len(), shown(&p));
                }
                None => print!("{text}"),
            }
        }
    }
    Ok(())
}
