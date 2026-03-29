use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Structured error system for consistent CLI error messages and agent-parseable output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliError {
    pub code: String,
    pub message: String,
    pub context: HashMap<String, String>,
    pub next_action: String,
}

impl CliError {
    /// Create a new structured error
    pub fn new(code: &str, message: &str, next_action: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            context: HashMap::new(),
            next_action: next_action.to_string(),
        }
    }

    /// Add context information to the error
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    /// Format error for human-readable output
    pub fn format_human(&self) -> String {
        let mut output = format!("Error: [{}] {}", self.code, self.message);
        output.push_str(&format!(" | Next: {}", self.next_action));
        
        if !self.context.is_empty() {
            let context_items: Vec<String> = self.context
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            output.push_str(&format!(" | Context: {}", context_items.join(", ")));
        }
        
        output
    }

    /// Format error as JSON for agent consumption
    pub fn format_json(&self) -> String {
        let json_output = serde_json::json!({
            "error": {
                "code": self.code,
                "message": self.message,
                "context": self.context,
                "nextAction": self.next_action
            }
        });
        
        serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| {
            format!(r#"{{"error": {{"code": "{}", "message": "JSON serialization failed"}}}}"#, self.code)
        })
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_human())
    }
}

/// Error taxonomy for repo.box CLI
pub struct ErrorCodes;

impl ErrorCodes {
    // Authentication and identity errors
    pub const AUTH_NO_IDENTITY: &'static str = "AUTH_001";
    pub const AUTH_INVALID_IDENTITY: &'static str = "AUTH_002";
    pub const AUTH_NO_KEY: &'static str = "AUTH_003";
    pub const AUTH_PERMISSION_DENIED: &'static str = "AUTH_004";

    // Configuration and setup errors
    pub const CONFIG_NOT_FOUND: &'static str = "CONFIG_001";
    pub const CONFIG_INVALID: &'static str = "CONFIG_002";
    pub const CONFIG_SETUP_CONFLICT: &'static str = "CONFIG_003";
    pub const CONFIG_ALIAS_NOT_FOUND: &'static str = "CONFIG_004";

    // Repository state errors
    pub const REPO_NOT_GIT: &'static str = "REPO_001";
    pub const REPO_NOT_REPOBOX: &'static str = "REPO_002";
    pub const REPO_INVALID_BRANCH: &'static str = "REPO_003";
    pub const REPO_INVALID_REF: &'static str = "REPO_004";

    // Network and connectivity errors
    pub const NET_CONNECTION_FAILED: &'static str = "NET_001";
    pub const NET_TIMEOUT: &'static str = "NET_002";
    pub const NET_UNAUTHORIZED: &'static str = "NET_003";
    pub const NET_SERVER_ERROR: &'static str = "NET_004";
}

/// Pre-defined error constructors for common scenarios
impl CliError {
    pub fn no_identity() -> Self {
        Self::new(
            ErrorCodes::AUTH_NO_IDENTITY,
            "no identity configured",
            "Generate a key: 'repobox keys generate --alias me' then 'repobox use me'"
        )
    }

    pub fn invalid_identity(identity: &str, details: &str) -> Self {
        Self::new(
            ErrorCodes::AUTH_INVALID_IDENTITY,
            "invalid identity format",
            "Use an alias '@alice', EVM address 'evm:0x1234...', or ENS 'vitalik.eth'"
        )
        .with_context("identity", identity)
        .with_context("details", details)
    }

    pub fn no_key(identity: &str) -> Self {
        Self::new(
            ErrorCodes::AUTH_NO_KEY,
            "no key found for identity",
            "Generate a key: 'repobox keys generate --alias <name>'"
        )
        .with_context("identity", identity)
    }

    pub fn permission_denied(identity: &str, action: &str, target: &str) -> Self {
        Self::new(
            ErrorCodes::AUTH_PERMISSION_DENIED,
            "permission denied",
            "Check permissions: 'repobox check <identity> <action> <target>'"
        )
        .with_context("identity", identity)
        .with_context("action", action)
        .with_context("target", target)
    }

    pub fn config_not_found() -> Self {
        Self::new(
            ErrorCodes::CONFIG_NOT_FOUND,
            "no .repobox/config.yml found",
            "Initialize repo.box: 'repobox init' or navigate to a repo.box-enabled repository"
        )
    }

    pub fn config_invalid(details: &str) -> Self {
        Self::new(
            ErrorCodes::CONFIG_INVALID,
            "invalid configuration",
            "Fix .repobox/config.yml or regenerate: 'repobox init --force'"
        )
        .with_context("details", details)
    }

    pub fn setup_conflict(details: &str) -> Self {
        Self::new(
            ErrorCodes::CONFIG_SETUP_CONFLICT,
            "setup configuration conflict",
            "Use specific setup flags: see 'repobox setup --help'"
        )
        .with_context("details", details)
    }

    pub fn alias_not_found(alias: &str) -> Self {
        Self::new(
            ErrorCodes::CONFIG_ALIAS_NOT_FOUND,
            "unknown alias",
            "Use 'repobox alias list' to see available aliases or create one: 'repobox alias add <name> <address>'"
        )
        .with_context("alias", alias)
    }

    pub fn not_git_repo() -> Self {
        Self::new(
            ErrorCodes::REPO_NOT_GIT,
            "not in a git repository",
            "Run 'git init' first or navigate to an existing git repository"
        )
    }

    pub fn not_repobox_repo() -> Self {
        Self::new(
            ErrorCodes::REPO_NOT_REPOBOX,
            "not a repo.box repository",
            "Initialize repo.box: 'repobox init'"
        )
    }
}

/// Global flag for JSON output mode
pub static mut JSON_OUTPUT: bool = false;

/// Print error using appropriate format (human-readable or JSON)
pub fn print_error(error: &CliError) {
    unsafe {
        if JSON_OUTPUT {
            eprintln!("{}", error.format_json());
        } else {
            eprintln!("{}", error.format_human());
        }
    }
}

/// Set global JSON output mode
pub fn set_json_output(enabled: bool) {
    unsafe {
        JSON_OUTPUT = enabled;
    }
}