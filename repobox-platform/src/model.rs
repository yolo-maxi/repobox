//! Domain types and the validation rules that keep registry values safe to
//! embed in a Caddyfile. Every string that reaches the rendered route model
//! goes through one of the validators here.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    PublicUnlisted,
    PublicListed,
}

impl Visibility {
    pub const ALL: [Visibility; 3] = [
        Visibility::Private,
        Visibility::PublicUnlisted,
        Visibility::PublicListed,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Visibility::Private => "private",
            Visibility::PublicUnlisted => "public_unlisted",
            Visibility::PublicListed => "public_listed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "private" => Some(Visibility::Private),
            "public_unlisted" => Some(Visibility::PublicUnlisted),
            "public_listed" => Some(Visibility::PublicListed),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Visibility::Private => "Private",
            Visibility::PublicUnlisted => "Public · unlisted",
            Visibility::PublicListed => "Public · listed",
        }
    }

    pub fn help(self) -> &'static str {
        match self {
            Visibility::Private => {
                "Only named users with a grant can open it. Listed only for them."
            }
            Visibility::PublicUnlisted => {
                "Anyone with the link can open it. Never shown in the directory."
            }
            Visibility::PublicListed => {
                "Anyone can open it and it appears in the public directory."
            }
        }
    }

    pub fn is_public(self) -> bool {
        !matches!(self, Visibility::Private)
    }
}

/// The platform identity contract of a managed app (repo.box policy).
///
/// `Platform`: the app authenticates nobody itself. It receives the identity
/// the edge gate injects (`X-RepoBox-*`, browser copies stripped) and uses
/// it only to scope records; there is no app password, login, setup link or
/// app session. Registering a *private* app requires this declaration, and
/// only an app that carries it may be switched to private.
///
/// `Pending`: registered before the policy (or public and undeclared); its
/// own login, if any, has not been removed and reviewed. It keeps serving,
/// is flagged everywhere the manifest is shown, and cannot become private.
///
/// The proxy cannot prove what application code renders; the declaration is
/// enforced at the only supported publish path (`app register`, `app
/// attest`) and by migration/preflight review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityContract {
    Platform,
    Pending,
}

impl IdentityContract {
    pub fn as_str(self) -> &'static str {
        match self {
            IdentityContract::Platform => "platform",
            IdentityContract::Pending => "pending",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "platform" => Some(IdentityContract::Platform),
            "pending" => Some(IdentityContract::Pending),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            IdentityContract::Platform => "platform identity",
            IdentityContract::Pending => "pending review",
        }
    }
    pub fn help(self) -> &'static str {
        match self {
            IdentityContract::Platform => {
                "The app trusts only the identity the edge injects and has no login of its own; app-level authorisation is record scoping by platform user."
            }
            IdentityContract::Pending => {
                "Not yet reviewed for the platform identity contract: any app-level password, login, setup link or session must be removed, then an operator runs `app attest`. Until then it cannot be made private."
            }
        }
    }
    pub fn is_platform(self) -> bool {
        matches!(self, IdentityContract::Platform)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    Static,
    Proxy,
}

impl AppKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AppKind::Static => "static",
            AppKind::Proxy => "proxy",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "static" => Some(AppKind::Static),
            "proxy" => Some(AppKind::Proxy),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    Member,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Member => "member",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Role::Admin),
            "member" => Some(Role::Member),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub role: Role,
    pub enabled: bool,
    pub created_at: i64,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }
}

#[derive(Debug, Clone)]
pub struct App {
    pub id: i64,
    pub name: String,
    pub title: String,
    pub description: String,
    pub owner_id: i64,
    pub kind: AppKind,
    pub target: String,
    pub visibility: Visibility,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub identity: IdentityContract,
    /// Audited, fixed-route anonymous handoff surface on this private parent app.
    pub agent_handoff_v1: bool,
    pub ai: AiPolicy,
    /// Set when an external publisher created the app: only that publisher
    /// can see or change it through the publisher API, and its route is
    /// rendered by the deploy worker (not by `routes render`).
    pub publisher_id: Option<i64>,
}

impl App {
    pub fn host(&self, domain: &str) -> String {
        format!("{}.{}", self.name, domain)
    }
    pub fn url(&self, domain: &str) -> String {
        format!("https://{}/", self.host(domain))
    }
}

// -------------------------------------------------------------------- AI

