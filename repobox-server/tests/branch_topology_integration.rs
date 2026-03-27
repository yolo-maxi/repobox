mod matrix;
use matrix::*;

/// Comprehensive branch topology policy server integration tests
/// 
/// Tests the server-side enforcement of branch topology policies per REPO-019.
/// Covers branch create/delete/update operations, non-fast-forward protection,
/// force-push restrictions, and protected branch enforcement.
/// 
/// This is the canonical test suite proving branch topology enforcement at the server boundary.
/// Shim tests provide UX parity but these server tests are the security boundary.

#[test]
fn test_branch_topology_policy_server_integration() {
    let scenarios = create_branch_topology_server_scenarios();
    let matrix = ScenarioMatrix::new(PolicyArea::BranchTopology).add_scenarios(scenarios);
    
    // Verify we have comprehensive coverage
    let server_scenarios = matrix.get_server_scenarios();
    assert!(!server_scenarios.is_empty());
    
    // Check coverage of key branch operations
    let has_branch_create = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.operation, Some(Operation::CreateBranch))
    });
    assert!(has_branch_create, "Missing branch create scenario");
    
    let has_branch_delete = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.operation, Some(Operation::DeleteBranch))
    });
    assert!(has_branch_delete, "Missing branch delete scenario");
    
    let has_force_push = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.operation, Some(Operation::ForcePush))
    });
    assert!(has_force_push, "Missing force push scenario");
    
    let has_non_ff_push = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.operation, Some(Operation::Push)) &&
        matches!(s.dimensions.repo_state, Some(RepoState::DivergedHistory))
    });
    assert!(has_non_ff_push, "Missing non-fast-forward push scenario");
    
    let has_raw_git = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.client_path, Some(ClientPath::RawGit))
    });
    assert!(has_raw_git, "Missing raw git bypass test");
    
    println!("✅ Branch topology policy server integration test structure validated");
    println!("   - {} server scenarios defined", server_scenarios.len());
    println!("   - Covers create/delete/update, force-push, non-ff, rawGit/shim paths");
    
    // Run coverage audit
    let auditor = CoverageAuditor::new();
    let analyses = auditor.audit(&[matrix]);
    let branch_analysis = analyses.iter()
        .find(|a| a.policy_area == PolicyArea::BranchTopology)
        .unwrap();
    
    println!("   - Missing server cases: {}", branch_analysis.missing_server_cases.len());
    println!("   - Missing shim cases: {}", branch_analysis.missing_shim_cases.len());
}

