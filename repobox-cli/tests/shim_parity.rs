//! REPO-026: Shim parity tests for branch/commit preflight UX
//!
//! Tests existing shim logic in repobox-core/src/shim.rs for parity with server enforcement.
//! Validates that shim early-blocking behavior matches server allow/deny decisions.
//!
//! ## Test Coverage Areas
//! 1. Branch naming policy enforcement
//! 2. Commit signature requirements  
//! 3. Repository state detection
//! 4. Policy configuration validation
//!
//! ## Parity Methodology
//! For each shim early-block scenario, verify equivalent server rejection exists
//! in REPO-015-022 test suite with matching outcome semantics.

use std::process::Command;
use tempfile::TempDir;
use repobox::shim::{process_command, ShimAction};
use repobox::config::Identity;

/// Helper to create a test git repository with repobox config
fn setup_test_repo(config_content: &str) -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().to_path_buf();

    // Initialize git repo
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
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo)
        .output()
        .unwrap();

    if !config_content.is_empty() {
        std::fs::create_dir_all(repo.join(".repobox")).unwrap();
        std::fs::write(repo.join(".repobox/config.yml"), config_content).unwrap();
    }

    (tmp, repo)
}

/// Helper to create test identity
fn test_identity(address: &str) -> Identity {
    Identity::parse(&format!("evm:{}", address)).unwrap()
}

/// Helper to parse command args
fn args(s: &str) -> Vec<String> {
    s.split_whitespace().map(|s| s.to_string()).collect()
}

// ==============================================================================
// Section 1: Branch Naming Policy Enforcement Tests
// ==============================================================================

#[test]
fn test_shim_branch_naming_feature_allowed() {
    let config = r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "agents branch >feature/*"
"#;
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xBBB0000000000000000000000000000000000002");

    // Test shim allows feature branch creation
    let action = process_command(
        &args("checkout -b feature/new-feature"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Delegate),
        "Shim should allow feature branch creation for agents, got: {:?}",
        action
    );
}

#[test]
fn test_shim_branch_naming_feature_denied() {
    let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "founders branch >*"
"#;
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xBBB0000000000000000000000000000000000002");

    // Test shim blocks branch creation when not allowed
    let action = process_command(
        &args("checkout -b feature/blocked"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Block(ref msg) if msg.contains("cannot branch")),
        "Shim should block branch creation when not allowed, got: {:?}",
        action
    );
}

#[test] 
fn test_shim_branch_naming_release_pattern() {
    let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "founders branch >release/*"
    - "agents branch >feature/*"
"#;
    let (_tmp, repo) = setup_test_repo(config);
    let agent_identity = test_identity("0xBBB0000000000000000000000000000000000002");
    let founder_identity = test_identity("0xAAA0000000000000000000000000000000000001");

    // Agent should be blocked from release branches
    let action = process_command(
        &args("checkout -b release/v1.0"),
        Some(&repo),
        Some(&agent_identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Block(ref msg) if msg.contains("cannot branch")),
        "Agent should be blocked from release branches, got: {:?}",
        action
    );

    // Founder should be allowed
    let action = process_command(
        &args("checkout -b release/v1.0"),
        Some(&repo),
        Some(&founder_identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Delegate),
        "Founder should be allowed to create release branches, got: {:?}",
        action
    );
}

#[test]
fn test_shim_branch_deletion_check() {
    let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "founders delete >*"
    - "agents delete >feature/*"
"#;
    let (_tmp, repo) = setup_test_repo(config);
    let agent_identity = test_identity("0xBBB0000000000000000000000000000000000002");
    let founder_identity = test_identity("0xAAA0000000000000000000000000000000000001");

    // Agent can delete feature branches
    let action = process_command(
        &args("branch -d feature/old"),
        Some(&repo),
        Some(&agent_identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Delegate),
        "Agent should be able to delete feature branches, got: {:?}",
        action
    );

    // Agent blocked from deleting main
    let action = process_command(
        &args("branch -d main"),
        Some(&repo),
        Some(&agent_identity),
        Some("feature/current"),
    );
    
    assert!(
        matches!(action, ShimAction::Block(ref msg) if msg.contains("cannot delete")),
        "Agent should be blocked from deleting main branch, got: {:?}",
        action
    );

    // Founder can delete any branch
    let action = process_command(
        &args("branch -d main"),
        Some(&repo),
        Some(&founder_identity),
        Some("feature/current"),
    );
    
    assert!(
        matches!(action, ShimAction::Delegate),
        "Founder should be able to delete any branch, got: {:?}",
        action
    );
}

// ==============================================================================
// Section 2: Commit Signature Requirements Tests
// ==============================================================================

#[test]
fn test_shim_commit_no_identity_blocks() {
    let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "founders edit *"
"#;
    let (_tmp, repo) = setup_test_repo(config);

    // Test shim blocks commit when no identity configured
    let action = process_command(
        &args("commit -m test"),
        Some(&repo),
        None, // No identity
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Block(ref msg) if msg.contains("no identity configured")),
        "Shim should block commits when no identity configured, got: {:?}",
        action
    );
}

