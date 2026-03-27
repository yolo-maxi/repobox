use repobox_server::tests::matrix::*;

/// Demonstration test showing the server-first test matrix framework in action
#[test]
fn test_matrix_framework_demonstration() {
    // Create ownership policy scenarios
    let ownership_scenarios = create_ownership_test_scenarios();
    let ownership_matrix = ScenarioMatrix::new(PolicyArea::Ownership)
        .add_scenarios(ownership_scenarios);

    // Create append-only policy scenarios  
    let append_scenarios = create_append_only_test_scenarios();
    let append_matrix = ScenarioMatrix::new(PolicyArea::AppendOnly)
        .add_scenarios(append_scenarios);

    let matrices = vec![ownership_matrix, append_matrix];

    // Run coverage audit
    let auditor = CoverageAuditor::new();
    let coverage_report = auditor.generate_report(&matrices);
    
    println!("Coverage Analysis:\n{}", coverage_report);

    // Verify we have scenarios for key policy areas
    assert!(!matrices.is_empty());
    
    let ownership_matrix = matrices.iter().find(|m| m.policy_area == PolicyArea::Ownership).unwrap();
    assert!(!ownership_matrix.scenarios.is_empty());
    
    let append_matrix = matrices.iter().find(|m| m.policy_area == PolicyArea::AppendOnly).unwrap();
    assert!(!append_matrix.scenarios.is_empty());

    // Verify canonical naming works
    let first_scenario = &ownership_matrix.scenarios[0];
    let canonical_name = first_scenario.canonical_name();
    assert!(canonical_name.contains("ownership"));
    assert!(canonical_name.contains("server") || canonical_name.contains("shim"));

    println!("✅ Matrix framework demonstration passed");
    println!("   - Created {} ownership scenarios", ownership_matrix.scenarios.len());
    println!("   - Created {} append-only scenarios", append_matrix.scenarios.len());
    println!("   - Generated canonical test names");
    println!("   - Coverage audit completed");
}

fn create_ownership_test_scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario::new("server_ownership_push_modify_owner_rawgit_allows", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Owner)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("ownership_repo")
            .with_action("modify_owned_file")
            .expect_allow()
            .covers(vec!["ownership enforcement for owners", "raw git compatibility"]),

        TestScenario::new("server_ownership_push_modify_nonowner_rawgit_rejects", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("ownership_repo")
            .with_action("modify_owned_file")
            .expect_reject(ReasonCode::OwnershipViolation)
            .with_message_includes(vec!["ownership", "violation"])
            .covers(vec!["ownership enforcement blocking", "raw git bypass resistance"]),

        TestScenario::new("server_ownership_push_delete_owner_rawgit_allows", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Delete)
            .with_actor_state(ActorState::Owner)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("ownership_repo")
            .with_action("delete_owned_file")
            .expect_allow()
            .covers(vec!["ownership enforcement for file deletion"]),

        TestScenario::new("server_ownership_push_delete_nonowner_rawgit_rejects", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Delete)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("ownership_repo")
            .with_action("delete_owned_file")
            .expect_reject(ReasonCode::OwnershipViolation)
            .covers(vec!["ownership enforcement blocking file deletion"]),

        TestScenario::new("shim_ownership_push_modify_nonowner_shim_rejects", PolicyArea::Ownership, TestLayer::Shim)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::Shim)
            .with_fixture("ownership_repo")
            .with_action("modify_owned_file")
            .expect_reject(ReasonCode::OwnershipViolation)
            .with_parity_to("server_ownership_push_modify_nonowner_rawgit_rejects")
            .covers(vec!["shim parity with server enforcement"])
            .with_notes("Shim should block early before network transfer"),
    ]
}

