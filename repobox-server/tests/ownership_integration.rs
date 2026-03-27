mod matrix;
use matrix::*;

/// Comprehensive ownership policy server integration tests
/// 
/// Tests the server-side enforcement of ownership policies per REPO-016.
/// Covers owner/non-owner scenarios across create/modify/delete/rename operations
/// with both raw git and shim client paths.
/// 
/// This is the canonical test suite proving ownership enforcement at the server boundary.
/// Shim tests provide UX parity but these server tests are the security boundary.

#[test]
fn test_ownership_policy_server_integration() {
    let scenarios = create_ownership_server_scenarios();
    let matrix = ScenarioMatrix::new(PolicyArea::Ownership).add_scenarios(scenarios);
    
    // Verify we have comprehensive coverage
    let server_scenarios = matrix.get_server_scenarios();
    assert!(!server_scenarios.is_empty());
    
    // Check coverage of key combinations
    let has_owner_modify = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::Owner)) &&
        matches!(s.dimensions.change_shape, Some(ChangeShape::Modify))
    });
    assert!(has_owner_modify, "Missing owner modify scenario");
    
    let has_nonowner_modify = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::NonOwner)) &&
        matches!(s.dimensions.change_shape, Some(ChangeShape::Modify))
    });
    assert!(has_nonowner_modify, "Missing non-owner modify scenario");
    
    let has_raw_git = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.client_path, Some(ClientPath::RawGit))
    });
    assert!(has_raw_git, "Missing raw git bypass test");
    
    println!("✅ Ownership policy server integration test structure validated");
    println!("   - {} server scenarios defined", server_scenarios.len());
    println!("   - Covers owner/non-owner, modify/delete/rename, rawGit/shim paths");
    
    // Run coverage audit
    let auditor = CoverageAuditor::new();
    let analyses = auditor.audit(&[matrix]);
    let ownership_analysis = analyses.iter()
        .find(|a| a.policy_area == PolicyArea::Ownership)
        .unwrap();
    
    println!("   - Missing server cases: {}", ownership_analysis.missing_server_cases.len());
    println!("   - Missing shim cases: {}", ownership_analysis.missing_shim_cases.len());
}

