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
}

impl App {
    pub fn host(&self, domain: &str) -> String {
        format!("{}.{}", self.name, domain)
    }
    pub fn url(&self, domain: &str) -> String {
        format!("https://{}/", self.host(domain))
    }
}

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
    fn user_names() {
        assert!(validate_user_name("fran").is_ok());
        assert!(validate_user_name("review.bot_1").is_ok());
        assert!(validate_user_name("Fran").is_err());
        assert!(validate_user_name("_x").is_err());
        assert!(validate_user_name("").is_err());
    }
}
