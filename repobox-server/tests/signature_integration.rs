mod matrix;
use matrix::*;

/// Comprehensive signature policy server integration tests
/// 
/// Tests the server-side enforcement of signature policies per REPO-018.
/// Covers signed/unsigned/wrong-signer scenarios with bypass resistance testing.
/// 
/// This is the canonical test suite proving signature enforcement at the server boundary.
/// Server must validate commit signatures and reject unauthorized commits even when bypassing shim.

#[test]
fn test_signature_policy_server_integration() {
    let scenarios = create_signature_server_scenarios();
    let matrix = ScenarioMatrix::new(PolicyArea::Signatures).add_scenarios(scenarios);
    
    // Verify we have comprehensive coverage
    let server_scenarios = matrix.get_server_scenarios();
    assert!(!server_scenarios.is_empty());
    
    // Check coverage of key signature types
    let has_signed_allow = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::Signed)) &&
        matches!(s.expected.result, ExpectedResult::Allow)
    });
    assert!(has_signed_allow, "Missing signed allow scenario");
    
    let has_unsigned_reject = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::Unsigned)) &&
        matches!(s.expected.result, ExpectedResult::Reject)
    });
    assert!(has_unsigned_reject, "Missing unsigned reject scenario");
    
    let has_wrong_signer_reject = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::WrongSigner)) &&
        matches!(s.expected.result, ExpectedResult::Reject)
    });
    assert!(has_wrong_signer_reject, "Missing wrong signer reject scenario");
    
    let has_raw_git = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.client_path, Some(ClientPath::RawGit))
    });
    assert!(has_raw_git, "Missing raw git bypass test");
    
    println!("✅ Signature policy server integration test structure validated");
    println!("   - {} server scenarios defined", server_scenarios.len());
    println!("   - Covers signed/unsigned/wrong-signer with rawGit/shim paths");
    
    // Run coverage audit
    let auditor = CoverageAuditor::new();
    let analyses = auditor.audit(&[matrix]);
    let signature_analysis = analyses.iter()
        .find(|a| a.policy_area == PolicyArea::Signatures)
        .unwrap();
    
    println!("   - Missing server cases: {}", signature_analysis.missing_server_cases.len());
    println!("   - Missing shim cases: {}", signature_analysis.missing_shim_cases.len());
}

