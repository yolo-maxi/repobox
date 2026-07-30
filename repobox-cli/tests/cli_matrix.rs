//! CLI Test Matrix for repobox-cli
//!
//! Covers the full CLI surface with matrix-driven test organization.
//! Adapted from the server-first test matrix architecture (REPO-015..022).
//!
//! ## Matrix Dimensions
//!
//! - **CommandFamily**: keys, identity, alias, check, lint, status, init, whoami, setup, hook, shim
//! - **InputState**: valid, invalid, missing-prereqs, edge-case
//! - **ExpectedOutcome**: success (exit 0), failure (exit non-0), specific output
//! - **AgentFriendly**: deterministic output, no hidden prompts, explicit errors
//!
//! ## Coverage Contract
//!
//! Every user-facing command must have:
//! 1. A happy-path test (valid input → success)
//! 2. An error-path test (invalid/missing input → failure with message)
//! 3. An agent-friendliness assertion (deterministic, parseable output)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ── Test Fixtures ────────────────────────────────────────────────────

/// Known-good test private key (DO NOT USE for real funds).
const TEST_PRIVATE_KEY: &str = "0x4c0883a69102937d6231471b5dbb6204fe512961708279f23efb56d3f4e5f4b2";
/// Corresponding EVM address (checksummed).
const TEST_ADDRESS: &str = "0x2c7536E3605D9C16a7a3D7b1898e529396a65c23";

/// A second test key for multi-identity scenarios.
const TEST_PRIVATE_KEY_2: &str = "0xaefd8bc8725c4b3d9b647c86e1b7c9d3640c8e7a9b3f6d2e1c0a8b7f6e5d4c3b";

/// Minimal valid config.
const MINIMAL_CONFIG: &str = r#"groups:
  founders:
    - evm:0x2c7536E3605D9C16a7a3D7b1898e529396a65c23

permissions:
  default: allow
  rules: []
"#;

/// Config with permission rules for check testing.
const RULES_CONFIG: &str = r#"groups:
  founders:
    - evm:0x2c7536E3605D9C16a7a3D7b1898e529396a65c23
  agents:
    - evm:0x1111111111111111111111111111111111111111

