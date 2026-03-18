use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::{Parser, Subcommand};

use repobox::aliases;
use repobox::config::{Identity, Verb};
use repobox::engine;
use repobox::identity;
use repobox::parser;
use repobox::shim::{self, ShimAction};

#[derive(Parser)]
#[command(name = "repobox", about = "Git permission layer for AI agents")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Raw git args when invoked as a shim
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    git_args: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize repo.box in the current git repo
    Init {
        /// Overwrite existing .repobox-config
        #[arg(long)]
        force: bool,
    },
    /// Manage EVM key pairs
    Keys {
        #[command(subcommand)]
        action: KeysAction,
    },
    /// Set or manage your identity
    Identity {
        #[command(subcommand)]
        action: IdentityAction,
    },
    /// Set identity by alias or address (shorthand for identity set)
    Use {
        /// Alias name (e.g. fran-test) or evm:0x... address
        name: String,
    },
    /// Show current identity
    Whoami,
    /// Manage local aliases
    Alias {
        #[command(subcommand)]
        action: AliasAction,
    },
    /// Check if an identity can perform an action
    Check {
        /// Identity or alias (e.g. @alice, evm:0x...)
        identity: String,
        /// Action verb (push, merge, edit, etc.)
        verb: String,
        /// Target (>main, contracts/**, etc.)
        target: String,
    },
    /// Validate .repobox-config
    Lint,
}

#[derive(Subcommand)]
enum KeysAction {
    /// Generate a new EVM key pair
    Generate {
        #[arg(long)]
        alias: Option<String>,
    },
    /// Import an existing private key
    Import {
        /// Hex-encoded private key
        key: String,
        #[arg(long)]
        alias: Option<String>,
    },
    /// List all stored keys
    List,
}

#[derive(Subcommand)]
enum IdentityAction {
    /// Set your active identity
    Set {
        /// Hex-encoded private key
        key: String,
        #[arg(long)]
        alias: Option<String>,
    },
}

#[derive(Subcommand)]
enum AliasAction {
    /// Add or update an alias
    Add {
        name: String,
        address: String,
    },
    /// Remove an alias
    Remove {
        name: String,
    },
    /// List all aliases
    List,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let home = home_dir();

    match cli.command {
        Some(Commands::Init { force }) => cmd_init(force),
        Some(Commands::Keys { action }) => cmd_keys(action, &home),
        Some(Commands::Identity { action }) => cmd_identity(action, &home),
        Some(Commands::Use { name }) => cmd_use(&name, &home),
        Some(Commands::Whoami) => cmd_whoami(&home),
        Some(Commands::Alias { action }) => cmd_alias(action, &home),
        Some(Commands::Check { identity: id_str, verb, target }) => {
            cmd_check(&id_str, &verb, &target, &home)
        }
        Some(Commands::Lint) => cmd_lint(),
        None => {
            // No subcommand → shim mode (intercept git commands)
            cmd_shim(&cli.git_args, &home)
        }
    }
}

// ── Init ──────────────────────────────────────────────────────────────

fn cmd_init(force: bool) -> ExitCode {
    // Check we're in a git repo
    let git_check = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output();

    match git_check {
        Ok(out) if out.status.success() => {}
        _ => {
            eprintln!("error: not a git repository");
            return ExitCode::FAILURE;
        }
    }

    let config_path = Path::new(".repobox-config");
    if config_path.exists() && !force {
        eprintln!("error: .repobox-config already exists. Use --force to overwrite");
        return ExitCode::FAILURE;
    }

    let template = r#"# repo.box configuration
# Docs: https://repo.box/docs

groups:
  founders:
    members: []
      # - evm:0xYOUR_ADDRESS_HERE
  agents:
    members: []
      # - evm:0xAGENT_ADDRESS_HERE

permissions:
  default: allow
  rules: []
    # Flat rules:
    #   - "@founders push >*"
    #   - "@founders merge >*"
    #   - "@founders edit .repobox-config"
    #
    # Nested rules:
    #   - "@agents":
    #       push:
    #         - ">feature/**"
    #         - ">fix/**"
    #       create:
    #         - ">feature/**"
    #       append:
    #         - ".repobox-config"
"#;

    if let Err(e) = std::fs::write(config_path, template) {
        eprintln!("error: failed to write .repobox-config: {e}");
        return ExitCode::FAILURE;
    }

    // Store real git path
    let real_git = find_real_git();
    let repobox_dir = Path::new(".repobox");
    let _ = std::fs::create_dir_all(repobox_dir);
    let _ = std::fs::write(repobox_dir.join("config"), format!("git = {real_git}"));

    println!("✅ Initialized repo.box");
    println!("   Created .repobox-config (edit to add groups and rules)");
    println!("   Real git: {real_git}");

    ExitCode::SUCCESS
}

