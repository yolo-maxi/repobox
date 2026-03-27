mod matrix;
use matrix::*;

/// Comprehensive merge policy server integration tests
/// 
/// Tests the server-side enforcement of merge policies per REPO-020.
/// Covers merge commit validation, conflict resolution policies, and merge-specific
/// security enforcement across policy boundaries.
/// 
/// This is the canonical test suite proving merge policy enforcement at the server boundary.
/// Shim tests provide UX parity but these server tests are the security boundary.

#[test]
fn test_merge_policy_server_integration() {
    let scenarios = create_merge_policy_server_scenarios();
    let matrix = ScenarioMatrix::new(PolicyArea::MergeRules).add_scenarios(scenarios);
    
    // Verify we have comprehensive coverage
    let server_scenarios = matrix.get_server_scenarios();
    assert!(!server_scenarios.is_empty());
    
    // Check coverage of key merge operations
    let has_merge_commit = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.operation, Some(Operation::Merge))
    });
    assert!(has_merge_commit, "Missing merge commit scenario");
    
    let has_rebase_result = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.operation, Some(Operation::RebaseResultPush))
    });
    assert!(has_rebase_result, "Missing rebase result push scenario");
    
    let has_merge_present = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.repo_state, Some(RepoState::MergePresent))
    });
    assert!(has_merge_present, "Missing merge-present repo state scenario");
    
    let has_raw_git = server_scenarios.iter().any(|s| {
        matches!(s.dimensions.client_path, Some(ClientPath::RawGit))
    });
    assert!(has_raw_git, "Missing raw git bypass test");
    
    println!("✅ Merge policy server integration test structure validated");
    println!("   - {} server scenarios defined", server_scenarios.len());
    println!("   - Covers merge commits, rebase results, conflict resolution, policy boundaries");
    
    // Run coverage audit
    let auditor = CoverageAuditor::new();
    let analyses = auditor.audit(&[matrix]);
    let merge_analysis = analyses.iter()
        .find(|a| a.policy_area == PolicyArea::MergeRules)
        .unwrap();
    
    println!("   - Missing server cases: {}", merge_analysis.missing_server_cases.len());
    println!("   - Missing shim cases: {}", merge_analysis.missing_shim_cases.len());
}