/// The only AI provider in v1: ChatMock on the Hetzner box, reached through
/// the platform broker. Apps never see its address or credentials.
pub const AI_PROVIDER_CHATMOCK: &str = "chatmock";
/// The one abuse/quota policy a *public* app may declare to get AI: only
/// signed-in platform users can call it (never anonymous visitors) and the
/// explicit per-user and per-app daily quotas apply.
pub const AI_PUBLIC_POLICY_SIGNED_IN_QUOTA: &str = "signed-in-quota";
/// Models the platform knows how to route. The broker additionally
/// intersects its own allowlist with what live ChatMock exposes.
pub const AI_KNOWN_MODELS: &[&str] = &["gpt-5.6-terra", "gpt-5.6-luna", "gpt-5.6-sol", "gpt-5.5"];
pub const AI_DEFAULT_MODEL: &str = "gpt-5.6-terra";
pub const AI_DEFAULT_MODELS: &[&str] = &["gpt-5.6-terra", "gpt-5.6-luna"];
pub const AI_DEFAULT_MAX_INPUT_CHARS: i64 = 32_000;
pub const AI_DEFAULT_MAX_OUTPUT_TOKENS: i64 = 2_048;
pub const AI_DEFAULT_USER_DAILY_REQUESTS: i64 = 200;
pub const AI_DEFAULT_APP_DAILY_REQUESTS: i64 = 2_000;
/// Platform ceilings: no app policy may exceed these.
pub const AI_MAX_INPUT_CHARS_CEILING: i64 = 64_000;
pub const AI_MAX_OUTPUT_TOKENS_CEILING: i64 = 4_096;
pub const AI_USER_DAILY_CEILING: i64 = 2_000;
pub const AI_APP_DAILY_CEILING: i64 = 20_000;
/// Request-shape ceilings enforced by both the platform handler and the broker.
pub const AI_MAX_BODY_BYTES: usize = 256 * 1024;
pub const AI_MAX_MESSAGES: usize = 64;

/// Per-app AI policy, persisted in the registry next to the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiPolicy {
    pub enabled: bool,
    pub provider: String,
    pub default_model: String,
    pub models: Vec<String>,
    pub max_input_chars: i64,
    pub max_output_tokens: i64,
    pub user_daily_requests: i64,
    pub app_daily_requests: i64,
    /// `None` for private apps. A public app needs
    /// `Some(AI_PUBLIC_POLICY_SIGNED_IN_QUOTA)` before AI can be enabled.
    pub public_policy: Option<String>,
}

impl AiPolicy {
    /// The policy a private platform-identity app gets at registration.
    pub fn private_default() -> Self {
        Self {
            enabled: true,
            provider: AI_PROVIDER_CHATMOCK.into(),
            default_model: AI_DEFAULT_MODEL.into(),
            models: AI_DEFAULT_MODELS.iter().map(|m| m.to_string()).collect(),
            max_input_chars: AI_DEFAULT_MAX_INPUT_CHARS,
            max_output_tokens: AI_DEFAULT_MAX_OUTPUT_TOKENS,
            user_daily_requests: AI_DEFAULT_USER_DAILY_REQUESTS,
            app_daily_requests: AI_DEFAULT_APP_DAILY_REQUESTS,
            public_policy: None,
        }
    }

