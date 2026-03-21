use std::collections::HashMap;

/// A parsed .repobox/config.yml file.
#[derive(Debug, Clone)]
pub struct Config {
    pub groups: HashMap<String, Group>,
    pub permissions: Permissions,
    pub x402: Option<X402Config>,
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

/// x402 payment configuration for paid repository access.
#[derive(Debug, Clone)]
pub struct X402Config {
    /// Price in USDC for permanent read access (e.g. "1.00").
    pub read_price: String,
    /// Payment recipient address (e.g. "0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048").
    pub recipient: String,
    /// Blockchain network for payments (e.g. "base").
    pub network: String,
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

    /// Parse an identity string, trying ENS resolution if it's not already an EVM address.
    /// For ENS names like "vitalik.eth", resolves to the corresponding address.
    pub fn parse_with_ens(s: &str) -> Result<Self, ConfigError> {
        // First try regular parsing
        if let Ok(identity) = Self::parse(s) {
            return Ok(identity);
        }

        // If that fails and it looks like an ENS name, try resolving it
        if is_ens_name(s) {
            match resolve_ens_name_sync(s) {
                Ok(address) => Ok(Identity {
                    kind: IdentityKind::Evm,
                    address,
                }),
                Err(e) => Err(ConfigError::InvalidIdentity(format!("{} (ENS resolution failed: {})", s, e))),
            }
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
    // File verbs
    Edit,
    Write,
    Append,
    Create,
}

impl Verb {
    pub fn parse(s: &str) -> Result<Self, ConfigError> {
        match s {
            "read" => Ok(Verb::Read),
            "push" => Ok(Verb::Push),
            "merge" => Ok(Verb::Merge),
            "branch" => Ok(Verb::Branch),
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
            Verb::Push | Verb::Merge | Verb::Branch | Verb::Delete | Verb::ForcePush
        )
    }

    pub fn is_file_verb(self) -> bool {
        matches!(self, Verb::Edit | Verb::Write | Verb::Append | Verb::Create)
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

/// Check if a string looks like an ENS name
fn is_ens_name(s: &str) -> bool {
    s.contains('.') && (
        s.ends_with(".eth") || 
        s.ends_with(".box") ||
        s.ends_with(".com") || 
        s.ends_with(".xyz") ||
        s.ends_with(".org") || 
        s.ends_with(".io") ||
        s.ends_with(".dev") || 
        s.ends_with(".app")
    )
}

/// Synchronous ENS name resolution using ureq
fn resolve_ens_name_sync(name: &str) -> Result<String, String> {
    use sha3::Digest;
    
    // Check if it's already an address (0x followed by 40 hex chars)
    if name.starts_with("0x") && name.len() == 42 && name[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(name.to_lowercase());
    }

    // Get API key from environment
    let api_key = std::env::var("ALCHEMY_API_KEY")
        .map_err(|_| "ALCHEMY_API_KEY environment variable not set - needed for ENS resolution".to_string())?;

    let rpc_url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", api_key);

    // Use ENS Public Resolver directly for addr() calls
    let public_resolver = "0x231b0Ee14048e9dCcD1d247744d114a4EB5E8E63"; // ENS Public Resolver
    
    let name_hash = namehash_ens(name);
    let addr_selector = &sha3::Keccak256::digest(b"addr(bytes32)")[..4];
    
    // Call addr(bytes32) on the resolver
    let calldata = format!("0x{}{:0>64}", hex::encode(addr_selector), hex::encode(name_hash));

    // Build eth_call JSON-RPC request
    let rpc_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_call",
        "params": [{
            "to": public_resolver,
            "data": calldata,
        }, "latest"]
    });

    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(10))
        .build();

    let rpc_body_str = serde_json::to_string(&rpc_body)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;

    let response_body = agent
        .post(&rpc_url)
        .set("Content-Type", "application/json")
        .send_string(&rpc_body_str)
        .map_err(|e| format!("RPC request failed: {}", e))?
        .into_string()
        .map_err(|e| format!("RPC response read error: {}", e))?;

    let rpc_json: serde_json::Value = serde_json::from_str(&response_body)
        .map_err(|e| format!("RPC response parse error: {}", e))?;

    // Check for RPC error
    if let Some(error) = rpc_json.get("error") {
        return Err(format!("RPC error: {}", error));
    }

    // Parse result as an address (32 bytes, address is in the last 20 bytes)
    let result_hex = rpc_json
        .get("result")
        .and_then(|v: &serde_json::Value| v.as_str())
        .ok_or_else(|| "No result in RPC response".to_string())?;

    if result_hex.len() < 66 { // 0x + 64 hex chars (32 bytes)
        return Err("Invalid response length".to_string());
    }

    // Extract address from last 20 bytes of the 32-byte response
    let addr_hex = &result_hex[result_hex.len() - 40..];
    
    // Check if all bytes are zero (no address set)
    if addr_hex.chars().all(|c| c == '0') {
        return Err(format!("ENS name '{}' does not resolve to an address", name));
    }

    Ok(format!("0x{}", addr_hex.to_lowercase()))
}

/// Calculate ENS namehash for a domain name
fn namehash_ens(name: &str) -> [u8; 32] {
    use sha3::Digest;
    
    let mut node = [0u8; 32];

    if name.is_empty() {
        return node;
    }

    let labels: Vec<&str> = name.split('.').collect();
    for label in labels.iter().rev() {
        let label_hash = sha3::Keccak256::digest(label.as_bytes());
        let mut hasher = sha3::Keccak256::new();
        hasher.update(node);
        hasher.update(label_hash);
        node = hasher.finalize().into();
    }

    node
}