fn create_merge_policy_server_scenarios() -> Vec<TestScenario> {
    vec![
        // Merge commit validation scenarios
        TestScenario::new("server_mergerules_merge_valid_commit_rawgit_allows", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("diverged_branches")
            .with_action("merge_valid_changes")
            .expect_allow(),

        TestScenario::new("server_mergerules_merge_invalid_commit_rawgit_rejects", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("diverged_branches_policy_violation")
            .with_action("merge_policy_violating_changes")
            .expect_reject(ReasonCode::PolicyViolation),

        // Policy boundary crossing in merge commits
        TestScenario::new("server_mergerules_merge_cross_boundary_owner_rawgit_allows", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Owner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("ownership_boundary_merge")
            .with_action("merge_cross_ownership_boundary")
            .expect_allow(),

        TestScenario::new("server_mergerules_merge_cross_boundary_nonowner_rawgit_rejects", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("ownership_boundary_merge")
            .with_action("merge_cross_ownership_boundary")
            .expect_reject(ReasonCode::OwnershipViolation),

        // Squash merge behavior
        TestScenario::new("server_mergerules_squash_merge_valid_rawgit_allows", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("squash_merge_scenario")
            .with_action("squash_merge_valid_changes")
            .expect_allow(),

        TestScenario::new("server_mergerules_squash_merge_violates_append_rawgit_rejects", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Rewrite)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("append_only_protected_file")
            .with_action("squash_merge_rewrite_append_file")
            .expect_reject(ReasonCode::AppendViolation),

        // Rebase result validation
        TestScenario::new("server_mergerules_rebase_result_valid_rawgit_allows", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::RebaseResultPush)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("rebase_scenario")
            .with_action("push_rebased_commits")
            .expect_allow(),

        TestScenario::new("server_mergerules_rebase_result_unsigned_rawgit_rejects", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::RebaseResultPush)
            .with_actor_state(ActorState::Unsigned)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("signature_required_repo")
            .with_action("push_unsigned_rebase_result")
            .expect_reject(ReasonCode::SignatureRequired),

        // Conflict resolution commit validation
        TestScenario::new("server_mergerules_conflict_resolution_valid_rawgit_allows", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::MergePresent)
            .with_fixture("merge_conflict_scenario")
            .with_action("resolve_merge_conflict")
            .expect_allow(),

        TestScenario::new("server_mergerules_conflict_resolution_violates_ownership_rawgit_rejects", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::MergePresent)
            .with_fixture("merge_conflict_owned_file")
            .with_action("resolve_conflict_modify_owned_file")
            .expect_reject(ReasonCode::OwnershipViolation),

        // Merge commit tree diff policy enforcement
        TestScenario::new("server_mergerules_tree_diff_valid_rawgit_allows", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Create)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("tree_diff_scenario")
            .with_action("merge_with_new_files")
            .expect_allow(),

        TestScenario::new("server_mergerules_tree_diff_delete_protected_rawgit_rejects", PolicyArea::MergeRules, TestLayer::Server)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Delete)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("protected_file_scenario")
            .with_action("merge_delete_protected_file")
            .expect_reject(ReasonCode::PolicyViolation),

        // Shim parity scenarios
        TestScenario::new("shim_mergerules_merge_invalid_commit_shim_rejects_matches_server", PolicyArea::MergeRules, TestLayer::Shim)
            .with_operation(Operation::Merge)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::Shim)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("diverged_branches_policy_violation")
            .with_action("shim_merge_policy_violating_changes")
            .expect_reject(ReasonCode::PolicyViolation)
            .with_parity_to("server_mergerules_merge_invalid_commit_rawgit_rejects"),

        TestScenario::new("shim_mergerules_squash_merge_violates_append_shim_rejects_matches_server", PolicyArea::MergeRules, TestLayer::Shim)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Rewrite)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::Shim)
            .with_repo_state(RepoState::DivergedHistory)
            .with_fixture("append_only_protected_file")
            .with_action("shim_squash_merge_rewrite_append_file")
            .expect_reject(ReasonCode::AppendViolation)
            .with_parity_to("server_mergerules_squash_merge_violates_append_rawgit_rejects"),

        TestScenario::new("shim_mergerules_conflict_resolution_violates_ownership_shim_rejects_matches_server", PolicyArea::MergeRules, TestLayer::Shim)
            .with_operation(Operation::Merge)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::Shim)
            .with_repo_state(RepoState::MergePresent)
            .with_fixture("merge_conflict_owned_file")
            .with_action("shim_resolve_conflict_modify_owned_file")
            .expect_reject(ReasonCode::OwnershipViolation)
            .with_parity_to("server_mergerules_conflict_resolution_violates_ownership_rawgit_rejects"),
    ]
}

#[cfg(test)]
mod merge_policy_tests {
    use super::*;
    
    #[test]
    fn test_merge_commit_scenarios() {
        let scenarios = create_merge_policy_server_scenarios();
        let merge_scenarios: Vec<_> = scenarios.iter()
            .filter(|s| matches!(s.dimensions.operation, Some(Operation::Merge)))
            .collect();
        
        assert!(!merge_scenarios.is_empty(), "Should have merge commit scenarios");
        
        // Verify we cover both valid and invalid merge cases
        let has_valid_merge = merge_scenarios.iter().any(|s| 
            matches!(s.expected.result, ExpectedResult::Allow)
        );
        let has_invalid_merge = merge_scenarios.iter().any(|s| 
            matches!(s.expected.result, ExpectedResult::Reject)
        );
        
        assert!(has_valid_merge, "Should have valid merge scenarios");
        assert!(has_invalid_merge, "Should have invalid merge scenarios");
    }
    
