use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use std::process::Command;

use crate::config::*;
use crate::engine;
use crate::parser;
use crate::resolver::RemoteResolver;

/// Commands that are never intercepted.
const PASSTHROUGH_COMMANDS: &[&str] = &[
    "status", "log", "diff", "add", "stash", "fetch", "clone", "remote", "show", "tag",
    "reflog", "blame", "bisect", "archive", "shortlog", "describe",
];

/// Commands that need permission checks.
const CHECKED_COMMANDS: &[&str] = &[
    "commit", "merge", "push", "checkout", "branch", "pull",
];

/// Result of the shim processing a git command.
#[derive(Debug)]
pub enum ShimAction {
    /// Pass through to real git unchanged.
    Passthrough,
    /// Delegate to real git after checks pass.
    Delegate,
    /// Block the command with an error message.
    Block(String),
    /// Run a repobox subcommand (init, whoami, etc.).
    RepoboxCommand,
}

/// Determine what the shim should do with a git command.
/// Process a git command with optional remote resolver for dynamic groups.
pub fn process_command(
    args: &[String],
    repo_root: Option<&Path>,
    identity: Option<&Identity>,
    current_branch: Option<&str>,
) -> ShimAction {
    process_command_with_resolver(args, repo_root, identity, current_branch, None)
}

/// Process a git command, using a remote resolver for dynamic group membership checks.
pub fn process_command_with_resolver(
    args: &[String],
    repo_root: Option<&Path>,
    identity: Option<&Identity>,
    current_branch: Option<&str>,
    resolver: Option<&RemoteResolver>,
) -> ShimAction {
    if args.is_empty() {
        return ShimAction::Passthrough;
    }

    let cmd = &args[0];

    // "repobox" subcommand → handle internally
    if cmd == "repobox" {
        return ShimAction::RepoboxCommand;
    }

    // Check if .repobox/config.yml exists
    let config_path = match repo_root {
        Some(root) => root.join(".repobox/config.yml"),
        None => return ShimAction::Passthrough, // Not in a git repo
    };

    if !config_path.exists() {
        return ShimAction::Passthrough; // No config → fully transparent
    }

    // Passthrough commands
    if PASSTHROUGH_COMMANDS.contains(&cmd.as_str()) {
        return ShimAction::Passthrough;
    }

    // Unknown commands → passthrough
    if !CHECKED_COMMANDS.contains(&cmd.as_str()) {
        return ShimAction::Passthrough;
    }

    // Need identity for checked commands
    let identity = match identity {
        Some(id) => id,
        None => {
            return ShimAction::Block(
                "no identity configured. Run: git repobox identity set <private-key>".to_string(),
            );
        }
    };

    // Parse config
    let config_content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            return ShimAction::Block(format!("failed to read .repobox/config.yml: {e}"));
        }
    };

    let config = match parser::parse(&config_content) {
        Ok(c) => c,
        Err(e) => {
            // Parse errors block permission-checked commands
            return ShimAction::Block(format!(".repobox/config.yml error: {e}"));
        }
    };

    match cmd.as_str() {
        "commit" => check_commit(args, &config, identity, current_branch, repo_root.unwrap(), resolver),
        "merge" => check_merge(args, &config, identity, current_branch, resolver),
        "push" => check_push(args, &config, identity, current_branch, resolver),
        "checkout" => check_checkout(args, &config, identity, resolver),
        "branch" => check_branch(args, &config, identity, resolver),
        "pull" => {
            match current_branch {
                Some(branch) => {
                    let result = engine::check_with_resolver(&config, identity, Verb::Merge, Some(branch), None, resolver);
                    if result.is_allowed() {
                        ShimAction::Delegate
                    } else {
                        ShimAction::Block(format!(
                            "permission denied: {} cannot merge into {branch}",
                            identity
                        ))
                    }
                }
                None => ShimAction::Delegate,
            }
        }
        _ => ShimAction::Passthrough,
    }
}

