use std::collections::HashMap;

/// A parsed .repobox/config.yml file.
#[derive(Debug, Clone)]
pub struct Config {
    pub groups: HashMap<String, Group>,
    pub permissions: Permissions,
    pub virtuals: Option<VirtualsConfig>,
}

/// A named group with members, optional includes, and optional resolver.
#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    pub members: Vec<Identity>,
    pub includes: Vec<String>,
    pub resolver: Option<GroupResolver>,
}

/// Remote group resolver — checks membership dynamically.
#[derive(Debug, Clone)]
pub enum GroupResolver {
    /// HTTP endpoint: GET <url>/members/<address> → { "member": true/false }
    Http {
        url: String,
        cache_ttl: u64, // seconds, 0 = no cache
    },
    /// On-chain: calls function(address) → bool via server proxy
    Onchain {
        chain: u64,
        contract: String,
        function: String,
        cache_ttl: u64,
    },
}

/// x402 payment configuration — parsed from `.repobox/x402.yml` (separate from config.yml).
#[derive(Debug, Clone)]
pub struct X402Config {
    /// Price in USDC for permanent read access (e.g. "1.00").
    pub read_price: String,
    /// Payment recipient address (e.g. "0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048").
    pub recipient: String,
    /// Blockchain network for payments (e.g. "base").
    pub network: String,
}

/// Virtuals integration configuration for AI agent bug bounties and payments.
#[derive(Debug, Clone)]
pub struct VirtualsConfig {
    /// Whether virtuals integration is enabled.
    pub enabled: bool,
    /// Bug bounty amounts by issue severity in USDC.
    pub bug_bounties: BugBountyConfig,
    /// Requirements for agents to participate.
    pub agent_requirements: AgentRequirements,
    /// Payment configuration for successful merges.
    pub payments: Option<VirtualsPaymentConfig>,
}

/// Bug bounty configuration by severity level.
#[derive(Debug, Clone)]
pub struct BugBountyConfig {
    pub critical: String,
    pub high: String,
    pub medium: String,
    pub low: String,
}

/// Requirements for AI agents to participate in bug bounties.
#[derive(Debug, Clone)]
pub struct AgentRequirements {
    /// Minimum reputation score (0.0 to 1.0).
    pub min_reputation: f64,
    /// Whether test coverage is required for fixes.
    pub required_tests: bool,
    /// Whether human review is required before merge.
    pub human_review_required: bool,
}

/// Payment configuration for Virtuals integration.
#[derive(Debug, Clone)]
pub struct VirtualsPaymentConfig {
    /// Blockchain network for payments (e.g. "base").
    pub network: String,
    /// Token contract address or symbol (e.g. "USDC").
    pub token: String,
    /// Treasury address holding funds for bounty payments.
    pub treasury: String,
    /// Address that sponsors gas fees for payments.
    pub gas_sponsor: Option<String>,
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
    Ens,  // New variant for ENS names
}

impl Identity {
    /// Parse an identity string like "evm:0xAAA...123" or "ens:vitalik.eth" or "vitalik.eth".
    pub fn parse(s: &str) -> Result<Self, ConfigError> {
        if let Some(addr) = s.strip_prefix("evm:") {
            if !addr.starts_with("0x") || addr.len() != 42 {
                return Err(ConfigError::InvalidIdentity(s.to_string()));
            }
            Ok(Identity {
                kind: IdentityKind::Evm,
                address: addr.to_string(),
            })
        } else if let Some(name) = s.strip_prefix("ens:") {
            validate_ens_name(name)?;
            Ok(Identity {
                kind: IdentityKind::Ens,
                address: name.to_string(),
            })
        } else if is_ens_name(s) {
            // Implicit ENS detection: bare names like "vitalik.eth" are auto-detected
            // by their TLD suffix (.eth, .box, .xyz, etc.) without requiring an "ens:" prefix.
            // Both forms are equivalent: "vitalik.eth" == "ens:vitalik.eth"
            // The canonical form (used in Display/serialization) always includes the prefix.
            validate_ens_name(s)?;
            Ok(Identity {
                kind: IdentityKind::Ens,
                address: s.to_string(),
            })
        } else if s.starts_with("0x") && s.len() == 42 {
            // Legacy EVM format without prefix
            Ok(Identity {
                kind: IdentityKind::Evm,
                address: s.to_string(),
            })
        } else {
            Err(ConfigError::InvalidIdentity(s.to_string()))
        }
    }