#[test]
fn test_shim_commit_with_valid_identity_delegates() {
    let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  default: allow
  rules: []
"#;
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xAAA0000000000000000000000000000000000001");

    // Create and stage a test file
    std::fs::write(repo.join("test.txt"), "hello").unwrap();
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(&repo)
        .output()
        .unwrap();

    // Test shim delegates commit when identity is valid
    let action = process_command(
        &args("commit -m test"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Delegate),
        "Shim should delegate commits with valid identity, got: {:?}",
        action
    );
}

// ==============================================================================
// Section 3: Repository State Detection Tests
// ==============================================================================

#[test]
fn test_shim_no_config_passthrough() {
    let (_tmp, repo) = setup_test_repo("");
    // Remove config file to simulate no repobox config
    let _ = std::fs::remove_file(repo.join(".repobox/config.yml"));
    
    let identity = test_identity("0xAAA0000000000000000000000000000000000001");

    // Test shim passes through when no config exists
    let action = process_command(
        &args("commit -m test"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Passthrough),
        "Shim should passthrough when no config exists, got: {:?}",
        action
    );
}

#[test]
fn test_shim_outside_repo_passthrough() {
    let identity = test_identity("0xAAA0000000000000000000000000000000000001");

    // Test shim passes through when outside repo
    let action = process_command(
        &args("commit -m test"),
        None, // No repo root
        Some(&identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Passthrough),
        "Shim should passthrough when outside repo, got: {:?}",
        action
    );
}

#[test]
fn test_shim_unborn_head_detection() {
    // Create empty repo (unborn HEAD state)
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path().to_path_buf();
    
    Command::new("git")
        .args(["init"])
        .current_dir(&repo)
        .output()
        .unwrap();
    
    // Don't set user config or make initial commit - repo stays in unborn HEAD state
    
    let config = r#"
groups:
  all: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  default: allow
  rules: []
"#;
    std::fs::create_dir_all(repo.join(".repobox")).unwrap();
    std::fs::write(repo.join(".repobox/config.yml"), config).unwrap();
    
    let identity = test_identity("0xAAA0000000000000000000000000000000000001");

    // Test shim handles unborn HEAD state gracefully  
    let action = process_command(
        &args("commit -m initial"),
        Some(&repo),
        Some(&identity),
        None, // No current branch (unborn HEAD)
    );
    
    // Should delegate - shim shouldn't crash on unborn HEAD
    assert!(
        matches!(action, ShimAction::Delegate | ShimAction::Passthrough),
        "Shim should handle unborn HEAD gracefully, got: {:?}",
        action
    );
}

// ==============================================================================
// Section 4: Policy Configuration Validation Tests  
// ==============================================================================

#[test]
fn test_shim_invalid_config_blocks() {
    let invalid_config = "this is not valid: yaml: [[[";
    let (_tmp, repo) = setup_test_repo(invalid_config);
    let identity = test_identity("0xAAA0000000000000000000000000000000000001");

    // Test shim blocks checked commands when config is invalid
    let action = process_command(
        &args("commit -m test"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Block(ref msg) if msg.contains("error")),
        "Shim should block commits when config is invalid, got: {:?}",
        action
    );
}

#[test]
fn test_shim_invalid_config_allows_read_commands() {
    let invalid_config = "this is not valid: yaml: [[[";
    let (_tmp, repo) = setup_test_repo(invalid_config);
    let identity = test_identity("0xAAA0000000000000000000000000000000000001");

    // Test shim allows passthrough commands even with invalid config
    let action = process_command(
        &args("status"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Passthrough),
        "Shim should allow read commands even with invalid config, got: {:?}",
        action
    );
}

#[test] 
fn test_shim_missing_config_file_passthrough() {
    let (_tmp, repo) = setup_test_repo("");
    // Explicitly remove config directory
    let _ = std::fs::remove_dir_all(repo.join(".repobox"));
    
    let identity = test_identity("0xAAA0000000000000000000000000000000000001");

    // Test shim passes through all commands when config missing
    for cmd in &["commit -m test", "merge feature", "push origin main", "checkout -b new"] {
        let action = process_command(
            &args(cmd),
            Some(&repo),
            Some(&identity),
            Some("main"),
        );
        
        assert!(
            matches!(action, ShimAction::Passthrough),
            "Command '{}' should passthrough with missing config, got: {:?}",
            cmd,
            action
        );
    }
}

// ==============================================================================
// Section 5: Push Operation Tests
// ==============================================================================

#[test]
fn test_shim_push_branch_allowed() {
    let config = r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "agents push >feature/*"
"#;
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xBBB0000000000000000000000000000000000002");

    // Test shim allows push to feature branch
    let action = process_command(
        &args("push origin feature/test"),
        Some(&repo),
        Some(&identity),
        Some("feature/test"),
    );
    
    assert!(
        matches!(action, ShimAction::Delegate),
        "Shim should allow push to feature branches, got: {:?}",
        action
    );
}

