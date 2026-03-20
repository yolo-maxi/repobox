use std::collections::HashMap;

/// A parsed .repobox.yml file.
#[derive(Debug, Clone)]
pub struct Config {
    pub groups: HashMap<String, Group>,
    pub permissions: Permissions,
}

/// A named group with members and optional includes.
#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    pub members: Vec<Identity>,
    pub includes: Vec<String>,
}

/// An EVM identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identity {
    pub kind: IdentityKind,
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdentityKind {
    Evm,
}

impl Identity {
    /// Parse an identity string like "evm:0xAAA...123".
    pub fn parse(s: &str) -> Result<Self, ConfigError> {
        if let Some(addr) = s.strip_prefix("evm:") {
            if !addr.starts_with("0x") {
                return Err(ConfigError::InvalidIdentity(s.to_string()));
            }
            Ok(Identity {
                kind: IdentityKind::Evm,
                address: addr.to_string(),
            })
        } else {
            Err(ConfigError::InvalidIdentity(s.to_string()))
        }
    }
}

impl std::fmt::Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            IdentityKind::Evm => write!(f, "evm:{}", self.address),
        }
    }
}

/// The permissions section of the config.
#[derive(Debug, Clone)]
pub struct Permissions {
    pub default: DefaultPolicy,
    pub rules: Vec<Rule>,
}

/// What happens when no rule matches a verb+target combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultPolicy {
    Allow,
    Deny,
}

/// A single permission rule in Subject-Verb-Object form.
#[derive(Debug, Clone)]
pub struct Rule {
    pub subject: Subject,
    pub verb: Verb,
    pub deny: bool,
    pub target: Target,
    /// Line number in the original config (for priority/debugging).
    pub line: usize,
}

/// The subject of a rule — either a group or a direct identity.
#[derive(Debug, Clone)]
pub enum Subject {
    Group(String),
    Identity(Identity),
}

impl Subject {
    /// Check if the given identity matches this subject, using the group map for resolution.
    pub fn matches(&self, identity: &Identity, groups: &HashMap<String, Vec<Identity>>) -> bool {
        match self {
            Subject::Group(name) => groups
                .get(name)
                .map(|members| members.contains(identity))
                .unwrap_or(false),
            Subject::Identity(id) => id == identity,
        }
    }
}

/// A permission verb (action).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    // Branch verbs
    Push,
    Merge,
    Create,
    Delete,
    ForcePush,
    // File verbs
    Edit,
    Write,
    Append,
}

impl Verb {
    pub fn parse(s: &str) -> Result<Self, ConfigError> {
        match s {
            "push" => Ok(Verb::Push),
            "merge" => Ok(Verb::Merge),
            "create" => Ok(Verb::Create),
            "delete" => Ok(Verb::Delete),
            "force-push" => Ok(Verb::ForcePush),
            "edit" => Ok(Verb::Edit),
            "write" => Ok(Verb::Write),
            "append" => Ok(Verb::Append),
            _ => Err(ConfigError::InvalidVerb(s.to_string())),
        }
    }

    pub fn is_branch_verb(self) -> bool {
        matches!(
            self,
            Verb::Push | Verb::Merge | Verb::Create | Verb::Delete | Verb::ForcePush
        )
    }

    pub fn is_file_verb(self) -> bool {
        matches!(self, Verb::Edit | Verb::Write | Verb::Append)
    }
}

impl std::fmt::Display for Verb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verb::Push => write!(f, "push"),
            Verb::Merge => write!(f, "merge"),
            Verb::Create => write!(f, "create"),
            Verb::Delete => write!(f, "delete"),
            Verb::ForcePush => write!(f, "force-push"),
            Verb::Edit => write!(f, "edit"),
            Verb::Write => write!(f, "write"),
            Verb::Append => write!(f, "append"),
        }
    }
}

/// A rule target — branch, path, or combined.
#[derive(Debug, Clone)]
pub struct Target {
    pub branch: Option<String>,
    pub path: Option<String>,
}

impl Target {
    /// Parse a target string.
    /// ">main" → branch only
    /// "contracts/**" → path only
    /// "contracts/** >main" → combined
    /// "*" → wildcard (all)
    pub fn parse(s: &str) -> Result<Self, ConfigError> {
        let s = s.trim();

        // Combined: "path >branch"
        if let Some(idx) = s.rfind(" >") {
            let path = s[..idx].trim();
            let path = strip_path_prefix(path);
            let branch = s[idx + 2..].trim().to_string();
            return Ok(Target {
                branch: Some(branch),
                path: Some(path),
            });
        }

        // Branch only: ">branch"
        if let Some(branch) = s.strip_prefix('>') {
            return Ok(Target {
                branch: Some(branch.to_string()),
                path: None,
            });
        }

        // Path only (or wildcard)
        let path = strip_path_prefix(s);
        Ok(Target {
            branch: None,
            path: Some(path),
        })
    }

    /// Check if this target pattern matches the given branch and/or path.
    pub fn matches(&self, branch: Option<&str>, path: Option<&str>) -> bool {
        let branch_ok = match (&self.branch, branch) {
            (None, _) => true,
            (Some(pattern), Some(actual)) => glob_match(pattern, actual),
            (Some(_), None) => false,
        };

        let path_ok = match (&self.path, path) {
            (None, _) => true,
            (Some(pattern), Some(actual)) => glob_match(pattern, actual),
            (Some(_), None) => false,
        };

        branch_ok && path_ok
    }
}

/// Strip optional `./` prefix from file paths (visual convention, not semantic).
fn strip_path_prefix(s: &str) -> String {
    s.strip_prefix("./").unwrap_or(s).to_string()
}

/// Simple glob matching: `*` matches within one level, `**` matches recursively.
pub fn glob_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    // Convert glob to a simple regex-like matcher
    let parts: Vec<&str> = pattern.split('/').collect();
    let value_parts: Vec<&str> = value.split('/').collect();
    glob_match_parts(&parts, &value_parts)
}

fn glob_match_parts(pattern: &[&str], value: &[&str]) -> bool {
    if pattern.is_empty() && value.is_empty() {
        return true;
    }
    if pattern.is_empty() {
        return false;
    }

    if pattern[0] == "**" {
        // ** matches zero or more path segments
        if glob_match_parts(&pattern[1..], value) {
            return true;
        }
        if !value.is_empty() {
            return glob_match_parts(pattern, &value[1..]);
        }
        return false;
    }

    if value.is_empty() {
        return false;
    }

    if pattern[0] == "*" || pattern[0] == value[0] {
        return glob_match_parts(&pattern[1..], &value[1..]);
    }

    // Handle patterns like "feature/*" matching "feature/fix"
    if pattern[0].contains('*') {
        let p = pattern[0];
        if let Some(prefix) = p.strip_suffix('*') {
            if value[0].starts_with(prefix) {
                return glob_match_parts(&pattern[1..], &value[1..]);
            }
        }
    }

    false
}

/// Config parsing/validation errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("invalid identity: {0}")]
    InvalidIdentity(String),

    #[error("invalid verb: {0}")]
    InvalidVerb(String),

    #[error("unknown group: {0}")]
    UnknownGroup(String),

    #[error("circular dependency: {0}")]
    CircularDependency(String),

    #[error("max include depth exceeded ({0})")]
    MaxIncludeDepth(usize),

    #[error("invalid rule: {0}")]
    InvalidRule(String),

    #[error("parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },
}