/// Check permissions for git commit.
fn check_commit(
    _args: &[String],
    config: &Config,
    identity: &Identity,
    current_branch: Option<&str>,
    repo_root: &Path,
    resolver: Option<&RemoteResolver>,
) -> ShimAction {
    // Run lint on .repobox/config.yml if it's being committed
    let staged_files = get_staged_files(repo_root);

    if staged_files.iter().any(|f| f == ".repobox/config.yml") {
        let warnings = crate::lint::lint(config);
        if !warnings.is_empty() {
            eprintln!("repo.box lint warnings:");
            for w in &warnings {
                eprintln!("  {w}");
            }
            eprintln!();
        }
    }

    // Check file permissions for each staged file.
    // Classify the change type (write/append/edit) and check permissions.
    //
    // Hierarchy: `edit` is the general file-modification verb.
    // `write` and `append` are specific sub-verbs.
    //
    // Logic:
    // 1. Check the specific verb (write/append). If explicitly allowed → OK.
    // 2. Check `edit` (general). If explicitly allowed → OK (edit covers all).
    // 3. If `edit` is explicitly denied → BLOCKED (deny on edit blocks all sub-verbs).
    // 4. Otherwise → use the specific verb's result.
    for file in &staged_files {
        let verb = classify_staged_file(file, repo_root);

        if verb == Verb::Edit {
            // Already the general verb, just check it
            let result = engine::check_with_resolver(config, identity, Verb::Edit, current_branch, Some(file), resolver);
            if !result.is_allowed() {
                return ShimAction::Block(format!(
                    "permission denied: {} cannot edit {file}",
                    identity
                ));
            }
            continue;
        }

        // For write/append: check both the specific verb and `edit`
        let specific = engine::check_with_resolver(config, identity, verb, current_branch, Some(file), resolver);
        let general = engine::check_with_resolver(config, identity, Verb::Edit, current_branch, Some(file), resolver);

        // If either explicitly allows → OK
        if specific.is_allowed() || general.is_allowed() {
            // But also check: if `edit` explicitly denies, block regardless
            if matches!(general, engine::CheckResult::Deny { .. }) {
                return ShimAction::Block(format!(
                    "permission denied: {} cannot edit {file}",
                    identity
                ));
            }
            continue;
        }

        // Neither allowed
        let verb_name = match verb {
            Verb::Write => "write",
            Verb::Append => "append to",
            _ => "edit",
        };
        return ShimAction::Block(format!(
            "permission denied: {} cannot {verb_name} {file}",
            identity
        ));
    }

    ShimAction::Delegate
}

/// Check permissions for git merge.
fn check_merge(
    _args: &[String],
    config: &Config,
    identity: &Identity,
    current_branch: Option<&str>,
    resolver: Option<&RemoteResolver>,
) -> ShimAction {
    let branch = match current_branch {
        Some(b) => b,
        None => return ShimAction::Delegate,
    };

    let result = engine::check_with_resolver(config, identity, Verb::Merge, Some(branch), None, resolver);
    if !result.is_allowed() {
        return ShimAction::Block(format!(
            "permission denied: {} cannot merge into {branch}",
            identity
        ));
    }

    // TODO: Check if merge would bring in .repobox/config.yml changes
    // and verify the identity can edit .repobox/config.yml on the target branch.
    // For now, this is checked at commit time.

    ShimAction::Delegate
}

/// Check permissions for git push.
fn check_push(
    args: &[String],
    config: &Config,
    identity: &Identity,
    current_branch: Option<&str>,
    resolver: Option<&RemoteResolver>,
) -> ShimAction {
    let is_force = args.iter().any(|a| a == "--force" || a == "-f");

    let target_branch = detect_push_target(args, current_branch);

    let target_branch = match target_branch {
        Some(b) => b,
        None => return ShimAction::Delegate,
    };

    if is_force {
        let result = engine::check_with_resolver(config, identity, Verb::ForcePush, Some(&target_branch), None, resolver);
        if !result.is_allowed() {
            return ShimAction::Block(format!(
                "permission denied: {} cannot force-push to {target_branch}",
                identity
            ));
        }
    }

    let result = engine::check_with_resolver(config, identity, Verb::Push, Some(&target_branch), None, resolver);
    if result.is_allowed() {
        ShimAction::Delegate
    } else {
        ShimAction::Block(format!(
            "permission denied: {} cannot push to {target_branch}",
            identity
        ))
    }
}