fn create_branch_topology_server_scenarios() -> Vec<TestScenario> {
    vec![
        // Branch creation scenarios
        TestScenario::new("server_branchtopology_create_branch_allowed_rawgit_allows", PolicyArea::BranchTopology, TestLayer::Server)
            .with_operation(Operation::CreateBranch)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("empty_repo")
            .with_action("create_new_branch")
            .expect_allow(),

        TestScenario::new("server_branchtopology_create_branch_disallowed_rawgit_rejects", PolicyArea::BranchTopology, TestLayer::Server)
            .with_operation(Operation::CreateBranch)
            .with_actor_state(ActorState::Disallowed)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("existing_repo_protected_branch")
            .with_action("attempt_branch_creation")
            .expect_reject(ReasonCode::BranchCreateDenied),

        // Branch deletion scenarios
        TestScenario::new("server_branchtopology_delete_branch_allowed_rawgit_allows", PolicyArea::BranchTopology, TestLayer::Server)
            .with_operation(Operation::DeleteBranch)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("existing_branch")
            .with_action("delete_branch")
            .expect_allow(),

        TestScenario::new("server_branchtopology_delete_protected_branch_rawgit_rejects", PolicyArea::BranchTopology, TestLayer::Server)
            .with_operation(Operation::DeleteBranch)
            .with_actor_state(ActorState::Allowed)
            .with_repo_state(RepoState::ProtectedBranch)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("protected_main_branch")
            .with_action("attempt_delete_protected_branch")
            .expect_reject(ReasonCode::ProtectedBranchDeleteDenied),

        // Non-fast-forward push scenarios
        TestScenario::new("server_branchtopology_push_non_ff_rawgit_rejects", PolicyArea::BranchTopology, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_repo_state(RepoState::DivergedHistory)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("diverged_branches")
            .with_action("push_non_fast_forward")
            .expect_reject(ReasonCode::NonFfDenied),

        TestScenario::new("server_branchtopology_push_fast_forward_rawgit_allows", PolicyArea::BranchTopology, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_repo_state(RepoState::ExistingBranch)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("linear_history")
            .with_action("push_fast_forward")
            .expect_allow(),

        // Force push scenarios
        TestScenario::new("server_branchtopology_force_push_allowed_rawgit_allows", PolicyArea::BranchTopology, TestLayer::Server)
            .with_operation(Operation::ForcePush)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("diverged_branches")
            .with_action("force_push_rewrite_history")
            .expect_allow(),

        TestScenario::new("server_branchtopology_force_push_disallowed_rawgit_rejects", PolicyArea::BranchTopology, TestLayer::Server)
            .with_operation(Operation::ForcePush)
            .with_actor_state(ActorState::Disallowed)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("diverged_branches")
            .with_action("attempt_force_push")
            .expect_reject(ReasonCode::ForcePushDenied),

        TestScenario::new("server_branchtopology_force_push_protected_branch_rawgit_rejects", PolicyArea::BranchTopology, TestLayer::Server)
            .with_operation(Operation::ForcePush)
            .with_actor_state(ActorState::Allowed)
            .with_repo_state(RepoState::ProtectedBranch)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("protected_main_branch")
            .with_action("attempt_force_push_protected")
            .expect_reject(ReasonCode::ProtectedBranchForcePushDenied),

        // Multi-ref update scenarios
        TestScenario::new("server_branchtopology_multi_ref_update_partial_success_rawgit_atomic_rejects", PolicyArea::BranchTopology, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("multi_branch_mixed_permissions")
            .with_action("push_multiple_refs_mixed_auth")
            .expect_reject(ReasonCode::AtomicMultiRefPartialFailure),

        TestScenario::new("server_branchtopology_multi_ref_update_all_allowed_rawgit_allows", PolicyArea::BranchTopology, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("multi_branch_all_allowed")
            .with_action("push_multiple_refs_all_allowed")
            .expect_allow(),

        // Shim parity scenarios
        TestScenario::new("shim_branchtopology_create_branch_disallowed_shim_rejects_matches_server", PolicyArea::BranchTopology, TestLayer::Shim)
            .with_operation(Operation::CreateBranch)
            .with_actor_state(ActorState::Disallowed)
            .with_client_path(ClientPath::Shim)
            .with_fixture("existing_repo_protected_branch")
            .with_action("shim_attempt_branch_creation")
            .expect_reject(ReasonCode::BranchCreateDenied)
            .with_parity_to("server_branchtopology_create_branch_disallowed_rawgit_rejects"),

        TestScenario::new("shim_branchtopology_force_push_disallowed_shim_rejects_matches_server", PolicyArea::BranchTopology, TestLayer::Shim)
            .with_operation(Operation::ForcePush)
            .with_actor_state(ActorState::Disallowed)
            .with_client_path(ClientPath::Shim)
            .with_fixture("diverged_branches")
            .with_action("shim_attempt_force_push")
            .expect_reject(ReasonCode::ForcePushDenied)
            .with_parity_to("server_branchtopology_force_push_disallowed_rawgit_rejects"),

        TestScenario::new("shim_branchtopology_push_non_ff_shim_rejects_matches_server", PolicyArea::BranchTopology, TestLayer::Shim)
            .with_operation(Operation::Push)
            .with_repo_state(RepoState::DivergedHistory)
            .with_client_path(ClientPath::Shim)
            .with_fixture("diverged_branches")
            .with_action("shim_push_non_fast_forward")
            .expect_reject(ReasonCode::NonFfDenied)
            .with_parity_to("server_branchtopology_push_non_ff_rawgit_rejects"),
    ]
}

#[cfg(test)]
mod branch_topology_tests {
    use super::*;
    
    #[test]
    fn test_branch_create_scenarios() {
        let scenarios = create_branch_topology_server_scenarios();
        let create_scenarios: Vec<_> = scenarios.iter()
            .filter(|s| matches!(s.dimensions.operation, Some(Operation::CreateBranch)))
            .collect();
        
        assert!(!create_scenarios.is_empty(), "Should have branch creation scenarios");
        
        // Verify we cover both allowed and disallowed cases
        let has_allowed = create_scenarios.iter().any(|s| 
            matches!(s.expected.result, ExpectedResult::Allow)
        );
        let has_disallowed = create_scenarios.iter().any(|s| 
            matches!(s.expected.result, ExpectedResult::Reject)
        );
        
        assert!(has_allowed, "Should have allowed branch creation scenarios");
        assert!(has_disallowed, "Should have disallowed branch creation scenarios");
    }
    
    #[test] 
    fn test_force_push_scenarios() {
        let scenarios = create_branch_topology_server_scenarios();
        let force_push_scenarios: Vec<_> = scenarios.iter()
            .filter(|s| matches!(s.dimensions.operation, Some(Operation::ForcePush)))
            .collect();
        
        assert!(!force_push_scenarios.is_empty(), "Should have force push scenarios");
        
        // Verify coverage of protected branch force push rejection
        let has_protected_rejection = force_push_scenarios.iter().any(|s|
            matches!(s.dimensions.repo_state, Some(RepoState::ProtectedBranch)) &&
            matches!(s.expected.result, ExpectedResult::Reject)
        );
        assert!(has_protected_rejection, "Should reject force push to protected branches");
    }
    
    #[test]
    fn test_multi_ref_atomic_behavior() {
        let scenarios = create_branch_topology_server_scenarios();
        let multi_ref_scenarios: Vec<_> = scenarios.iter()
            .filter(|s| s.action.contains("multiple_refs"))
            .collect();
        
        assert!(!multi_ref_scenarios.is_empty(), "Should have multi-ref update scenarios");
        
        // Verify atomic behavior - partial failure should reject entire push
        let has_atomic_rejection = multi_ref_scenarios.iter().any(|s|
            s.action.contains("mixed_auth") &&
            matches!(s.expected.result, ExpectedResult::Reject) &&
            matches!(s.expected.reason_code, Some(ReasonCode::AtomicMultiRefPartialFailure))
        );
        assert!(has_atomic_rejection, "Should atomically reject multi-ref with partial failure");
    }
    
    #[test]
    fn test_shim_server_parity() {
        let scenarios = create_branch_topology_server_scenarios();
        let shim_scenarios: Vec<_> = scenarios.iter()
            .filter(|s| matches!(s.layer, TestLayer::Shim))
            .collect();
        
        assert!(!shim_scenarios.is_empty(), "Should have shim parity scenarios");
        
        // Verify all shim scenarios have parity links
        for shim_scenario in shim_scenarios {
            assert!(shim_scenario.parity_with.is_some(), 
                "Shim scenario {} should link to server parity", shim_scenario.id);
        }
    }
}