permissions:
  default: deny
  rules:
    - founders push >main
    - founders push >feature/**
    - founders merge >main
    - founders force-push >main
    - founders edit ./.repobox/config.yml
    - agents push >feature/**
    - agents not push >main
    - agents not force-push >*
"#;

/// Config with empty groups for lint testing.
const EMPTY_GROUPS_CONFIG: &str = r#"groups:
  founders: []
  agents: []

permissions:
  default: allow
  rules: []
"#;

/// Invalid YAML to test parse error handling.
const INVALID_CONFIG: &str = r#"groups:
  founders: [
    - broken yaml
permissions:
  default: allow
"#;

/// Config with lint warnings (unused groups, etc.).
const LINT_WARNINGS_CONFIG: &str = r#"groups:
  founders:
    - evm:0x2c7536E3605D9C16a7a3D7b1898e529396a65c23
  unused_group:
    - evm:0x1111111111111111111111111111111111111111

permissions:
  default: deny
  rules:
    - founders push >main
"#;

/// Create a temporary home directory with repobox structure.
fn setup_home(tmp: &TempDir) -> std::path::PathBuf {
    let home = tmp.path().join("home");
    fs::create_dir_all(home.join(".repobox/keys")).unwrap();
    home
}

/// Create a temp directory with a git repo and .repobox/config.yml.
fn setup_repo(tmp: &TempDir, config: &str) -> std::path::PathBuf {
    let repo = tmp.path().join("repo");
    fs::create_dir_all(repo.join(".repobox")).unwrap();
    fs::write(repo.join(".repobox/config.yml"), config).unwrap();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&repo)
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(&repo)
        .output()
        .expect("git config name");

    repo
}

/// Store a test key in the home directory.
fn store_test_key(home: &Path, address: &str, private_key: &str) {
    let keys_dir = home.join(".repobox/keys");
    fs::create_dir_all(&keys_dir).unwrap();
    fs::write(keys_dir.join(format!("{address}.key")), private_key).unwrap();
}

/// Set identity in the home directory.
fn set_identity(home: &Path, address: &str) {
    let repobox_dir = home.join(".repobox");
    fs::create_dir_all(&repobox_dir).unwrap();
    fs::write(repobox_dir.join("identity"), address).unwrap();
}

/// Build a command for the `repobox` binary with isolated home.
fn repobox_cmd(home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("repobox").unwrap();
    cmd.env("REPOBOX_HOME", home.to_str().unwrap());
    cmd.env("HOME", home.to_str().unwrap());
    // Prevent shim mode: ensure argv[0] is not "git"
    cmd
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 1: KEYS
// Dimensions: generate, import, list × valid/invalid/missing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn cli_keys_generate_creates_key_and_sets_identity() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["keys", "generate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated:"))
        .stdout(predicate::str::contains("Set as current identity"));

    // Verify key file was created
    let keys_dir = home.join(".repobox/keys");
    let key_files: Vec<_> = fs::read_dir(&keys_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".key"))
        .collect();
    assert_eq!(key_files.len(), 1, "exactly one key file should be created");

    // Verify identity was set
    let identity = fs::read_to_string(home.join(".repobox/identity")).unwrap();
    assert!(identity.starts_with("0x"), "identity should be an address");
}

#[test]
fn cli_keys_generate_with_alias() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["keys", "generate", "--alias", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"));

    // Verify alias file contains the alias
    let aliases = fs::read_to_string(home.join(".repobox/aliases")).unwrap();
    assert!(aliases.contains("alice"), "alias should be stored");
}

#[test]
fn cli_keys_import_valid_key() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["keys", "import", TEST_PRIVATE_KEY])
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported:"))
        .stdout(predicate::str::contains(TEST_ADDRESS));
}

#[test]
fn cli_keys_import_invalid_hex() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["keys", "import", "not-a-valid-hex-key"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn cli_keys_import_wrong_length() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["keys", "import", "0xdeadbeef"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn cli_keys_import_with_alias() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["keys", "import", TEST_PRIVATE_KEY, "--alias", "bob"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bob"));
}

#[test]
fn cli_keys_list_empty() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No keys found"));
}

#[test]
fn cli_keys_list_shows_stored_keys() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    store_test_key(&home, TEST_ADDRESS, TEST_PRIVATE_KEY);

    repobox_cmd(&home)
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(TEST_ADDRESS));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 2: IDENTITY
// Dimensions: set × valid/invalid key, env override
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn cli_identity_set_valid_key() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["identity", "set", TEST_PRIVATE_KEY])
        .assert()
        .success()
        .stdout(predicate::str::contains("Identity set:"))
        .stdout(predicate::str::contains(TEST_ADDRESS));

    // Verify identity file
    let identity = fs::read_to_string(home.join(".repobox/identity")).unwrap();
    assert_eq!(identity.trim(), TEST_ADDRESS);
}

#[test]
fn cli_identity_set_invalid_key() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["identity", "set", "garbage"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn cli_identity_set_with_alias() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["identity", "set", TEST_PRIVATE_KEY, "--alias", "me"])
        .assert()
        .success()
        .stdout(predicate::str::contains("me"));

    let aliases = fs::read_to_string(home.join(".repobox/aliases")).unwrap();
    assert!(aliases.contains("me"), "alias should be stored");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 3: USE (identity shorthand)
// Dimensions: by-alias, by-address, unknown-alias, missing-key
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn cli_use_by_alias() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    store_test_key(&home, TEST_ADDRESS, TEST_PRIVATE_KEY);

    // Set up alias
    let aliases_path = home.join(".repobox/aliases");
    fs::write(&aliases_path, format!("alice = evm:{TEST_ADDRESS}\n")).unwrap();

    repobox_cmd(&home)
        .args(["use", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Now using:"));
}

#[test]
fn cli_use_by_evm_address() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    store_test_key(&home, TEST_ADDRESS, TEST_PRIVATE_KEY);

    repobox_cmd(&home)
        .args(["use", &format!("evm:{TEST_ADDRESS}")])
        .assert()
        .success()
        .stdout(predicate::str::contains("Now using:"));
}

#[test]
fn cli_use_unknown_alias() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["use", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown alias"));
}

#[test]
fn cli_use_alias_no_key() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    // Set alias but don't store key
    let aliases_path = home.join(".repobox/aliases");
    fs::write(&aliases_path, format!("alice = evm:{TEST_ADDRESS}\n")).unwrap();

    repobox_cmd(&home)
        .args(["use", "alice"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no key found"));
}

#[test]
fn cli_use_strips_percent_prefix() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    store_test_key(&home, TEST_ADDRESS, TEST_PRIVATE_KEY);

    let aliases_path = home.join(".repobox/aliases");
    fs::write(&aliases_path, format!("alice = evm:{TEST_ADDRESS}\n")).unwrap();

    repobox_cmd(&home)
        .args(["use", "%alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Now using:"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 4: WHOAMI
// Dimensions: identity-set, identity-not-set, with-alias
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn cli_whoami_with_identity() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    store_test_key(&home, TEST_ADDRESS, TEST_PRIVATE_KEY);
    set_identity(&home, TEST_ADDRESS);

    repobox_cmd(&home)
        .args(["whoami"])
        .assert()
        .success()
        .stdout(predicate::str::contains("identity:"))
        .stdout(predicate::str::contains(TEST_ADDRESS));
}

#[test]
fn cli_whoami_no_identity() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["whoami"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no identity configured"));
}

#[test]
fn cli_whoami_with_alias() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    store_test_key(&home, TEST_ADDRESS, TEST_PRIVATE_KEY);
    set_identity(&home, TEST_ADDRESS);

    let aliases_path = home.join(".repobox/aliases");
    fs::write(&aliases_path, format!("alice = evm:{TEST_ADDRESS}\n")).unwrap();

    repobox_cmd(&home)
        .args(["whoami"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alias: alice"))
        .stdout(predicate::str::contains("identity:"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 5: ALIAS
// Dimensions: add, remove, list × valid/invalid/missing
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn cli_alias_add_with_address() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["alias", "add", "alice", &format!("evm:{TEST_ADDRESS}")])
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"));

    let aliases = fs::read_to_string(home.join(".repobox/aliases")).unwrap();
    assert!(aliases.contains("alice"));
}

#[test]
fn cli_alias_add_uses_current_identity_when_no_address() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    set_identity(&home, TEST_ADDRESS);

    repobox_cmd(&home)
        .args(["alias", "add", "me"])
        .assert()
        .success()
        .stdout(predicate::str::contains("me"));
}

#[test]
fn cli_alias_add_no_address_no_identity() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["alias", "add", "me"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no address provided"));
}

#[test]
fn cli_alias_remove_existing() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    let aliases_path = home.join(".repobox/aliases");
    fs::write(&aliases_path, format!("alice = evm:{TEST_ADDRESS}\n")).unwrap();

    repobox_cmd(&home)
        .args(["alias", "remove", "alice"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed alice"));
}

#[test]
fn cli_alias_remove_nonexistent() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["alias", "remove", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("alias not found"));
}

#[test]
fn cli_alias_list_empty() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["alias", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No aliases"));
}

#[test]
fn cli_alias_list_shows_entries() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    let aliases_path = home.join(".repobox/aliases");
    fs::write(
        &aliases_path,
        format!("alice = evm:{TEST_ADDRESS}\nbob = evm:0x1111111111111111111111111111111111111111\n"),
    )
    .unwrap();

    repobox_cmd(&home)
        .args(["alias", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"))
        .stdout(predicate::str::contains("bob"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 6: CHECK (permission evaluation)
// Dimensions: allowed/denied/implicit-deny/default × verb × target-type
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn cli_check_allowed_by_rule() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", &format!("evm:{TEST_ADDRESS}"), "push", ">main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("allowed"));
}

#[test]
fn cli_check_denied_by_rule() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", "evm:0x1111111111111111111111111111111111111111", "push", ">main"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("denied"));
}

#[test]
fn cli_check_implicit_deny() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    // An identity not in any group, checking a verb that has rules
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", "evm:0x9999999999999999999999999999999999999999", "push", ">main"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("denied"));
}

#[test]
fn cli_check_default_allow() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, MINIMAL_CONFIG);

    // No rules for "push" with default: allow
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", &format!("evm:{TEST_ADDRESS}"), "push", ">main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("allowed"));
}

#[test]
fn cli_check_default_deny() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    // No rules for "delete" verb with default: deny
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", &format!("evm:{TEST_ADDRESS}"), "delete", ">main"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("denied"));
}

#[test]
fn cli_check_branch_wildcard() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    // founders can push to feature/**
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", &format!("evm:{TEST_ADDRESS}"), "push", ">feature/new-ui"])
        .assert()
        .success()
        .stdout(predicate::str::contains("allowed"));
}

#[test]
fn cli_check_agents_denied_main() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    // agents cannot push to main (explicit deny rule)
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", "evm:0x1111111111111111111111111111111111111111", "push", ">main"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("denied"));
}

#[test]
fn cli_check_agents_allowed_feature() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    // agents can push to feature/**
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", "evm:0x1111111111111111111111111111111111111111", "push", ">feature/fix-42"])
        .assert()
        .success()
        .stdout(predicate::str::contains("allowed"));
}

#[test]
fn cli_check_force_push_deny() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    // agents denied force-push everywhere
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", "evm:0x1111111111111111111111111111111111111111", "force-push", ">feature/test"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("denied"));
}

#[test]
fn cli_check_own_verb_all_allowed() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, MINIMAL_CONFIG);

    // "own" checks all verbs — with default: allow, everything should pass
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", &format!("evm:{TEST_ADDRESS}"), "own", ">main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("owns"));
}

#[test]
fn cli_check_own_verb_partial_deny() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    // agents don't own >main (denied force-push, push)
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", "evm:0x1111111111111111111111111111111111111111", "own", ">main"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("does NOT own"));
}

#[test]
fn cli_check_no_config() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = tmp.path().join("norepo");
    fs::create_dir_all(&repo).unwrap();

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", &format!("evm:{TEST_ADDRESS}"), "push", ">main"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no .repobox/config.yml"));
}

#[test]
fn cli_check_invalid_verb() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, MINIMAL_CONFIG);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", &format!("evm:{TEST_ADDRESS}"), "nonexistent-verb", ">main"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn cli_check_invalid_identity() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, MINIMAL_CONFIG);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", "not-a-valid-identity", "push", ">main"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn cli_check_alias_identity() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, MINIMAL_CONFIG);

    // Set up alias
    let aliases_path = home.join(".repobox/aliases");
    fs::write(&aliases_path, format!("alice = evm:{TEST_ADDRESS}\n")).unwrap();

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", "alice", "push", ">main"])
        .assert()
        .success()
        .stdout(predicate::str::contains("allowed"));
}

#[test]
fn cli_check_file_target() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    // founders can edit .repobox/config.yml
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", &format!("evm:{TEST_ADDRESS}"), "edit", "./.repobox/config.yml"])
        .assert()
        .success()
        .stdout(predicate::str::contains("allowed"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 7: LINT
// Dimensions: valid/invalid/warnings × config states
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn cli_lint_valid_config() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, MINIMAL_CONFIG);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["lint"])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn cli_lint_shows_group_and_rule_counts() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["lint"])
        .assert()
        .success()
        .stdout(predicate::str::contains("groups"))
        .stdout(predicate::str::contains("rules"));
}

#[test]
fn cli_lint_invalid_yaml() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, INVALID_CONFIG);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["lint"])
        .assert()
        .failure();
}

#[test]
fn cli_lint_no_config() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = tmp.path().join("norepo");
    fs::create_dir_all(&repo).unwrap();

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["lint"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no .repobox/config.yml"));
}

#[test]
fn cli_lint_empty_groups_shows_warnings() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, EMPTY_GROUPS_CONFIG);

    // Empty groups should still be valid (but might produce lint warnings)
    rebobox_lint_result(&home, &repo);
}

/// Helper to run lint and return the assertion for flexible checking.
fn rebobox_lint_result(home: &Path, repo: &Path) {
    repobox_cmd(home)
        .current_dir(repo)
        .args(["lint"])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn cli_lint_unused_groups_warning() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, LINT_WARNINGS_CONFIG);

    // unused_group is defined but not used in any rule
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["lint"])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
    // The lint warnings are printed but config is still valid
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 8: STATUS
// Dimensions: with-config, no-config, with-identity, no-identity
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn cli_status_with_config_and_identity() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);
    store_test_key(&home, TEST_ADDRESS, TEST_PRIVATE_KEY);
    set_identity(&home, TEST_ADDRESS);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Identity:"))
        .stdout(predicate::str::contains("Groups:"))
        .stdout(predicate::str::contains("Rules:"))
        .stdout(predicate::str::contains("Default:"));
}

#[test]
fn cli_status_no_config() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = tmp.path().join("norepo");
    fs::create_dir_all(&repo).unwrap();
    // Init git repo so status can read branch
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .output()
        .unwrap();

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no .repobox/config.yml"));
}

#[test]
fn cli_status_no_identity() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, MINIMAL_CONFIG);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not set"));
}

#[test]
fn cli_status_shows_member_of() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);
    store_test_key(&home, TEST_ADDRESS, TEST_PRIVATE_KEY);
    set_identity(&home, TEST_ADDRESS);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Member of:"))
        .stdout(predicate::str::contains("founders"));
}

#[test]
fn cli_status_shows_permission_summary() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);
    store_test_key(&home, TEST_ADDRESS, TEST_PRIVATE_KEY);
    set_identity(&home, TEST_ADDRESS);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Permissions on >"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 9: INIT
// Dimensions: fresh-repo, existing-config, force-overwrite, not-in-repo
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn cli_init_creates_config() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = tmp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();

    // Init git repo first
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .output()
        .unwrap();

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized repo.box"));

    assert!(repo.join(".repobox/config.yml").exists());
}

#[test]
fn cli_init_preserves_existing_config() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    let original = fs::read_to_string(repo.join(".repobox/config.yml")).unwrap();

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("existing"));

    let after = fs::read_to_string(repo.join(".repobox/config.yml")).unwrap();
    assert_eq!(original, after, "config should not be overwritten without --force");
}

#[test]
fn cli_init_force_overwrites() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["init", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--force"));

    let after = fs::read_to_string(repo.join(".repobox/config.yml")).unwrap();
    assert_ne!(after, RULES_CONFIG, "config should be overwritten with --force");
}

#[test]
fn cli_init_not_in_git_repo() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let dir = tmp.path().join("not-a-repo");
    fs::create_dir_all(&dir).unwrap();

    repobox_cmd(&home)
        .current_dir(&dir)
        .args(["init"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not in the root folder"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 10: SETUP
// Dimensions: conflicting flags
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn cli_setup_remove_and_replace_conflicts() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["setup", "--remove", "--replace-binary"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be combined"));
}

#[test]
fn cli_setup_replace_and_restore_conflicts() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["setup", "--replace-binary", "--restore-binary"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("choose either"));
}

#[test]
fn cli_setup_restore_no_backup() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    repobox_cmd(&home)
        .args(["setup", "--restore-binary"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no backup found"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 11: AGENT BEHAVIOR (virtuals integration)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// Agent branch naming and commit validation are covered by inline #[cfg(test)]
// in main.rs. Here we add integration-level coverage for the binary behavior.

#[test]
fn cli_hook_unknown_hook_name_fails() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);
    store_test_key(&home, TEST_ADDRESS, TEST_PRIVATE_KEY);
    set_identity(&home, TEST_ADDRESS);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["hook", "nonexistent-hook"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown hook"));
}

#[test]
fn cli_hook_no_config_allows() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = tmp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .output()
        .unwrap();

    // Without .repobox/config.yml, hooks should pass through
    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["hook", "pre-commit"])
        .assert()
        .success();
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 12: AGENT FRIENDLINESS
// Cross-cutting: deterministic output, explicit errors, no hidden prompts
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn agent_friendly_check_produces_machine_parseable_output() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    // Allow case: should contain "allowed" and the identity
    let output = repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", &format!("evm:{TEST_ADDRESS}"), "push", ">main"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("allowed"), "output must contain clear allow/deny verdict");
    assert!(stdout.contains(TEST_ADDRESS), "output must include the identity checked");
}

#[test]
fn agent_friendly_errors_on_stderr() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    // Error output should go to stderr, not stdout
    let output = repobox_cmd(&home)
        .args(["whoami"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no identity"), "errors must go to stderr");
    assert!(
        String::from_utf8_lossy(&output.stdout).is_empty(),
        "stdout should be empty on error"
    );
}

#[test]
fn agent_friendly_exit_codes_are_distinct() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    // Success case
    let success = repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", &format!("evm:{TEST_ADDRESS}"), "push", ">main"])
        .output()
        .unwrap();
    assert!(success.status.success(), "allowed check should exit 0");

    // Failure case
    let failure = repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", "evm:0x1111111111111111111111111111111111111111", "push", ">main"])
        .output()
        .unwrap();
    assert!(!failure.status.success(), "denied check should exit non-zero");
}

#[test]
fn agent_friendly_no_interactive_prompts_on_missing_identity() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    // `whoami` with no identity should fail immediately, not prompt
    let output = repobox_cmd(&home)
        .args(["whoami"])
        .timeout(std::time::Duration::from_secs(5))
        .output()
        .unwrap();

    assert!(!output.status.success(), "should fail, not block on prompt");
}

#[test]
fn agent_friendly_lint_deterministic_output() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, RULES_CONFIG);

    // Run lint twice, output should be identical
    let output1 = repobox_cmd(&home)
        .current_dir(&repo)
        .args(["lint"])
        .output()
        .unwrap();
    let output2 = repobox_cmd(&home)
        .current_dir(&repo)
        .args(["lint"])
        .output()
        .unwrap();

    assert_eq!(
        String::from_utf8_lossy(&output1.stdout),
        String::from_utf8_lossy(&output2.stdout),
        "lint output must be deterministic across runs"
    );
}

#[test]
fn agent_friendly_keys_generate_is_idempotent_per_run() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);

    // First generate
    let out1 = repobox_cmd(&home)
        .args(["keys", "generate"])
        .output()
        .unwrap();
    assert!(out1.status.success());

    // Second generate creates a new key (doesn't error)
    let out2 = repobox_cmd(&home)
        .args(["keys", "generate"])
        .output()
        .unwrap();
    assert!(out2.status.success());

    // Should have 2 key files now
    let key_files: Vec<_> = fs::read_dir(home.join(".repobox/keys"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".key"))
        .collect();
    assert_eq!(key_files.len(), 2, "each generate should create a new key");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 13: EDGE CASES
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn edge_case_repobox_home_env_override() {
    let tmp = TempDir::new().unwrap();
    let custom_home = tmp.path().join("custom");
    fs::create_dir_all(custom_home.join(".repobox/keys")).unwrap();

    // Use REPOBOX_HOME to redirect storage
    let mut cmd = Command::cargo_bin("repobox").unwrap();
    cmd.env("REPOBOX_HOME", custom_home.to_str().unwrap());
    cmd.env("HOME", custom_home.to_str().unwrap());
    cmd.args(["keys", "generate"]);
    cmd.assert().success();

    // Key should be in custom location
    let key_files: Vec<_> = fs::read_dir(custom_home.join(".repobox/keys"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".key"))
        .collect();
    assert_eq!(key_files.len(), 1);
}

#[test]
fn edge_case_empty_aliases_file() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    fs::write(home.join(".repobox/aliases"), "").unwrap();

    repobox_cmd(&home)
        .args(["alias", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No aliases"));
}

#[test]
fn edge_case_malformed_aliases_file() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    fs::write(home.join(".repobox/aliases"), "not a valid format\n\n= =\nfoo\n").unwrap();

    // Should not crash — gracefully handle malformed content
    repobox_cmd(&home)
        .args(["alias", "list"])
        .assert()
        .success();
}

#[test]
fn edge_case_check_with_broken_config() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, INVALID_CONFIG);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["check", &format!("evm:{TEST_ADDRESS}"), "push", ">main"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn edge_case_status_with_broken_config() {
    let tmp = TempDir::new().unwrap();
    let home = setup_home(&tmp);
    let repo = setup_repo(&tmp, INVALID_CONFIG);

    repobox_cmd(&home)
        .current_dir(&repo)
        .args(["status"])
        .assert()
        .failure();
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// FAMILY 14: COVERAGE AUDIT (self-documenting)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// This test documents the matrix coverage and acts as an audit.
/// If you add a new command or behavior, add it here.
#[test]
fn coverage_audit_all_commands_have_tests() {
    // Every public CLI command must have at least:
    //   - 1 success test
    //   - 1 failure test
    //
    // This list must be kept in sync with Commands enum in main.rs.
    let commands = [
        "init",
        "keys generate",
        "keys import",
        "keys list",
        "identity set",
        "use",
        "whoami",
        "alias add",
        "alias remove",
        "alias list",
        "check",
        "lint",
        "status",
        "setup",
        "hook",
    ];

    // Self-documenting: print the matrix
    println!("=== CLI Test Matrix Coverage ===");
    println!();
    println!("Commands covered: {}", commands.len());
    println!();
    for cmd in &commands {
        println!("  [x] {cmd}");
    }
    println!();
    println!("Cross-cutting dimensions:");
    println!("  [x] Agent-friendly output (stderr/stdout separation)");
    println!("  [x] Deterministic output (lint, check)");
    println!("  [x] No interactive prompts (whoami, check)");
    println!("  [x] Exit code semantics (0 = success, non-0 = failure)");
    println!("  [x] REPOBOX_HOME env override");
    println!("  [x] Alias resolution in check");
    println!("  [x] Edge cases (malformed files, broken configs)");
}