// ── Keys ──────────────────────────────────────────────────────────────

fn cmd_keys(action: KeysAction, home: &Path) -> ExitCode {
    match action {
        KeysAction::Generate { alias } => {
            // Generate random 32-byte private key
            use rand::RngCore;
            let mut key_bytes = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut key_bytes);
            let private_key_hex = format!("0x{}", hex::encode(key_bytes));

            let address = match identity::derive_address(&private_key_hex) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            if let Err(e) = identity::store_key(home, &address, &private_key_hex) {
                eprintln!("error storing key: {e}");
                return ExitCode::FAILURE;
            }

            let identity_str = format!("evm:{address}");

            if let Some(alias_name) = &alias {
                if let Err(e) = aliases::set_alias(home, alias_name, &identity_str) {
                    eprintln!("warning: failed to set alias: {e}");
                }
            }

            // Auto-set as current identity
            if let Err(e) = identity::set_identity(home, &address) {
                eprintln!("warning: failed to set identity: {e}");
            }
            let _ = Command::new("git")
                .args(["config", "--local", "user.signingkey", &identity_str])
                .output();

            let display = match &alias {
                Some(a) => format!("@{a} ({identity_str})"),
                None => identity_str,
            };
            println!("🔑 Generated: {display}");
            println!("   Key stored in ~/.repobox/keys/{address}.key");
            println!("   ✅ Set as current identity");

            ExitCode::SUCCESS
        }
        KeysAction::Import { key, alias } => {
            let address = match identity::derive_address(&key) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            if let Err(e) = identity::store_key(home, &address, &key) {
                eprintln!("error storing key: {e}");
                return ExitCode::FAILURE;
            }

            let identity_str = format!("evm:{address}");

            if let Some(alias_name) = &alias {
                if let Err(e) = aliases::set_alias(home, alias_name, &identity_str) {
                    eprintln!("warning: failed to set alias: {e}");
                }
            }

            let display = match &alias {
                Some(a) => format!("@{a} ({identity_str})"),
                None => identity_str,
            };
            println!("🔑 Imported: {display}");

            ExitCode::SUCCESS
        }
        KeysAction::List => {
            let keys_dir = identity::repobox_home_with_base(home).join("keys");
            match std::fs::read_dir(&keys_dir) {
                Ok(entries) => {
                    let mut found = false;
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name = name.to_string_lossy();
                        if name.ends_with(".key") {
                            let addr = name.trim_end_matches(".key");
                            let identity_str = format!("evm:{addr}");
                            let display = aliases::display_identity(home, &identity_str);
                            println!("  {display}");
                            found = true;
                        }
                    }
                    if !found {
                        println!("No keys found. Run: git repobox keys generate");
                    }
                }
                Err(_) => {
                    println!("No keys found. Run: git repobox keys generate");
                }
            }
            ExitCode::SUCCESS
        }
    }
}

// ── Identity ──────────────────────────────────────────────────────────

fn cmd_identity(action: IdentityAction, home: &Path) -> ExitCode {
    match action {
        IdentityAction::Set { key, alias } => {
            let address = match identity::derive_address(&key) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            // Store key if not already stored
            let _ = identity::store_key(home, &address, &key);

            // Set identity
            if let Err(e) = identity::set_identity(home, &address) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }

            let identity_str = format!("evm:{address}");

            // Set git config
            let _ = Command::new("git")
                .args(["config", "--local", "user.signingkey", &identity_str])
                .output();

            if let Some(alias_name) = &alias {
                if let Err(e) = aliases::set_alias(home, alias_name, &identity_str) {
                    eprintln!("warning: failed to set alias: {e}");
                }
            }

            let display = match &alias {
                Some(a) => format!("@{a} ({identity_str})"),
                None => identity_str,
            };
            println!("✅ Identity set: {display}");

            ExitCode::SUCCESS
        }
    }
}

// ── Use (set identity by alias/address) ───────────────────────────────

fn cmd_use(name: &str, home: &Path) -> ExitCode {
    // Strip @ prefix if present
    let name = name.strip_prefix('@').unwrap_or(name);

    // Try resolving as alias first
    let identity_str = match aliases::resolve_alias(home, name) {
        Ok(Some(addr)) => addr,
        _ => {
            // Try as raw identity string
            if name.starts_with("evm:") {
                name.to_string()
            } else {
                eprintln!("error: unknown alias '{name}'. Run: git repobox alias list");
                return ExitCode::FAILURE;
            }
        }
    };

    // Extract address from identity string
    let address = identity_str.strip_prefix("evm:").unwrap_or(&identity_str);

    // Check we have the key
    let key_path = identity::repobox_home_with_base(home)
        .join("keys")
        .join(format!("{address}.key"));
    if !key_path.exists() {
        eprintln!("error: no key found for {identity_str}. Run: git repobox keys generate");
        return ExitCode::FAILURE;
    }

    // Set identity
    if let Err(e) = identity::set_identity(home, address) {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }

    // Set git config
    let _ = Command::new("git")
        .args(["config", "--local", "user.signingkey", &identity_str])
        .output();

    let display = aliases::display_identity(home, &identity_str);
    println!("✅ Now using: {display}");

    ExitCode::SUCCESS
}