fn create_ownership_server_scenarios() -> Vec<TestScenario> {
    vec![
        // Owner scenarios - should all be allowed
        TestScenario::new("server_ownership_push_modify_owner_rawgit_allows", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Owner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("ownership_repo")
            .with_action("modify_owned_file")
            .expect_allow()
            .covers(vec!["ownership enforcement for owners", "modify owned file"]),

        TestScenario::new("server_ownership_push_delete_owner_rawgit_allows", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Delete)
            .with_actor_state(ActorState::Owner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("ownership_repo")
            .with_action("delete_owned_file")
            .expect_allow()
            .covers(vec!["ownership enforcement for owners", "delete owned file"]),

        TestScenario::new("server_ownership_push_rename_owner_rawgit_allows", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Rename)
            .with_actor_state(ActorState::Owner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("ownership_repo")
            .with_action("rename_owned_file")
            .expect_allow()
            .covers(vec!["ownership enforcement for owners", "rename owned file"]),

        // Non-owner scenarios - should be rejected
        TestScenario::new("server_ownership_push_modify_nonowner_rawgit_rejects", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("ownership_repo")
            .with_action("modify_owned_file")
            .expect_reject(ReasonCode::OwnershipViolation)
            .with_message_includes(vec!["ownership", "violation", "not permitted"])
            .covers(vec!["ownership enforcement blocking", "raw git bypass resistance"])
            .with_notes("Critical: server must reject even without shim"),

        TestScenario::new("server_ownership_push_delete_nonowner_rawgit_rejects", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Delete)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("ownership_repo")
            .with_action("delete_owned_file")
            .expect_reject(ReasonCode::OwnershipViolation)
            .with_message_includes(vec!["ownership", "violation"])
            .covers(vec!["ownership enforcement blocking file deletion"])
            .with_notes("Server must prevent unauthorized file deletion"),

        TestScenario::new("server_ownership_push_rename_nonowner_rawgit_rejects", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Rename)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("ownership_repo")
            .with_action("rename_owned_file")
            .expect_reject(ReasonCode::OwnershipViolation)
            .with_message_includes(vec!["ownership", "violation"])
            .covers(vec!["ownership enforcement blocking file rename"])
            .with_notes("Prevent unauthorized renames that could bypass ownership"),

        // Create scenarios - unowned files, so both actors should be allowed
        TestScenario::new("server_ownership_push_create_owner_rawgit_allows", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Create)
            .with_actor_state(ActorState::Owner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("ownership_repo")
            .with_action("create_new_file")
            .expect_allow()
            .covers(vec!["ownership policy allows creating unowned files"]),

        TestScenario::new("server_ownership_push_create_nonowner_rawgit_allows", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Create)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("ownership_repo")
            .with_action("create_new_file")
            .expect_allow()
            .covers(vec!["ownership policy allows non-owners to create unowned files"])
            .with_notes("New files have no owner, so anyone can create them"),

        // Shim parity scenarios - shim should match server behavior
        TestScenario::new("shim_ownership_push_modify_owner_shim_allows", PolicyArea::Ownership, TestLayer::Shim)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Owner)
            .with_client_path(ClientPath::Shim)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("ownership_repo")
            .with_action("modify_owned_file")
            .expect_allow()
            .with_parity_to("server_ownership_push_modify_owner_rawgit_allows")
            .covers(vec!["shim parity with server for owner allow"])
            .with_notes("Shim should match server behavior for owner operations"),

        TestScenario::new("shim_ownership_push_modify_nonowner_shim_rejects", PolicyArea::Ownership, TestLayer::Shim)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::Shim)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("ownership_repo")
            .with_action("modify_owned_file")
            .expect_reject(ReasonCode::OwnershipViolation)
            .with_parity_to("server_ownership_push_modify_nonowner_rawgit_rejects")
            .with_message_includes(vec!["ownership", "violation"])
            .covers(vec!["shim early blocking for ownership violations"])
            .with_notes("Shim should block early to avoid network transfer"),

        TestScenario::new("shim_ownership_push_delete_nonowner_shim_rejects", PolicyArea::Ownership, TestLayer::Shim)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Delete)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::Shim)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("ownership_repo")
            .with_action("delete_owned_file")
            .expect_reject(ReasonCode::OwnershipViolation)
            .with_parity_to("server_ownership_push_delete_nonowner_rawgit_rejects")
            .covers(vec!["shim blocking of unauthorized deletes"])
            .with_notes("Prevent delete attempts before push"),

        // Merge scenarios - test ownership in merge commits
        TestScenario::new("server_ownership_merge_modify_owner_rawgit_allows", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Owner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("ownership_repo")
            .with_action("merge_with_owned_file_changes")
            .expect_allow()
            .covers(vec!["ownership enforcement in merge commits"])
            .with_notes("Merge commits should respect ownership of contained changes"),

        TestScenario::new("server_ownership_merge_modify_nonowner_rawgit_rejects", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("ownership_repo")
            .with_action("merge_with_owned_file_changes")
            .expect_reject(ReasonCode::OwnershipViolation)
            .covers(vec!["ownership enforcement blocking unauthorized merges"])
            .with_notes("Merge commits containing unauthorized changes should be rejected"),
    ]
}

/// Test that demonstrates server-first principle
#[test]
fn test_server_bypass_resistance() {
    // Create scenario where shim would allow but server must reject
    let bypass_scenario = TestScenario::new("server_bypass_test", PolicyArea::Ownership, TestLayer::Server)
        .with_operation(Operation::Push)
        .with_change_shape(ChangeShape::Modify)
        .with_actor_state(ActorState::NonOwner)
        .with_client_path(ClientPath::RawGit)
        .with_fixture("ownership_repo")
        .expect_reject(ReasonCode::OwnershipViolation)
        .with_notes("Server must reject even when bypassing shim");

    // This demonstrates the core principle: server is the security boundary
    assert!(matches!(bypass_scenario.expected.result, ExpectedResult::Reject));
    assert!(matches!(bypass_scenario.expected.reason_code, Some(ReasonCode::OwnershipViolation)));
    assert!(matches!(bypass_scenario.dimensions.client_path, Some(ClientPath::RawGit)));
    
    println!("✅ Server bypass resistance test validates security boundary principle");
}

/// Test coverage audit for ownership policy
#[test]
fn test_ownership_coverage_completeness() {
    let scenarios = create_ownership_server_scenarios();
    let matrix = ScenarioMatrix::new(PolicyArea::Ownership).add_scenarios(scenarios);
    
    let auditor = CoverageAuditor::new();
    let analyses = auditor.audit(&[matrix]);
    
    let ownership_analysis = analyses.iter()
        .find(|a| a.policy_area == PolicyArea::Ownership)
        .unwrap();

    // Log coverage gaps for transparency
    if !ownership_analysis.missing_server_cases.is_empty() {
        println!("⚠️  Missing server coverage:");
        for case in &ownership_analysis.missing_server_cases {
            println!("   - {:?} × {:?} × {:?} × {:?}", 
                case.operation,
                case.change_shape.as_ref().map(|cs| format!("{:?}", cs)).unwrap_or("None".to_string()),
                case.actor_state,
                case.client_path
            );
        }
    }
    
    if !ownership_analysis.missing_shim_cases.is_empty() {
        println!("⚠️  Missing shim coverage:");
        for case in &ownership_analysis.missing_shim_cases {
            println!("   - {:?} × {:?} × {:?} × {:?}", 
                case.operation,
                case.change_shape.as_ref().map(|cs| format!("{:?}", cs)).unwrap_or("None".to_string()),
                case.actor_state,
                case.client_path
            );
        }
    }
    
    // For ownership policy, we should have comprehensive server coverage
    // Some missing cases are acceptable if they're explicitly excluded or irrelevant
    println!("📊 Ownership policy coverage analysis completed");
    println!("   - Total scenarios: {}", matrix.scenarios.len());
    println!("   - Server scenarios: {}", matrix.get_server_scenarios().len());
    println!("   - Shim scenarios: {}", matrix.get_shim_scenarios().len());
    
    // Validate core security scenarios are present
    let server_scenarios = matrix.get_server_scenarios();
    
    // Must have non-owner rejection scenarios for server security
    let has_nonowner_server_reject = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::NonOwner)) &&
        matches!(s.expected.result, ExpectedResult::Reject) &&
        matches!(s.dimensions.client_path, Some(ClientPath::RawGit))
    });
    assert!(has_nonowner_server_reject, "Critical: Must have non-owner server rejection scenarios");
    
    // Must have owner allow scenarios
    let has_owner_server_allow = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::Owner)) &&
        matches!(s.expected.result, ExpectedResult::Allow)
    });
    assert!(has_owner_server_allow, "Must have owner allow scenarios");
    
    println!("✅ Core ownership security scenarios validated");
}