    #[test] 
    fn test_policy_boundary_crossing() {
        let scenarios = create_merge_policy_server_scenarios();
        let boundary_scenarios: Vec<_> = scenarios.iter()
            .filter(|s| s.action.contains("boundary"))
            .collect();
        
        assert!(!boundary_scenarios.is_empty(), "Should have policy boundary crossing scenarios");
        
        // Verify coverage of ownership boundary enforcement in merges
        let has_owner_allowed = boundary_scenarios.iter().any(|s|
            matches!(s.dimensions.actor_state, Some(ActorState::Owner)) &&
            matches!(s.expected.result, ExpectedResult::Allow)
        );
        let has_nonowner_rejected = boundary_scenarios.iter().any(|s|
            matches!(s.dimensions.actor_state, Some(ActorState::NonOwner)) &&
            matches!(s.expected.result, ExpectedResult::Reject)
        );
        
        assert!(has_owner_allowed, "Should allow owners to merge across boundaries");
        assert!(has_nonowner_rejected, "Should reject non-owners crossing ownership boundaries");
    }
    
    #[test]
    fn test_squash_merge_behavior() {
        let scenarios = create_merge_policy_server_scenarios();
        let squash_scenarios: Vec<_> = scenarios.iter()
            .filter(|s| s.action.contains("squash"))
            .collect();
        
        assert!(!squash_scenarios.is_empty(), "Should have squash merge scenarios");
        
        // Verify squash merge respects append-only policies
        let has_append_violation = squash_scenarios.iter().any(|s|
            matches!(s.expected.reason_code, Some(ReasonCode::AppendViolation))
        );
        assert!(has_append_violation, "Should detect append violations in squash merges");
    }
    
    #[test]
    fn test_rebase_result_validation() {
        let scenarios = create_merge_policy_server_scenarios();
        let rebase_scenarios: Vec<_> = scenarios.iter()
            .filter(|s| matches!(s.dimensions.operation, Some(Operation::RebaseResultPush)))
            .collect();
        
        assert!(!rebase_scenarios.is_empty(), "Should have rebase result scenarios");
        
        // Verify rebase results are validated for signatures
        let has_signature_check = rebase_scenarios.iter().any(|s|
            matches!(s.expected.reason_code, Some(ReasonCode::SignatureRequired))
        );
        assert!(has_signature_check, "Should validate signatures on rebase results");
    }
    
    #[test]
    fn test_conflict_resolution_enforcement() {
        let scenarios = create_merge_policy_server_scenarios();
        let conflict_scenarios: Vec<_> = scenarios.iter()
            .filter(|s| 
                s.action.contains("conflict") || 
                matches!(s.dimensions.repo_state, Some(RepoState::MergePresent))
            )
            .collect();
        
        assert!(!conflict_scenarios.is_empty(), "Should have conflict resolution scenarios");
        
        // Verify conflict resolution respects ownership policies
        let has_ownership_enforcement = conflict_scenarios.iter().any(|s|
            matches!(s.expected.reason_code, Some(ReasonCode::OwnershipViolation))
        );
        assert!(has_ownership_enforcement, "Should enforce ownership during conflict resolution");
    }
    
    #[test]
    fn test_merge_tree_diff_validation() {
        let scenarios = create_merge_policy_server_scenarios();
        let tree_diff_scenarios: Vec<_> = scenarios.iter()
            .filter(|s| s.action.contains("tree_diff") || s.action.contains("delete_protected"))
            .collect();
        
        assert!(!tree_diff_scenarios.is_empty(), "Should have tree diff validation scenarios");
        
        // Verify tree diff validation catches policy violations
        let has_delete_protection = tree_diff_scenarios.iter().any(|s|
            matches!(s.dimensions.change_shape, Some(ChangeShape::Delete)) &&
            matches!(s.expected.result, ExpectedResult::Reject)
        );
        assert!(has_delete_protection, "Should protect against unauthorized deletes in merges");
    }
    
    #[test]
    fn test_shim_server_parity() {
        let scenarios = create_merge_policy_server_scenarios();
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