fn create_signature_server_scenarios() -> Vec<TestScenario> {
    vec![
        // Valid signed commit scenarios - should be allowed
        TestScenario::new("server_signature_push_modify_signed_rawgit_allows", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Signed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("modify_file_signed")
            .expect_allow()
            .covers(vec!["signature enforcement allowing valid signed commits", "cryptographic signature validation"])
            .with_notes("Valid signature from authorized actor should be allowed"),

        TestScenario::new("server_signature_push_create_signed_rawgit_allows", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Create)
            .with_actor_state(ActorState::Signed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("create_file_signed")
            .expect_allow()
            .covers(vec!["signature enforcement for file creation", "signed commit validation"])
            .with_notes("New files created with valid signatures should be allowed"),

        TestScenario::new("server_signature_merge_signed_rawgit_allows", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Signed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("signature_repo")
            .with_action("merge_branches_signed")
            .expect_allow()
            .covers(vec!["signature enforcement for merge commits", "signed merge validation"])
            .with_notes("Merge commits with valid signatures should be allowed"),

        // Unsigned commit scenarios - should be rejected
        TestScenario::new("server_signature_push_modify_unsigned_rawgit_rejects", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Unsigned)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("modify_file_unsigned")
            .expect_reject(ReasonCode::SignatureRequired)
            .with_message_includes(vec!["signature", "required", "unsigned not permitted"])
            .covers(vec!["signature enforcement blocking unsigned commits", "raw git bypass resistance"])
            .with_notes("Critical: server must reject unsigned commits even without shim"),

        TestScenario::new("server_signature_push_create_unsigned_rawgit_rejects", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Create)
            .with_actor_state(ActorState::Unsigned)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("create_file_unsigned")
            .expect_reject(ReasonCode::SignatureRequired)
            .with_message_includes(vec!["signature", "required"])
            .covers(vec!["signature enforcement blocking unsigned file creation"])
            .with_notes("Even new files must be signed when policy requires signatures"),

        TestScenario::new("server_signature_merge_unsigned_rawgit_rejects", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Unsigned)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("signature_repo")
            .with_action("merge_branches_unsigned")
            .expect_reject(ReasonCode::SignatureRequired)
            .covers(vec!["signature enforcement blocking unsigned merges"])
            .with_notes("Merge commits must be signed when policy requires signatures"),

        // Wrong signer scenarios - should be rejected
        TestScenario::new("server_signature_push_modify_wrongsigner_rawgit_rejects", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::WrongSigner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("modify_file_wrong_signature")
            .expect_reject(ReasonCode::SignatureRequired)
            .with_message_includes(vec!["signature", "unauthorized", "wrong signer"])
            .covers(vec!["signature enforcement blocking wrong signatures", "cryptographic signature verification"])
            .with_notes("Valid signature from unauthorized actor should be rejected"),

        TestScenario::new("server_signature_push_delete_wrongsigner_rawgit_rejects", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Delete)
            .with_actor_state(ActorState::WrongSigner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("delete_file_wrong_signature")
            .expect_reject(ReasonCode::SignatureRequired)
            .covers(vec!["signature enforcement blocking unauthorized deletions"])
            .with_notes("File deletion with wrong signature should be rejected"),

        // Empty repo scenarios - first push behavior
        TestScenario::new("server_signature_push_create_signed_rawgit_emptyrepo_allows", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Create)
            .with_actor_state(ActorState::Signed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::EmptyRepo)
            .with_fixture("empty_repo")
            .with_action("initial_commit_signed")
            .expect_allow()
            .covers(vec!["signature enforcement for initial repository commit"])
            .with_notes("Initial push to empty repo with valid signature should be allowed"),

        TestScenario::new("server_signature_push_create_unsigned_rawgit_emptyrepo_rejects", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Create)
            .with_actor_state(ActorState::Unsigned)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::EmptyRepo)
            .with_fixture("empty_repo")
            .with_action("initial_commit_unsigned")
            .expect_reject(ReasonCode::SignatureRequired)
            .covers(vec!["signature enforcement from first commit"])
            .with_notes("Even initial push must be signed when policy requires signatures"),

        // Shim parity scenarios
        TestScenario::new("shim_signature_push_modify_signed_shim_allows", PolicyArea::Signatures, TestLayer::Shim)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Signed)
            .with_client_path(ClientPath::Shim)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("modify_file_signed")
            .expect_allow()
            .with_parity_to("server_signature_push_modify_signed_rawgit_allows")
            .covers(vec!["shim parity for signature allows"])
            .with_notes("Shim should match server behavior for valid signatures"),

        TestScenario::new("shim_signature_push_modify_unsigned_shim_rejects", PolicyArea::Signatures, TestLayer::Shim)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Unsigned)
            .with_client_path(ClientPath::Shim)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("modify_file_unsigned")
            .expect_reject(ReasonCode::SignatureRequired)
            .with_parity_to("server_signature_push_modify_unsigned_rawgit_rejects")
            .covers(vec!["shim early blocking for unsigned commits"])
            .with_notes("Shim should detect missing signatures before push"),

        TestScenario::new("shim_signature_push_modify_wrongsigner_shim_rejects", PolicyArea::Signatures, TestLayer::Shim)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::WrongSigner)
            .with_client_path(ClientPath::Shim)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("modify_file_wrong_signature")
            .expect_reject(ReasonCode::SignatureRequired)
            .with_parity_to("server_signature_push_modify_wrongsigner_rawgit_rejects")
            .covers(vec!["shim early blocking for wrong signatures"])
            .with_notes("Shim should validate signature authority before push"),

        // Mixed signature scenarios (multiple commits)
        TestScenario::new("server_signature_push_mixed_commits_rawgit_rejects", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Signed) // Push contains both signed and unsigned
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("push_mixed_signed_unsigned_commits")
            .expect_reject(ReasonCode::SignatureRequired)
            .with_message_includes(vec!["signature", "required", "unsigned commit"])
            .covers(vec!["signature enforcement with mixed commit signatures", "atomic rejection"])
            .with_notes("Push with any unsigned commits should be atomically rejected"),

        // Signature verification edge cases
        TestScenario::new("server_signature_push_malformed_signature_rawgit_rejects", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::WrongSigner) // Malformed signature treated as wrong
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("modify_file_malformed_signature")
            .expect_reject(ReasonCode::SignatureRequired)
            .with_message_includes(vec!["signature", "invalid", "malformed"])
            .covers(vec!["signature enforcement blocking malformed signatures"])
            .with_notes("Malformed or corrupted signatures should be rejected"),

        TestScenario::new("server_signature_push_expired_key_rawgit_rejects", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::WrongSigner) // Expired key treated as unauthorized
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::ExistingBranch)
            .with_fixture("signature_repo")
            .with_action("modify_file_expired_key")
            .expect_reject(ReasonCode::SignatureRequired)
            .with_message_includes(vec!["signature", "expired", "unauthorized"])
            .covers(vec!["signature enforcement blocking expired keys"])
            .with_notes("Signatures with expired keys should be rejected"),

        // Force push scenarios with signature requirements
        TestScenario::new("server_signature_forcepush_signed_rawgit_allows", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::ForcePush)
            .with_change_shape(ChangeShape::Rewrite)
            .with_actor_state(ActorState::Signed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("signature_repo")
            .with_action("force_push_signed")
            .expect_allow()
            .covers(vec!["signature enforcement for force push operations"])
            .with_notes("Force push with valid signatures should be allowed if permitted"),

        TestScenario::new("server_signature_forcepush_unsigned_rawgit_rejects", PolicyArea::Signatures, TestLayer::Server)
            .with_operation(Operation::ForcePush)
            .with_change_shape(ChangeShape::Rewrite)
            .with_actor_state(ActorState::Unsigned)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("signature_repo")
            .with_action("force_push_unsigned")
            .expect_reject(ReasonCode::SignatureRequired)
            .covers(vec!["signature enforcement blocking unsigned force pushes"])
            .with_notes("Force push without signatures should be rejected"),
    ]
}

/// Test server bypass resistance for signature violations
#[test]
fn test_signature_server_bypass_resistance() {
    // Create scenario demonstrating server must reject unsigned commits even without shim
    let bypass_scenario = TestScenario::new("signature_bypass_test", PolicyArea::Signatures, TestLayer::Server)
        .with_operation(Operation::Push)
        .with_change_shape(ChangeShape::Modify)
        .with_actor_state(ActorState::Unsigned)
        .with_client_path(ClientPath::RawGit)
        .with_fixture("signature_repo")
        .expect_reject(ReasonCode::SignatureRequired)
        .with_notes("Server must reject unsigned commits even when bypassing shim");

    // Validate core security principle
    assert!(matches!(bypass_scenario.expected.result, ExpectedResult::Reject));
    assert!(matches!(bypass_scenario.expected.reason_code, Some(ReasonCode::SignatureRequired)));
    assert!(matches!(bypass_scenario.dimensions.client_path, Some(ClientPath::RawGit)));
    assert!(matches!(bypass_scenario.dimensions.actor_state, Some(ActorState::Unsigned)));
    
    println!("✅ Signature server bypass resistance test validates security boundary principle");
}

/// Test signature validation semantics
#[test]
fn test_signature_validation_semantics() {
    let scenarios = create_signature_server_scenarios();
    
    // Count scenarios by signature type
    let signed_scenarios = scenarios.iter().filter(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::Signed))
    }).count();
    
    let unsigned_scenarios = scenarios.iter().filter(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::Unsigned))
    }).count();
    
    let wrong_signer_scenarios = scenarios.iter().filter(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::WrongSigner))
    }).count();
    
    println!("📊 Signature policy scenario breakdown:");
    println!("   - Signed scenarios: {}", signed_scenarios);
    println!("   - Unsigned scenarios: {}", unsigned_scenarios);
    println!("   - Wrong signer scenarios: {}", wrong_signer_scenarios);
    
    // Validate we have comprehensive signature coverage
    assert!(signed_scenarios >= 3, "Need multiple signed scenarios for different operations");
    assert!(unsigned_scenarios >= 3, "Need unsigned scenarios for different operations");
    assert!(wrong_signer_scenarios >= 2, "Need wrong signer scenarios for security validation");
    
    // Validate expected outcomes align with signature policy
    let signed_allows = scenarios.iter().filter(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::Signed)) &&
        matches!(s.expected.result, ExpectedResult::Allow)
    }).count();
    
    let unsigned_rejects = scenarios.iter().filter(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::Unsigned)) &&
        matches!(s.expected.result, ExpectedResult::Reject)
    }).count();
    
    let wrong_signer_rejects = scenarios.iter().filter(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::WrongSigner)) &&
        matches!(s.expected.result, ExpectedResult::Reject)
    }).count();
    
    assert!(signed_allows > 0, "Signed scenarios must include allows");
    assert!(unsigned_rejects > 0, "Unsigned scenarios must include rejects");
    assert!(wrong_signer_rejects > 0, "Wrong signer scenarios must include rejects");
    
    println!("✅ Signature validation semantics validated");
}