    /// Same limits, switched off (existing apps, public apps without a policy).
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Self::private_default()
        }
    }

    /// The registration default for an app: on for private apps that carry
    /// the platform identity contract, off for everything else.
    pub fn registration_default(visibility: Visibility, identity: IdentityContract) -> Self {
        if visibility == Visibility::Private && identity.is_platform() {
            Self::private_default()
        } else {
            Self::disabled()
        }
    }

    pub fn models_csv(&self) -> String {
        self.models.join(",")
    }

    pub fn parse_models(csv: &str) -> Vec<String> {
        csv.split(',')
            .map(|m| m.trim().to_string())
            .filter(|m| !m.is_empty())
            .collect()
    }

    /// Validate the policy for an app with this visibility and identity
    /// contract. Limits are always validated (a disabled policy is re-enabled
    /// as-is), the public/identity rules only when enabled.
    pub fn validate(
        &self,
        visibility: Visibility,
        identity: IdentityContract,
    ) -> Result<(), String> {
        if self.provider != AI_PROVIDER_CHATMOCK {
            return Err(format!(
                "AI provider must be '{AI_PROVIDER_CHATMOCK}' (the only provider in v1)"
            ));
        }
        if self.models.is_empty() || self.models.len() > AI_KNOWN_MODELS.len() {
            return Err("AI policy needs at least one allowed model".into());
        }
        for m in &self.models {
            if !AI_KNOWN_MODELS.contains(&m.as_str()) {
                return Err(format!(
                    "model '{m}' is not routable; known models: {}",
                    AI_KNOWN_MODELS.join(", ")
                ));
            }
        }
        let mut dedup = self.models.clone();
        dedup.sort();
        dedup.dedup();
        if dedup.len() != self.models.len() {
            return Err("allowed models must not repeat".into());
        }
        if !self.models.contains(&self.default_model) {
            return Err(format!(
                "default model '{}' must be one of the allowed models",
                self.default_model
            ));
        }
        let range = |v: i64, max: i64, what: &str| {
            if (1..=max).contains(&v) {
                Ok(())
            } else {
                Err(format!("{what} must be 1-{max} (platform ceiling)"))
            }
        };
        range(
            self.max_input_chars,
            AI_MAX_INPUT_CHARS_CEILING,
            "max input characters",
        )?;
        range(
            self.max_output_tokens,
            AI_MAX_OUTPUT_TOKENS_CEILING,
            "max output tokens",
        )?;
        range(
            self.user_daily_requests,
            AI_USER_DAILY_CEILING,
            "per-user daily requests",
        )?;
        range(
            self.app_daily_requests,
            AI_APP_DAILY_CEILING,
            "per-app daily requests",
        )?;
        if self.user_daily_requests > self.app_daily_requests {
            return Err("per-user daily requests may not exceed per-app daily requests".into());
        }
        if let Some(p) = &self.public_policy
            && p != AI_PUBLIC_POLICY_SIGNED_IN_QUOTA
        {
            return Err(format!(
                "public AI policy must be '{AI_PUBLIC_POLICY_SIGNED_IN_QUOTA}'"
            ));
        }
        if self.enabled {
            if !identity.is_platform() {
                return Err("AI requires the platform identity contract (`app attest` first): the endpoint scopes usage by the gate-injected user".into());
            }
            if visibility.is_public() && self.public_policy.is_none() {
                return Err(format!(
                    "public apps need an explicit abuse/quota policy before AI can be enabled (`--public-policy {AI_PUBLIC_POLICY_SIGNED_IN_QUOTA}` with explicit per-user and per-app daily quotas); only signed-in platform users can call it"
                ));
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "provider": self.provider,
            "default_model": self.default_model,
            "models": self.models,
            "max_input_chars": self.max_input_chars,
            "max_output_tokens": self.max_output_tokens,
            "user_daily_requests": self.user_daily_requests,
            "app_daily_requests": self.app_daily_requests,
            "public_policy": self.public_policy,
            "streaming": false,
            "endpoint_path": AI_CHAT_PATH,
        })
    }
}

/// The same-origin path every managed app host reserves for the platform AI
/// endpoint, and the prefix Caddy keeps away from app origins.
pub const RESERVED_PATH_PREFIX: &str = "/_repo_box/";
pub const AI_CHAT_PATH: &str = "/_repo_box/ai/v1/chat/completions";
pub const AI_MODELS_PATH: &str = "/_repo_box/ai/v1/models";

// --------------------------------------------------------- service tokens

/// Scopes a machine (service) token can carry. A token is always bound to
/// one owner and only ever sees apps that owner owns (optionally narrowed
/// to an explicit app list). There is deliberately no scope for users,
/// grants, sessions, Caddy apply, SSH, the database or provider secrets.
pub const SERVICE_SCOPES: &[(&str, &str)] = &[
    (
        "apps:read",
        "List and inspect the owner's apps (registry fields, identity contract, AI policy).",
    ),
    (
        "apps:request",
        "File an app registration request; an operator approves it with the CLI.",
    ),
    (
        "ai:read",
        "Read an app's AI policy and today's usage counters.",
    ),
    (
        "ai:write",
        "Change an app's AI policy within platform ceilings (enable/disable, models, limits, quotas).",
    ),
    (
        "routes:read",
        "Preview the generated Caddy route of an app (read-only; applying routes stays operator-only).",
    ),
    (
        "release:read",
        "Read the control plane release status (version, schema, AI broker reachability).",
    ),
];