#[test]
fn test_shim_push_main_denied() {
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
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xBBB0000000000000000000000000000000000002");

    // Test shim blocks push to main for agents
    let action = process_command(
        &args("push origin main"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Block(ref msg) if msg.contains("cannot push")),
        "Shim should block push to main for agents, got: {:?}",
        action
    );
}

#[test]
fn test_shim_force_push_denied() {
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
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xBBB0000000000000000000000000000000000002");

    // Test shim blocks force push when not allowed
    let action = process_command(
        &args("push --force origin main"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Block(ref msg) if msg.contains("force-push")),
        "Shim should block force-push when not allowed, got: {:?}",
        action
    );
}

// ==============================================================================
// Section 6: Merge Operation Tests
// ==============================================================================

#[test]
fn test_shim_merge_allowed() {
    let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "founders merge >main"
"#;
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xAAA0000000000000000000000000000000000001");

    // Test shim allows merge when permitted
    let action = process_command(
        &args("merge feature/test"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Delegate),
        "Shim should allow merge when permitted, got: {:?}",
        action
    );
}

#[test] 
fn test_shim_merge_denied() {
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
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xBBB0000000000000000000000000000000000002");

    // Test shim blocks merge when not permitted
    let action = process_command(
        &args("merge feature/test"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Block(ref msg) if msg.contains("cannot merge")),
        "Shim should block merge when not permitted, got: {:?}",
        action
    );
}

// ==============================================================================
// Section 7: Error Message Quality Tests  
// ==============================================================================

#[test]
fn test_shim_error_messages_are_deterministic() {
    let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  default: deny
  rules:
    - "founders push >main"
"#;
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xBBB0000000000000000000000000000000000002");

    // Test error messages are consistent across multiple calls - use main branch to ensure block
    let action1 = process_command(
        &args("push origin main"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    let action2 = process_command(
        &args("push origin main"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    // Both should produce identical error messages
    match (&action1, &action2) {
        (ShimAction::Block(msg1), ShimAction::Block(msg2)) => {
            assert_eq!(msg1, msg2, "Error messages should be deterministic");
        }
        _ => panic!("Expected both actions to be Block variants, got: {:?} and {:?}", action1, action2),
    }
}

#[test]
fn test_shim_error_messages_include_identity() {
    let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "founders push >main"
"#;
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xBBB0000000000000000000000000000000000002");

    // Test error message includes identity for context
    let action = process_command(
        &args("push origin main"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    if let ShimAction::Block(msg) = action {
        assert!(
            msg.contains("0xBBB0000000000000000000000000000000000002"),
            "Error message should include identity: {}",
            msg
        );
    } else {
        panic!("Expected Block action, got: {:?}", action);
    }
}

#[test]
fn test_shim_error_messages_specify_operation() {
    let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]  
permissions:
  rules:
    - "founders merge >main"
"#;
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xBBB0000000000000000000000000000000000002");

    // Test different operations produce specific error messages
    let push_action = process_command(
        &args("push origin main"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    let merge_action = process_command(
        &args("merge feature/test"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    let branch_action = process_command(
        &args("checkout -b new/branch"),
        Some(&repo),
        Some(&identity),
        Some("main"),
    );
    
    // Each should mention the specific operation
    if let ShimAction::Block(msg) = push_action {
        assert!(msg.contains("push"), "Push error should mention 'push': {}", msg);
    }
    
    if let ShimAction::Block(msg) = merge_action {
        assert!(msg.contains("merge"), "Merge error should mention 'merge': {}", msg);
    }
    
    if let ShimAction::Block(msg) = branch_action {
        assert!(msg.contains("branch"), "Branch error should mention 'branch': {}", msg);
    }
}

// ==============================================================================
// Section 8: Passthrough Command Tests
// ==============================================================================

#[test]
fn test_shim_read_commands_always_passthrough() {
    let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "founders push >main"
"#;
    let (_tmp, repo) = setup_test_repo(config);
    let identity = test_identity("0xAAA0000000000000000000000000000000000001");

    // Test read commands always passthrough regardless of config
    let read_commands = [
        "status", "log", "diff", "add", "stash", 
        "fetch", "clone", "remote", "show", "tag",
        "reflog", "blame", "bisect", "archive", "shortlog", "describe"
    ];
    
    for cmd in &read_commands {
        let action = process_command(
            &args(cmd),
            Some(&repo),
            Some(&identity),
            Some("main"),
        );
        
        assert!(
            matches!(action, ShimAction::Passthrough),
            "Command '{}' should always passthrough, got: {:?}",
            cmd,
            action
        );
    }
}

#[test]
fn test_shim_pull_always_delegates() {
    let config = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "founders push >main"
"#;
    let (_tmp, repo) = setup_test_repo(config);

    // Test pull delegates even without identity or with invalid config
    let action = process_command(
        &args("pull origin main"),
        Some(&repo),
        None, // No identity
        Some("main"),
    );
    
    assert!(
        matches!(action, ShimAction::Delegate),
        "Pull should delegate even without identity, got: {:?}",
        action
    );
}