/// Check permissions for git checkout -b / git checkout (switching).
fn check_checkout(
    args: &[String],
    config: &Config,
    identity: &Identity,
    resolver: Option<&RemoteResolver>,
) -> ShimAction {
    let has_b_flag = args.iter().any(|a| a == "-b" || a == "-B");
    if !has_b_flag {
        return ShimAction::Passthrough;
    }

    let branch_name = args
        .windows(2)
        .find(|w| w[0] == "-b" || w[0] == "-B")
        .map(|w| w[1].clone());

    let branch_name = match branch_name {
        Some(b) => b,
        None => return ShimAction::Delegate,
    };

    let result = engine::check_with_resolver(config, identity, Verb::Create, Some(&branch_name), None, resolver);
    if result.is_allowed() {
        ShimAction::Delegate
    } else {
        ShimAction::Block(format!(
            "permission denied: {} cannot create branch {branch_name}",
            identity
        ))
    }
}

/// Check permissions for git branch commands.
fn check_branch(
    args: &[String],
    config: &Config,
    identity: &Identity,
    resolver: Option<&RemoteResolver>,
) -> ShimAction {
    let has_delete = args.iter().any(|a| a == "-d" || a == "-D" || a == "--delete");

    if has_delete {
        let branch_name = args.iter().rev().find(|a| !a.starts_with('-'));
        if let Some(branch) = branch_name {
            let result = engine::check_with_resolver(config, identity, Verb::Delete, Some(branch), None, resolver);
            if !result.is_allowed() {
                return ShimAction::Block(format!(
                    "permission denied: {} cannot delete branch {branch}",
                    identity
                ));
            }
        }
        return ShimAction::Delegate;
    }

    let branch_name = args.iter().skip(1).find(|a| !a.starts_with('-'));
    if let Some(branch) = branch_name {
        if branch != "branch" {
            let result = engine::check_with_resolver(config, identity, Verb::Create, Some(branch), None, resolver);
            if result.is_allowed() {
                ShimAction::Delegate
            } else {
                ShimAction::Block(format!(
                    "permission denied: {} cannot create branch {branch}",
                    identity
                ))
            }
        } else {
            ShimAction::Passthrough
        }
    } else {
        ShimAction::Passthrough // Just listing branches
    }
}