    /// Get the canonical string representation
    pub fn canonical(&self) -> String {
        match self.kind {
            IdentityKind::Evm => format!("evm:{}", self.address),
            IdentityKind::Ens => format!("ens:{}", self.address),
        }
    }
}

impl std::fmt::Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            IdentityKind::Evm => write!(f, "evm:{}", self.address),
            IdentityKind::Ens => write!(f, "ens:{}", self.address),
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
    /// Matches everyone — wildcard subject `*`
    All,
    Group(String),
    Identity(Identity),
}

impl Subject {
    /// Check if the given identity matches this subject, using the group map for resolution.
    pub fn matches(&self, identity: &Identity, groups: &HashMap<String, Vec<Identity>>) -> bool {
        match self {
            Subject::All => true,
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
    // Access verbs
    Read,
    // Branch verbs
    Push,
    Merge,
    Branch,
    Delete,
    ForcePush,
    // File verbs (hierarchy: edit > insert > append > upload)
    Edit,
    Insert,
    Append,
    Upload,
}

impl Verb {
    pub fn parse(s: &str) -> Result<Self, ConfigError> {
        match s {
            "read" => Ok(Verb::Read),
            "push" => Ok(Verb::Push),
            "merge" => Ok(Verb::Merge),
            "branch" => Ok(Verb::Branch),
            "upload" => Ok(Verb::Upload),
            "insert" => Ok(Verb::Insert),
            "delete" => Ok(Verb::Delete),
            "force-push" => Ok(Verb::ForcePush),
            "edit" => Ok(Verb::Edit),
            "append" => Ok(Verb::Append),
            // Deprecated aliases — parse but warn
            "write" | "create" => {
                eprintln!(
                    "repo.box: '{}' is deprecated, use 'upload' instead (for new files) or 'insert' (for adding lines)",
                    s
                );
                Ok(Verb::Upload)
            }
            _ => Err(ConfigError::InvalidVerb(s.to_string())),
        }
    }

    pub fn is_branch_verb(self) -> bool {
        matches!(
            self,
            Verb::Push | Verb::Merge | Verb::Branch | Verb::Delete | Verb::ForcePush
        )
    }

    pub fn is_file_verb(self) -> bool {
        matches!(self, Verb::Edit | Verb::Insert | Verb::Append | Verb::Upload)
    }

    pub fn is_access_verb(self) -> bool {
        matches!(self, Verb::Read)
    }
}

impl std::fmt::Display for Verb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verb::Read => write!(f, "read"),
            Verb::Push => write!(f, "push"),
            Verb::Merge => write!(f, "merge"),
            Verb::Branch => write!(f, "branch"),
            Verb::Upload => write!(f, "upload"),
            Verb::Insert => write!(f, "insert"),
            Verb::Delete => write!(f, "delete"),
            Verb::ForcePush => write!(f, "force-push"),
            Verb::Edit => write!(f, "edit"),
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
/// Check if a string looks like an ENS name
fn is_ens_name(s: &str) -> bool {
    s.contains('.') && (
        s.ends_with(".eth") || s.ends_with(".box") || 
        s.ends_with(".com") || s.ends_with(".xyz") || 
        s.ends_with(".org") || s.ends_with(".io") || 
        s.ends_with(".dev") || s.ends_with(".app")
    )
}

/// Validate ENS name format according to DNS rules
fn validate_ens_name(name: &str) -> Result<(), ConfigError> {
    if !is_ens_name(name) {
        return Err(ConfigError::InvalidIdentity(
            format!("invalid ENS name format: {}", name)
        ));
    }
    
    if name.len() > 253 {
        return Err(ConfigError::InvalidIdentity(
            "ENS name too long (max 253 chars)".to_string()
        ));
    }
    
    for label in name.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err(ConfigError::InvalidIdentity(
                "invalid ENS label length".to_string()
            ));
        }
        
        if label.starts_with('-') || label.ends_with('-') {
            return Err(ConfigError::InvalidIdentity(
                "ENS labels cannot start/end with hyphen".to_string()
            ));
        }
        
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ConfigError::InvalidIdentity(
                "invalid characters in ENS name".to_string()
            ));
        }
    }
    
    Ok(())
}

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