// ── Whoami ────────────────────────────────────────────────────────────

fn cmd_whoami(home: &Path) -> ExitCode {
    match identity::get_identity(home) {
        Ok(Some(id)) => {
            let id_str = id.to_string();
            let display = aliases::display_identity(home, &id_str);
            println!("{display}");
            ExitCode::SUCCESS
        }
        Ok(None) => {
            eprintln!("no identity configured. Run: git repobox identity set <private-key>");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

// ── Alias ─────────────────────────────────────────────────────────────

fn cmd_alias(action: AliasAction, home: &Path) -> ExitCode {
    match action {
        AliasAction::Add { name, address } => {
            if let Err(e) = aliases::set_alias(home, &name, &address) {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
            println!("✅ @{name} → {address}");
            ExitCode::SUCCESS
        }
        AliasAction::Remove { name } => {
            match aliases::remove_alias(home, &name) {
                Ok(true) => {
                    println!("✅ Removed @{name}");
                    ExitCode::SUCCESS
                }
                Ok(false) => {
                    eprintln!("alias not found: {name}");
                    ExitCode::FAILURE
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        AliasAction::List => {
            let map = aliases::read_aliases(home);
            if map.is_empty() {
                println!("No aliases. Run: git repobox alias add <name> <address>");
            } else {
                let mut entries: Vec<_> = map.into_iter().collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                for (name, addr) in entries {
                    println!("  @{name} = {addr}");
                }
            }
            ExitCode::SUCCESS
        }
    }
}

// ── Check ─────────────────────────────────────────────────────────────

fn cmd_check(id_str: &str, verb_str: &str, target_str: &str, home: &Path) -> ExitCode {
    let config_path = Path::new(".repobox-config");
    if !config_path.exists() {
        eprintln!("error: no .repobox-config found");
        return ExitCode::FAILURE;
    }

    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error reading .repobox-config: {e}");
            return ExitCode::FAILURE;
        }
    };

    let config = match parser::parse(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Resolve alias to address
    let resolved = if id_str.starts_with('@') {
        let name = &id_str[1..];
        match aliases::resolve_alias(home, name) {
            Ok(Some(addr)) => addr,
            _ => {
                eprintln!("error: unknown alias: {id_str}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        id_str.to_string()
    };

    let identity = match Identity::parse(&resolved) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let verb = match Verb::parse(verb_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let target = repobox::config::Target::parse(target_str).unwrap();

    let result = engine::check(
        &config,
        &identity,
        verb,
        target.branch.as_deref(),
        target.path.as_deref(),
    );

    let display = aliases::display_identity(home, &resolved);

    if result.is_allowed() {
        println!("✅ allowed — {display} {verb_str} {target_str}");
        match result {
            engine::CheckResult::Allow { rule_line } => {
                println!("   matched rule {rule_line}");
            }
            engine::CheckResult::Default { .. } => {
                println!("   (no rules for this verb+target → default: allow)");
            }
            _ => {}
        }
        ExitCode::SUCCESS
    } else {
        println!("❌ denied — {display} {verb_str} {target_str}");
        match result {
            engine::CheckResult::Deny { rule_line: _, reason } => {
                println!("   {reason}");
            }
            engine::CheckResult::ImplicitDeny { verb } => {
                println!("   implicit deny: rules exist for '{verb}', no match for this identity");
            }
            engine::CheckResult::Default { .. } => {
                println!("   (default: deny)");
            }
            _ => {}
        }
        ExitCode::FAILURE
    }
}

// ── Lint ──────────────────────────────────────────────────────────────

fn cmd_lint() -> ExitCode {
    let config_path = Path::new(".repobox-config");
    if !config_path.exists() {
        eprintln!("error: no .repobox-config found");
        return ExitCode::FAILURE;
    }

    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error reading .repobox-config: {e}");
            return ExitCode::FAILURE;
        }
    };

    match parser::parse(&content) {
        Ok(config) => {
            let n_groups = config.groups.len();
            let n_rules = config.permissions.rules.len();
            println!("✅ .repobox-config is valid");
            println!("   {n_groups} groups, {n_rules} rules, default: {:?}", config.permissions.default);

            // Warn about potential issues
            // Check for broader allow above narrower deny for same subject
            for (i, rule) in config.permissions.rules.iter().enumerate() {
                if !rule.deny {
                    // Look for deny rules below this one for the same subject
                    for later in &config.permissions.rules[i + 1..] {
                        if later.deny && later.verb == rule.verb {
                            if let (Subject::Group(a), Subject::Group(b)) =
                                (&rule.subject, &later.subject)
                            {
                                if a == b {
                                    println!(
                                        "   ⚠️  warning: deny rule for @{b} {} is shadowed by allow rule above (line {})",
                                        later.verb, rule.line
                                    );
                                }
                            }
                        }
                    }
                }
            }

            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("❌ {e}");
            ExitCode::FAILURE
        }
    }
}

// ── Shim Mode ─────────────────────────────────────────────────────────

fn cmd_shim(args: &[String], home: &Path) -> ExitCode {
    if args.is_empty() {
        // No args → show help
        eprintln!("repobox — Git permission layer for AI agents");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("  git repobox init          Initialize repo.box");
        eprintln!("  git repobox keys generate Generate a new key pair");
        eprintln!("  git repobox identity set  Set your identity");
        eprintln!("  git repobox whoami        Show current identity");
        eprintln!("  git repobox alias         Manage aliases");
        eprintln!("  git repobox check         Check permissions");
        eprintln!("  git repobox lint          Validate config");
        return ExitCode::SUCCESS;
    }

    // Determine repo root
    let repo_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
            } else {
                None
            }
        });

    // Read identity from git config (user.signingkey)
    let identity = read_identity_from_git_config()
        .or_else(|| identity::get_identity(home).ok().flatten());

    // Get current branch
    let current_branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    let action = shim::process_command(
        args,
        repo_root.as_deref(),
        identity.as_ref(),
        current_branch.as_deref(),
    );

    match action {
        ShimAction::Passthrough | ShimAction::Delegate => {
            // Run real git with the original args
            let real_git = find_real_git();
            let status = Command::new(&real_git)
                .args(args)
                .status()
                .unwrap_or_else(|e| {
                    eprintln!("error: failed to run git: {e}");
                    std::process::exit(1);
                });

            if status.success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        ShimAction::Block(msg) => {
            // Resolve aliases in the error message
            let display_msg = enhance_error_message(&msg, home);
            eprintln!("❌ {display_msg}");
            ExitCode::FAILURE
        }
        ShimAction::RepoboxCommand => {
            // Re-parse as repobox subcommand
            let cli = Cli::parse_from(
                std::iter::once("repobox".to_string()).chain(args[1..].iter().cloned()),
            );
            let home = home_dir();
            match cli.command {
                Some(Commands::Init { force }) => cmd_init(force),
                Some(Commands::Keys { action }) => cmd_keys(action, &home),
                Some(Commands::Identity { action }) => cmd_identity(action, &home),
                Some(Commands::Use { name }) => cmd_use(&name, &home),
                Some(Commands::Whoami) => cmd_whoami(&home),
                Some(Commands::Alias { action }) => cmd_alias(action, &home),
                Some(Commands::Check { identity: id_str, verb, target }) => {
                    cmd_check(&id_str, &verb, &target, &home)
                }
                Some(Commands::Lint) => cmd_lint(),
                None => {
                    eprintln!("Unknown repobox command. Run: git repobox --help");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────

fn home_dir() -> PathBuf {
    std::env::var("REPOBOX_HOME")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("HOME").map(PathBuf::from))
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

fn find_real_git() -> String {
    // Try .repobox/config first
    if let Ok(content) = std::fs::read_to_string(".repobox/config") {
        for line in content.lines() {
            if let Some(path) = line.strip_prefix("git = ") {
                if Path::new(path.trim()).exists() {
                    return path.trim().to_string();
                }
            }
        }
    }

    // Fall back to finding git in PATH (skipping ourselves)
    let our_path = std::env::current_exe().ok();
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            let candidate = PathBuf::from(dir).join("git");
            if candidate.exists() {
                if let Some(ref ours) = our_path {
                    if candidate == *ours {
                        continue;
                    }
                }
                return candidate.to_string_lossy().to_string();
            }
        }
    }

    "git".to_string()
}

fn read_identity_from_git_config() -> Option<Identity> {
    let output = Command::new("git")
        .args(["config", "user.signingkey"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let key_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Identity::parse(&key_str).ok()
}

fn enhance_error_message(msg: &str, home: &Path) -> String {
    // Try to replace raw evm addresses with aliases
    let aliases = aliases::read_aliases(home);
    let mut result = msg.to_string();
    for (name, addr) in &aliases {
        result = result.replace(addr.as_str(), &format!("@{name} ({addr})"));
    }
    result
}

use repobox::config::Subject;