pub fn validate_scope(scope: &str) -> Result<(), String> {
    if SERVICE_SCOPES.iter().any(|(s, _)| *s == scope) {
        Ok(())
    } else {
        Err(format!(
            "unknown scope '{scope}'; known: {}",
            SERVICE_SCOPES
                .iter()
                .map(|(s, _)| *s)
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

/// Raw service tokens are `rbp_` + 43 base64url characters.
pub const SERVICE_TOKEN_PREFIX: &str = "rbp_";

/// Host labels that must never become managed app routes because they are
/// already meaningful on repo.box or would collide with infrastructure.
pub const RESERVED_APP_NAMES: &[&str] = &[
    "auth",
    "www",
    "git",
    "ens",
    "api",
    "repo",
    "mail",
    "smtp",
    "admin",
    "ns1",
    "ns2",
    "localhost",
    "files",
    "upload",
    "explore",
    "docs",
    "playground",
];

/// A managed app name is a single DNS label: lowercase, digits and hyphens,
/// no leading or trailing hyphen, at most 63 characters, and not reserved.
pub fn validate_app_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 63 {
        return Err("app name must be 1-63 characters".into());
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("app name may only contain a-z, 0-9 and '-'".into());
    }
    if name.starts_with('-') || name.ends_with('-') {
        return Err("app name may not start or end with '-'".into());
    }
    if RESERVED_APP_NAMES.contains(&name) {
        return Err(format!("app name '{name}' is reserved"));
    }
    Ok(())
}

/// User names are short handles: lowercase, digits, '.', '_' and '-'.
pub fn validate_user_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 32 {
        return Err("user name must be 1-32 characters".into());
    }
    let first = name.as_bytes()[0];
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return Err("user name must start with a letter or digit".into());
    }
    if !name.bytes().all(|b| {
        b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'_' || b == b'-'
    }) {
        return Err("user name may only contain a-z, 0-9, '.', '_' and '-'".into());
    }
    Ok(())
}

pub fn validate_display_name(name: &str) -> Result<(), String> {
    let n = name.trim();
    if n.is_empty() || n.chars().count() > 64 {
        return Err("display name must be 1-64 characters".into());
    }
    if n.chars().any(|c| c.is_control()) {
        return Err("display name may not contain control characters".into());
    }
    Ok(())
}

/// Validate and normalise a route target.
///
/// * `proxy` targets must be loopback (`127.0.0.1:PORT`, `localhost:PORT` or
///   `[::1]:PORT`). Anything else would let the edge forward to a non-private
///   origin, so it is rejected outright.
/// * `static` targets must be absolute, contain no `..` segments and only use
///   a conservative character set, because the path is written verbatim into
///   the Caddyfile.
pub fn validate_target(kind: AppKind, target: &str) -> Result<String, String> {
    match kind {
        AppKind::Proxy => {
            let (host, port) = target
                .rsplit_once(':')
                .ok_or_else(|| "proxy target must look like 127.0.0.1:PORT".to_string())?;
            let port: u16 = port
                .parse()
                .map_err(|_| "proxy target port must be 1-65535".to_string())?;
            if port == 0 {
                return Err("proxy target port must be 1-65535".into());
            }
            let host = match host {
                "127.0.0.1" | "localhost" => "127.0.0.1",
                "[::1]" => "[::1]",
                _ => return Err("proxy target must be loopback (127.0.0.1 or [::1])".into()),
            };
            Ok(format!("{host}:{port}"))
        }
        AppKind::Static => {
            if !target.starts_with('/') {
                return Err("static root must be an absolute path".into());
            }
            if target.len() > 200 {
                return Err("static root path is too long".into());
            }
            if !target
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'.' | b'_' | b'-'))
            {
                return Err(
                    "static root may only contain A-Z, a-z, 0-9, '/', '.', '_' and '-'".into(),
                );
            }
            if target.split('/').any(|seg| seg == "..") {
                return Err("static root may not contain '..'".into());
            }
            let trimmed = target.trim_end_matches('/');
            if trimmed.is_empty() {
                return Err("static root may not be '/'".into());
            }
            Ok(trimmed.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_names() {
        assert!(validate_app_name("demo-private").is_ok());
        assert!(validate_app_name("a").is_ok());
        assert!(validate_app_name("Demo").is_err());
        assert!(validate_app_name("-x").is_err());
        assert!(validate_app_name("x-").is_err());
        assert!(validate_app_name("auth").is_err());
        assert!(validate_app_name("a b").is_err());
        assert!(validate_app_name("a{b}").is_err());
        assert!(validate_app_name("").is_err());
    }

    #[test]
    fn targets() {
        assert_eq!(
            validate_target(AppKind::Proxy, "localhost:3231").unwrap(),
            "127.0.0.1:3231"
        );
        assert_eq!(
            validate_target(AppKind::Proxy, "127.0.0.1:80").unwrap(),
            "127.0.0.1:80"
        );
        assert!(validate_target(AppKind::Proxy, "0.0.0.0:3231").is_err());
        assert!(validate_target(AppKind::Proxy, "10.0.0.5:3231").is_err());
        assert!(validate_target(AppKind::Proxy, "127.0.0.1:0").is_err());
        assert!(validate_target(AppKind::Proxy, "127.0.0.1").is_err());
        assert_eq!(
            validate_target(AppKind::Static, "/srv/repobox-platform/apps/x/").unwrap(),
            "/srv/repobox-platform/apps/x"
        );
        assert!(validate_target(AppKind::Static, "relative/dir").is_err());
        assert!(validate_target(AppKind::Static, "/srv/../etc").is_err());
        assert!(validate_target(AppKind::Static, "/srv/x y").is_err());
        assert!(validate_target(AppKind::Static, "/srv/x\n}").is_err());
    }

    #[test]
    fn ai_policy_defaults_and_validation() {
        use Visibility::*;
        let p = AiPolicy::registration_default(Private, IdentityContract::Platform);
        assert!(p.enabled);
        assert_eq!(p.provider, "chatmock");
        assert!(p.models.contains(&p.default_model));
        assert!(p.validate(Private, IdentityContract::Platform).is_ok());
        // Pending or public apps start with AI off.
        assert!(!AiPolicy::registration_default(Private, IdentityContract::Pending).enabled);
        assert!(!AiPolicy::registration_default(PublicListed, IdentityContract::Platform).enabled);
        assert!(
            !AiPolicy::registration_default(PublicUnlisted, IdentityContract::Platform).enabled
        );
        // Public apps need the explicit abuse/quota policy.
        let mut public = AiPolicy::private_default();
        let err = public
            .validate(PublicListed, IdentityContract::Platform)
            .unwrap_err();
        assert!(err.contains("abuse/quota policy"), "{err}");
        public.public_policy = Some(AI_PUBLIC_POLICY_SIGNED_IN_QUOTA.into());
        assert!(
            public
                .validate(PublicListed, IdentityContract::Platform)
                .is_ok()
        );
        public.public_policy = Some("anyone".into());
        assert!(
            public
                .validate(PublicListed, IdentityContract::Platform)
                .is_err()
        );
        // Disabled on a public app is fine without a policy.
        assert!(
            AiPolicy::disabled()
                .validate(PublicListed, IdentityContract::Pending)
                .is_ok()
        );
        // Pending identity cannot have AI on.
        assert!(
            AiPolicy::private_default()
                .validate(PublicUnlisted, IdentityContract::Pending)
                .is_err()
        );
        let bad = |f: &dyn Fn(&mut AiPolicy)| {
            let mut p = AiPolicy::private_default();
            f(&mut p);
            p.validate(Private, IdentityContract::Platform).is_err()
        };
        assert!(bad(&|p| p.provider = "openai".into()));
        assert!(bad(&|p| p.models = vec![]));
        assert!(bad(&|p| p.models = vec!["gpt-4o".into()]));
        assert!(bad(
            &|p| p.models = vec!["gpt-5.6-terra".into(), "gpt-5.6-terra".into()]
        ));
        assert!(bad(&|p| p.default_model = "gpt-5.6-sol".into()));
        assert!(bad(&|p| p.max_input_chars = 0));
        assert!(bad(&|p| p.max_input_chars = AI_MAX_INPUT_CHARS_CEILING + 1));
        assert!(bad(
            &|p| p.max_output_tokens = AI_MAX_OUTPUT_TOKENS_CEILING + 1
        ));
        assert!(bad(&|p| p.user_daily_requests = AI_USER_DAILY_CEILING + 1));
        assert!(bad(&|p| p.app_daily_requests = 0));
        assert!(bad(&|p| {
            p.user_daily_requests = 50;
            p.app_daily_requests = 10;
        }));
        assert_eq!(
            AiPolicy::parse_models(" gpt-5.6-terra, ,gpt-5.6-luna "),
            vec!["gpt-5.6-terra".to_string(), "gpt-5.6-luna".to_string()]
        );
    }

    #[test]
    fn user_names() {
        assert!(validate_user_name("fran").is_ok());
        assert!(validate_user_name("review.bot_1").is_ok());
        assert!(validate_user_name("Fran").is_err());
        assert!(validate_user_name("_x").is_err());
        assert!(validate_user_name("").is_err());
    }
}