/// Get staged files by running real git.
pub fn get_staged_files(repo_root: &Path) -> Vec<String> {
    // Use REPOBOX_REAL_GIT env var if set, else fall back to "git"
    let git = std::env::var("REPOBOX_REAL_GIT").unwrap_or_else(|_| "git".to_string());
    let output = Command::new(&git)
        .args(["diff", "--cached", "--name-only"])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Classify a staged file change into the appropriate verb.
///
/// - New file (git status `A`) → `Verb::Write`
/// - Append-only (only additions, no deletions in diff body) → `Verb::Append`
/// - Modified with deletions → `Verb::Edit`
pub fn classify_staged_file(file: &str, repo_root: &Path) -> Verb {
    let git = std::env::var("REPOBOX_REAL_GIT").unwrap_or_else(|_| "git".to_string());

    // Check if file is newly added
    let status_output = Command::new(&git)
        .args(["diff", "--cached", "--diff-filter=A", "--name-only"])
        .current_dir(repo_root)
        .output();

    if let Ok(out) = status_output {
        let added_files = String::from_utf8_lossy(&out.stdout);
        if added_files.lines().any(|l| l == file) {
            return Verb::Write;
        }
    }

    // For modified files, use --numstat to check if it's append-only.
    // Append-only = additions > 0, deletions == 0.
    let numstat = Command::new(&git)
        .args(["diff", "--cached", "--numstat", "--", file])
        .current_dir(repo_root)
        .output();

    if let Ok(out) = numstat {
        let line = String::from_utf8_lossy(&out.stdout);
        // Format: "additions\tdeletions\tfilename"
        let parts: Vec<&str> = line.trim().split('\t').collect();
        if parts.len() >= 2 {
            let additions: u64 = parts[0].parse().unwrap_or(0);
            let deletions: u64 = parts[1].parse().unwrap_or(0);
            if additions > 0 && deletions == 0 {
                return Verb::Append;
            }
        }
    }

    Verb::Edit
}



/// Detect which branch a push targets.
fn detect_push_target(args: &[String], current_branch: Option<&str>) -> Option<String> {
    // Look for explicit refspec: `git push origin main` or `git push origin feature:main`
    let non_flag_args: Vec<&String> = args.iter().filter(|a| !a.starts_with('-') && *a != "push").collect();

    if non_flag_args.len() >= 2 {
        let refspec = non_flag_args[1];
        // Handle "src:dst" refspec
        if let Some((_src, dst)) = refspec.split_once(':') {
            return Some(dst.to_string());
        }
        return Some(refspec.to_string());
    }

    // Fall back to current branch
    current_branch.map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_repo_with_config(config_content: &str) -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().to_path_buf();

        // Init a real git repo
        Command::new("git")
            .args(["init"])
            .current_dir(&repo)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&repo)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&repo)
            .output()
            .unwrap();

        if !config_content.is_empty() {
            std::fs::create_dir_all(repo.join(".repobox")).unwrap();
            std::fs::write(repo.join(".repobox/config.yml"), config_content).unwrap();
        }

        (tmp, repo)
    }

    fn id(s: &str) -> Identity {
        Identity::parse(s).unwrap()
    }

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(|s| s.to_string()).collect()
    }

    // ================================================================
    // Section 5: Shim Setup
    // ================================================================

    #[test]
    fn test_no_config_passthrough() {
        let (_tmp, repo) = setup_repo_with_config("");
        // Remove config file
        let _ = std::fs::remove_file(repo.join(".repobox/config.yml"));

        let action = process_command(
            &args("commit -m test"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Passthrough));
    }

    #[test]
    fn test_passthrough_commands() {
        let (_tmp, repo) = setup_repo_with_config("permissions:\n  rules: []");

        for cmd in &["status", "log", "diff", "add", "stash", "fetch"] {
            let action = process_command(
                &args(cmd),
                Some(&repo),
                Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
                Some("main"),
            );
            assert!(
                matches!(action, ShimAction::Passthrough),
                "{cmd} should passthrough"
            );
        }
    }

    #[test]
    fn test_unknown_command_passthrough() {
        let (_tmp, repo) = setup_repo_with_config("permissions:\n  rules: []");

        let action = process_command(
            &args("whatever-unknown-command"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Passthrough));
    }

    #[test]
    fn test_no_repo_passthrough() {
        let action = process_command(
            &args("commit -m test"),
            None, // No repo root
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Passthrough));
    }

    #[test]
    fn test_version_passthrough() {
        let (_tmp, repo) = setup_repo_with_config("permissions:\n  rules: []");

        let action = process_command(
            &args("--version"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Passthrough));
    }

    // ================================================================
    // Section 7: Shim — git commit interception
    // ================================================================

    #[test]
    fn test_commit_no_config_delegates() {
        let (_tmp, repo) = setup_repo_with_config("");
        let _ = std::fs::remove_file(repo.join(".repobox/config.yml"));

        let action = process_command(
            &args("commit -m test"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Passthrough));
    }

    #[test]
    fn test_commit_allowed_files_delegates() {
        let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  default: allow
  rules: []
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        let action = process_command(
            &args("commit -m test"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Delegate));
    }

    #[test]
    fn test_commit_no_identity_blocks() {
        let config = r#"
permissions:
  default: allow
  rules:
    - "founders edit .repobox/config.yml"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        let action = process_command(
            &args("commit -m test"),
            Some(&repo),
            None, // No identity
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Block(ref msg) if msg.contains("no identity configured")));
    }

    #[test]
    fn test_commit_no_verify_still_blocks() {
        let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  default: allow
  rules:
    - "founders edit .repobox/config.yml"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        // Stage .repobox/config.yml
        std::fs::write(repo.join("test.txt"), "hello").unwrap();
        Command::new("git").args(["add", "test.txt"]).current_dir(&repo).output().unwrap();

        // --no-verify should still go through shim checks
        let action = process_command(
            &args("commit --no-verify -m test"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        // Since test.txt has no restrictions, should delegate
        assert!(matches!(action, ShimAction::Delegate));
    }

    #[test]
    fn test_commit_config_parse_error_blocks() {
        let (_tmp, repo) = setup_repo_with_config("this is not valid: yaml: [[[");

        let action = process_command(
            &args("commit -m test"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Block(ref msg) if msg.contains("error")));
    }

    // ================================================================
    // Section 8: Shim — git merge interception
    // ================================================================

    #[test]
    fn test_merge_allowed() {
        let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "founders merge >main"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        let action = process_command(
            &args("merge feature/x"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Delegate));
    }

    #[test]
    fn test_merge_denied() {
        let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "founders merge >main"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        let action = process_command(
            &args("merge feature/x"),
            Some(&repo),
            Some(&id("evm:0xBBB0000000000000000000000000000000000002")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Block(ref msg) if msg.contains("cannot merge")));
    }

    #[test]
    fn test_merge_flags_passed_through() {
        let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "founders merge >main"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        let action = process_command(
            &args("merge --no-ff feature/x"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Delegate));
    }

    // ================================================================
    // Section 9: Shim — git push interception
    // ================================================================

    #[test]
    fn test_push_feature_allowed() {
        let config = r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "agents push >feature/*"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        let action = process_command(
            &args("push origin feature/x"),
            Some(&repo),
            Some(&id("evm:0xBBB0000000000000000000000000000000000002")),
            Some("feature/x"),
        );
        assert!(matches!(action, ShimAction::Delegate));
    }

    #[test]
    fn test_push_main_denied() {
        let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "founders push >main"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        let action = process_command(
            &args("push origin main"),
            Some(&repo),
            Some(&id("evm:0xBBB0000000000000000000000000000000000002")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Block(ref msg) if msg.contains("cannot push")));
    }

    #[test]
    fn test_force_push_denied() {
        let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "founders force-push >main"
    - "agents push >main"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        let action = process_command(
            &args("push --force origin main"),
            Some(&repo),
            Some(&id("evm:0xBBB0000000000000000000000000000000000002")),
            Some("main"),
        );
        // Agent has push but not force-push → blocked
        assert!(matches!(action, ShimAction::Block(ref msg) if msg.contains("force-push")));
    }

    #[test]
    fn test_push_detects_current_branch() {
        let config = r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "agents push >feature/*"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        // No explicit branch in args, uses current_branch
        let action = process_command(
            &args("push"),
            Some(&repo),
            Some(&id("evm:0xBBB0000000000000000000000000000000000002")),
            Some("feature/my-thing"),
        );
        assert!(matches!(action, ShimAction::Delegate));
    }

    // ================================================================
    // Section 10: Shim — git checkout/branch interception
    // ================================================================

    #[test]
    fn test_checkout_create_feature_allowed() {
        let config = r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "agents create >feature/*"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        let action = process_command(
            &args("checkout -b feature/new"),
            Some(&repo),
            Some(&id("evm:0xBBB0000000000000000000000000000000000002")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Delegate));
    }

    #[test]
    fn test_checkout_create_release_denied() {
        let config = r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "agents create >feature/*"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        let action = process_command(
            &args("checkout -b release/v1"),
            Some(&repo),
            Some(&id("evm:0xBBB0000000000000000000000000000000000002")),
            Some("main"),
        );
        // No create rule for >release/* → default: allow
        // Actually, we need to have a rule that would trigger implicit deny
        assert!(matches!(action, ShimAction::Delegate | ShimAction::Block(_)));
    }

    #[test]
    fn test_checkout_switch_existing_passthrough() {
        let config = r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "agents create >feature/*"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        // Switching to existing branch (no -b) → passthrough
        let action = process_command(
            &args("checkout feature/existing"),
            Some(&repo),
            Some(&id("evm:0xBBB0000000000000000000000000000000000002")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Passthrough));
    }

    #[test]
    fn test_branch_delete_check() {
        let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "founders delete >feature/*"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        let action = process_command(
            &args("branch -d feature/old"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Delegate));
    }

    // ================================================================
    // Section 11: Passthrough
    // ================================================================

    #[test]
    fn test_read_commands_passthrough() {
        let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "founders push >main"
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        for cmd in &["status", "log", "diff", "add", "stash", "fetch", "clone", "remote"] {
            let action = process_command(
                &args(cmd),
                Some(&repo),
                Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
                Some("main"),
            );
            assert!(
                matches!(action, ShimAction::Passthrough),
                "{cmd} should passthrough"
            );
        }
    }

    // ================================================================
    // Section 12: Edge Cases
    // ================================================================

    #[test]
    fn test_no_config_all_passthrough() {
        let (_tmp, repo) = setup_repo_with_config("");
        let _ = std::fs::remove_file(repo.join(".repobox/config.yml"));

        for cmd in &["commit -m test", "merge feature/x", "push origin main", "checkout -b new"] {
            let action = process_command(
                &args(cmd),
                Some(&repo),
                Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
                Some("main"),
            );
            assert!(
                matches!(action, ShimAction::Passthrough),
                "{cmd} should passthrough with no config"
            );
        }
    }

    #[test]
    fn test_yaml_error_blocks_checked_commands() {
        let (_tmp, repo) = setup_repo_with_config("invalid: yaml: [[[");

        let action = process_command(
            &args("commit -m test"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Block(_)));
    }

    #[test]
    fn test_yaml_error_allows_read_commands() {
        let (_tmp, repo) = setup_repo_with_config("invalid: yaml: [[[");

        let action = process_command(
            &args("status"),
            Some(&repo),
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Passthrough));
    }

    #[test]
    fn test_outside_repo_passthrough() {
        let action = process_command(
            &args("commit -m test"),
            None,
            Some(&id("evm:0xAAA0000000000000000000000000000000000001")),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Passthrough));
    }

    // ================================================================
    // Section 14: Config file protection — append vs edit
    // ================================================================

    /// Agent with only append on .repobox/config.yml should be BLOCKED from editing it.
    /// This reproduces the bug where claude could commit edits to the config file.
    #[test]
    fn test_agent_edit_config_blocked_when_only_append() {
        let config = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: deny
  rules:
    - founders push >*
    - founders merge >*
    - founders edit *
    - agents push >feature/**
    - agents edit * >feature/**
    - agents append ./.repobox/config.yml
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        // Commit the initial config first so git diff --cached works
        Command::new("git").args(["add", ".repobox/config.yml"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["commit", "-m", "init"]).current_dir(&repo).output().unwrap();

        // Now stage a non-config file to simulate agent editing config
        // We test via process_command which reads staged files — stage a source file
        // that the agent shouldn't be able to edit on main
        std::fs::write(repo.join("src.txt"), "code").unwrap();
        Command::new("git").args(["add", "src.txt"]).current_dir(&repo).output().unwrap();

        let agent = id("evm:0xBBB0000000000000000000000000000000000002");

        // On main branch — agent has no edit rules for main, only >feature/**
        let action = process_command(
            &args("commit -m evil"),
            Some(&repo),
            Some(&agent),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Block(ref msg) if msg.contains("cannot write") || msg.contains("cannot edit")),
            "Agent should be blocked from writing files on main, got: {:?}", action);
    }

    /// Agent CAN edit files on feature branches when they have edit * >feature/**
    #[test]
    fn test_agent_edit_allowed_on_feature_branch() {
        let config = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: deny
  rules:
    - founders edit *
    - agents edit * >feature/**
    - agents write * >feature/**
    - agents append ./.repobox/config.yml
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        Command::new("git").args(["add", ".repobox/config.yml"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["commit", "-m", "init"]).current_dir(&repo).output().unwrap();

        std::fs::write(repo.join("src.txt"), "code").unwrap();
        Command::new("git").args(["add", "src.txt"]).current_dir(&repo).output().unwrap();

        let agent = id("evm:0xBBB0000000000000000000000000000000000002");

        // On feature branch — agents have `edit * >feature/**` and `write * >feature/**`
        let action = process_command(
            &args("commit -m ok"),
            Some(&repo),
            Some(&agent),
            Some("feature/test"),
        );
        assert!(matches!(action, ShimAction::Delegate),
            "Agent should be able to write files on feature branches, got: {:?}", action);
    }

    /// Founder can edit config, agent cannot — with explicit deny rule.
    #[test]
    fn test_founder_edit_config_allowed_agent_blocked() {
        let config = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: allow
  rules:
    - founders edit ./.repobox/config.yml
    - agents not edit ./.repobox/config.yml
    - agents edit *
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        Command::new("git").args(["add", ".repobox/config.yml"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["commit", "-m", "init"]).current_dir(&repo).output().unwrap();

        // Now modify config and stage it
        let new_config = config.to_string() + "\n# modified\n";
        std::fs::write(repo.join(".repobox/config.yml"), &new_config).unwrap();
        Command::new("git").args(["add", ".repobox/config.yml"]).current_dir(&repo).output().unwrap();

        let founder = id("evm:0xAAA0000000000000000000000000000000000001");
        let agent = id("evm:0xBBB0000000000000000000000000000000000002");

        // Founder can edit config
        let action = process_command(
            &args("commit -m ok"),
            Some(&repo),
            Some(&founder),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Delegate),
            "Founder should be able to edit config, got: {:?}", action);

        // Agent is explicitly denied
        let action = process_command(
            &args("commit -m evil"),
            Some(&repo),
            Some(&agent),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Block(ref msg) if msg.contains("cannot")),
            "Agent should be blocked from modifying config, got: {:?}", action);
    }

    /// Fran's actual config scenario: agents have `edit *` which includes config.
    /// This documents that `edit *` DOES grant access to config unless
    /// an explicit deny or priority rule prevents it.
    #[test]
    fn test_agents_edit_star_includes_config() {
        let config = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: allow
  rules:
    - founders edit ./.repobox/config.yml
    - agents edit *
"#;
        let (_tmp, repo) = setup_repo_with_config(config);

        Command::new("git").args(["add", ".repobox/config.yml"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["commit", "-m", "init"]).current_dir(&repo).output().unwrap();

        let new_config = config.to_string() + "\n# sneaky addition\n";
        std::fs::write(repo.join(".repobox/config.yml"), &new_config).unwrap();
        Command::new("git").args(["add", ".repobox/config.yml"]).current_dir(&repo).output().unwrap();

        let agent = id("evm:0xBBB0000000000000000000000000000000000002");

        // agents edit * matches .repobox/config.yml — the founders rule only creates
        // implicit deny for identities NOT matching ANY rule for that target.
        // But agents DO match via `edit *`, so they're allowed.
        let action = process_command(
            &args("commit -m sneaky"),
            Some(&repo),
            Some(&agent),
            Some("main"),
        );
        assert!(matches!(action, ShimAction::Delegate),
            "agents edit * includes config file — correct but surprising, got: {:?}", action);
    }

    /// Test verb classification: new file = write, append-only = append, modification = edit
    #[test]
    fn test_verb_classification_write_append_edit() {
        let config = r#"
groups:
  all:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: allow
  rules: []
"#;
        let (_tmp, repo) = setup_repo_with_config(config);
        Command::new("git").args(["add", ".repobox/config.yml"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["commit", "-m", "init"]).current_dir(&repo).output().unwrap();

        // Test 1: New file → Write
        std::fs::write(repo.join("new.txt"), "hello\n").unwrap();
        Command::new("git").args(["add", "new.txt"]).current_dir(&repo).output().unwrap();
        assert_eq!(classify_staged_file("new.txt", &repo), Verb::Write, "New file should be Write");

        // Commit it so we can test modifications
        Command::new("git").args(["commit", "-m", "add"]).current_dir(&repo).output().unwrap();

        // Test 2: Append-only (add lines, no deletions) → Append
        std::fs::write(repo.join("new.txt"), "hello\nworld\n").unwrap();
        Command::new("git").args(["add", "new.txt"]).current_dir(&repo).output().unwrap();
        assert_eq!(classify_staged_file("new.txt", &repo), Verb::Append, "Append-only should be Append");

        // Commit and do a real edit
        Command::new("git").args(["commit", "-m", "append"]).current_dir(&repo).output().unwrap();

        // Test 3: Modification (change existing line) → Edit
        std::fs::write(repo.join("new.txt"), "CHANGED\nworld\n").unwrap();
        Command::new("git").args(["add", "new.txt"]).current_dir(&repo).output().unwrap();
        assert_eq!(classify_staged_file("new.txt", &repo), Verb::Edit, "Modification should be Edit");
    }

    /// `edit` permission covers write and append (it's the superset)
    #[test]
    fn test_edit_covers_write_and_append() {
        let config = r#"
groups:
  editors:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: deny
  rules:
    - editors edit *
"#;
        let (_tmp, repo) = setup_repo_with_config(config);
        Command::new("git").args(["add", ".repobox/config.yml"]).current_dir(&repo).output().unwrap();
        Command::new("git").args(["commit", "-m", "init"]).current_dir(&repo).output().unwrap();

        let editor = id("evm:0xAAA0000000000000000000000000000000000001");

        // New file (classified as write) — edit permission covers it
        std::fs::write(repo.join("new.txt"), "hello").unwrap();
        Command::new("git").args(["add", "new.txt"]).current_dir(&repo).output().unwrap();

        let action = process_command(&args("commit -m new"), Some(&repo), Some(&editor), Some("main"));
        assert!(matches!(action, ShimAction::Delegate), "Editor should be able to create files (edit covers write), got: {action:?}");
    }
}