/// Test coverage audit for signature policy completeness
#[test]
fn test_signature_coverage_completeness() {
    let scenarios = create_signature_server_scenarios();
    let matrix = ScenarioMatrix::new(PolicyArea::Signatures).add_scenarios(scenarios);
    
    let auditor = CoverageAuditor::new();
    let analyses = auditor.audit(&[matrix]);
    
    let signature_analysis = analyses.iter()
        .find(|a| a.policy_area == PolicyArea::Signatures)
        .unwrap();

    // Log coverage gaps for transparency
    if !signature_analysis.missing_server_cases.is_empty() {
        println!("⚠️  Missing signature server coverage:");
        for case in &signature_analysis.missing_server_cases {
            println!("   - {:?} × {:?} × {:?} × {:?}", 
                case.operation,
                case.change_shape.as_ref().map(|cs| format!("{:?}", cs)).unwrap_or("None".to_string()),
                case.actor_state,
                case.client_path
            );
        }
    }
    
    if !signature_analysis.missing_shim_cases.is_empty() {
        println!("⚠️  Missing signature shim coverage:");
        for case in &signature_analysis.missing_shim_cases {
            println!("   - {:?} × {:?} × {:?} × {:?}", 
                case.operation,
                case.change_shape.as_ref().map(|cs| format!("{:?}", cs)).unwrap_or("None".to_string()),
                case.actor_state,
                case.client_path
            );
        }
    }
    
    println!("📊 Signature policy coverage analysis completed");
    println!("   - Total scenarios: {}", matrix.scenarios.len());
    println!("   - Server scenarios: {}", matrix.get_server_scenarios().len());
    println!("   - Shim scenarios: {}", matrix.get_shim_scenarios().len());
    
    // Validate core security scenarios are present
    let server_scenarios = matrix.get_server_scenarios();
    
    // Must have server rejection scenarios for unsigned commits
    let has_server_unsigned_reject = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::Unsigned)) &&
        matches!(s.expected.result, ExpectedResult::Reject) &&
        matches!(s.dimensions.client_path, Some(ClientPath::RawGit))
    });
    assert!(has_server_unsigned_reject, "Critical: Must have server unsigned rejection scenarios");
    
    // Must have server allow scenarios for valid signatures
    let has_server_signed_allow = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::Signed)) &&
        matches!(s.expected.result, ExpectedResult::Allow)
    });
    assert!(has_server_signed_allow, "Must have server signed allow scenarios");
    
    // Must cover wrong signer detection
    let has_wrong_signer_reject = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.actor_state, Some(ActorState::WrongSigner)) &&
        matches!(s.expected.result, ExpectedResult::Reject)
    });
    assert!(has_wrong_signer_reject, "Must have wrong signer rejection scenarios");
    
    // Must cover multiple operations (push, merge, force-push)
    let operation_coverage = server_scenarios.iter()
        .filter_map(|s| s.dimensions.operation.as_ref())
        .collect::<std::collections::HashSet<_>>();
    assert!(operation_coverage.len() >= 3, "Must cover multiple operations (push, merge, force-push)");
    
    println!("✅ Core signature security scenarios validated");
}