fn create_append_only_test_scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario::new("server_appendonly_push_append_allowed_rawgit_allows", PolicyArea::AppendOnly, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Append)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("append_only_repo")
            .with_action("append_to_log")
            .expect_allow()
            .covers(vec!["append-only policy allowing appends"]),

        TestScenario::new("server_appendonly_push_rewrite_allowed_rawgit_rejects", PolicyArea::AppendOnly, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Rewrite)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("append_only_repo")
            .with_action("rewrite_log")
            .expect_reject(ReasonCode::AppendViolation)
            .with_message_includes(vec!["append", "violation"])
            .covers(vec!["append-only policy blocking rewrites", "server-side append detection"]),

        TestScenario::new("server_appendonly_push_modify_allowed_rawgit_rejects", PolicyArea::AppendOnly, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::RawGit)
            .with_fixture("append_only_repo")
            .with_action("modify_log_middle")
            .expect_reject(ReasonCode::AppendViolation)
            .covers(vec!["append-only policy blocking insertions/edits"]),

        TestScenario::new("shim_appendonly_push_rewrite_allowed_shim_rejects", PolicyArea::AppendOnly, TestLayer::Shim)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Rewrite)
            .with_actor_state(ActorState::Allowed)
            .with_client_path(ClientPath::Shim)
            .with_fixture("append_only_repo")
            .with_action("rewrite_log")
            .expect_reject(ReasonCode::AppendViolation)
            .with_parity_to("server_appendonly_push_rewrite_allowed_rawgit_rejects")
            .covers(vec!["shim early blocking of append violations"])
            .with_notes("Shim should detect rewrite before push"),
    ]
}

/// Integration test showing coverage auditing
#[test] 
fn test_coverage_auditing() {
    // Create partial scenario coverage
    let partial_scenarios = vec![
        TestScenario::new("partial_test", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Owner)
            .with_client_path(ClientPath::RawGit),
    ];
    
    let partial_matrix = ScenarioMatrix::new(PolicyArea::Ownership)
        .add_scenarios(partial_scenarios);

    let auditor = CoverageAuditor::new();
    let analyses = auditor.audit(&[partial_matrix]);
    
    let ownership_analysis = analyses.iter()
        .find(|a| a.policy_area == PolicyArea::Ownership)
        .unwrap();

    // Should have missing coverage since we only added one scenario
    assert!(!ownership_analysis.missing_server_cases.is_empty());
    
    println!("✅ Coverage auditing test passed");
    println!("   - Detected {} missing server cases", ownership_analysis.missing_server_cases.len());
    println!("   - Detected {} missing shim cases", ownership_analysis.missing_shim_cases.len());
}

/// Test demonstrating dimension registry
#[test]
fn test_dimension_registry() {
    let registry = DimensionRegistry::new();
    
    // Verify all required dimensions are registered
    assert!(registry.get_layers().contains(&TestLayer::Server));
    assert!(registry.get_layers().contains(&TestLayer::Shim));
    assert!(registry.get_policy_areas().contains(&PolicyArea::Ownership));
    assert!(registry.get_policy_areas().contains(&PolicyArea::AppendOnly));
    assert!(registry.get_operations().contains(&Operation::Push));
    assert!(registry.get_change_shapes().contains(&ChangeShape::Modify));
    assert!(registry.get_actor_states().contains(&ActorState::Owner));
    assert!(registry.get_client_paths().contains(&ClientPath::RawGit));
    assert!(registry.get_expected_results().contains(&ExpectedResult::Allow));
    assert!(registry.get_reason_codes().contains(&ReasonCode::OwnershipViolation));
    
    println!("✅ Dimension registry test passed");
    println!("   - {} layers registered", registry.get_layers().len());
    println!("   - {} policy areas registered", registry.get_policy_areas().len());
    println!("   - {} operations registered", registry.get_operations().len());
}

/// Test showing test fixture creation
#[cfg(test)]
mod fixture_tests {
    use super::*;

    #[test]
    fn test_ownership_fixture_creation() {
        let fixture = TestFixture::ownership_repo();
        assert!(fixture.is_ok());
        
        let fixture = fixture.unwrap();
        assert!(fixture.repo_path().join("docs/spec.md").exists());
        assert!(fixture.repo_path().join("src/core.rs").exists());
        assert!(fixture.config_path().exists());
        
        println!("✅ Ownership fixture test passed");
    }

    #[test]
    fn test_append_only_fixture_creation() {
        let fixture = TestFixture::append_only_repo();
        assert!(fixture.is_ok());
        
        let fixture = fixture.unwrap();
        assert!(fixture.repo_path().join("logs/events.log").exists());
        assert!(fixture.config_path().exists());
        
        println!("✅ Append-only fixture test passed");
    }

    #[test]
    fn test_actor_creation() {
        let owner = TestActor::owner();
        assert!(owner.address.is_some());
        assert!(owner.private_key.is_some());
        
        let non_owner = TestActor::non_owner();
        assert!(non_owner.address.is_some());
        assert_ne!(owner.address, non_owner.address);
        
        let unsigned = TestActor::unsigned();
        assert!(unsigned.address.is_none());
        
        println!("✅ Actor creation test passed");
    }
}