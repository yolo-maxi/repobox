use std::collections::{HashMap, HashSet};
use super::dimensions::*;
use super::scenario::{TestScenario, ScenarioMatrix};

/// Coverage contract for a policy family
#[derive(Debug, Clone)]
pub struct CoverageContract {
    pub policy_area: PolicyArea,
    pub required_dimensions: RequiredDimensions,
    pub required_combinations: Vec<DimensionCombination>,
    pub exclusions: Vec<ExclusionRule>,
}

/// Dimensions that are relevant to a policy family
#[derive(Debug, Clone)]
pub struct RequiredDimensions {
    pub operations: Vec<Operation>,
    pub change_shapes: Vec<ChangeShape>,
    pub actor_states: Vec<ActorState>,
    pub client_paths: Vec<ClientPath>,
    pub repo_states: Vec<RepoState>,
}

/// A specific combination of dimensions that must be tested
#[derive(Debug, Clone)]
pub struct DimensionCombination {
    pub operation: Operation,
    pub change_shape: Option<ChangeShape>,
    pub actor_state: ActorState,
    pub client_path: ClientPath,
    pub repo_state: Option<RepoState>,
    pub layer: TestLayer,
}

/// Rule for excluding certain combinations as invalid/irrelevant
#[derive(Debug, Clone)]
pub struct ExclusionRule {
    pub reason: String,
    pub pattern: DimensionPattern,
}

/// Pattern for matching dimension combinations
#[derive(Debug, Clone)]
pub struct DimensionPattern {
    pub operation: Option<Operation>,
    pub change_shape: Option<ChangeShape>,
    pub actor_state: Option<ActorState>,
    pub client_path: Option<ClientPath>,
    pub repo_state: Option<RepoState>,
}

/// Result of coverage analysis
#[derive(Debug)]
pub struct CoverageAnalysis {
    pub policy_area: PolicyArea,
    pub missing_server_cases: Vec<DimensionCombination>,
    pub missing_shim_cases: Vec<DimensionCombination>,
    pub orphan_scenarios: Vec<String>,
    pub duplicate_scenarios: Vec<(String, String)>,
}

impl CoverageContract {
    /// Create coverage contract for ownership policy
    pub fn ownership() -> Self {
        Self {
            policy_area: PolicyArea::Ownership,
            required_dimensions: RequiredDimensions {
                operations: vec![Operation::Push, Operation::Merge],
                change_shapes: vec![ChangeShape::Create, ChangeShape::Modify, ChangeShape::Delete, ChangeShape::Rename],
                actor_states: vec![ActorState::Owner, ActorState::NonOwner],
                client_paths: vec![ClientPath::RawGit, ClientPath::Shim],
                repo_states: vec![RepoState::ExistingBranch],
            },
            required_combinations: Self::generate_ownership_combinations(),
            exclusions: vec![
                ExclusionRule {
                    reason: "Create operation doesn't apply to ownership of existing files".to_string(),
                    pattern: DimensionPattern {
                        operation: Some(Operation::Push),
                        change_shape: Some(ChangeShape::Create),
                        actor_state: Some(ActorState::NonOwner),
                        client_path: None,
                        repo_state: None,
                    },
                }
            ],
        }
    }

    /// Create coverage contract for append-only policy
    pub fn append_only() -> Self {
        Self {
            policy_area: PolicyArea::AppendOnly,
            required_dimensions: RequiredDimensions {
                operations: vec![Operation::Push],
                change_shapes: vec![ChangeShape::Append, ChangeShape::Rewrite, ChangeShape::Modify],
                actor_states: vec![ActorState::Allowed],
                client_paths: vec![ClientPath::RawGit, ClientPath::Shim],
                repo_states: vec![RepoState::ExistingBranch],
            },
            required_combinations: Self::generate_append_only_combinations(),
            exclusions: vec![],
        }
    }

    /// Create coverage contract for signature policy
    pub fn signatures() -> Self {
        Self {
            policy_area: PolicyArea::Signatures,
            required_dimensions: RequiredDimensions {
                operations: vec![Operation::Push, Operation::Merge],
                change_shapes: vec![ChangeShape::Create, ChangeShape::Modify],
                actor_states: vec![ActorState::Signed, ActorState::Unsigned, ActorState::WrongSigner],
                client_paths: vec![ClientPath::RawGit, ClientPath::Shim],
                repo_states: vec![RepoState::ExistingBranch, RepoState::EmptyRepo],
            },
            required_combinations: Self::generate_signature_combinations(),
            exclusions: vec![],
        }
    }

    fn generate_ownership_combinations() -> Vec<DimensionCombination> {
        let mut combinations = Vec::new();
        
        for &operation in &[Operation::Push, Operation::Merge] {
            for &change_shape in &[ChangeShape::Modify, ChangeShape::Delete, ChangeShape::Rename] {
                for &actor_state in &[ActorState::Owner, ActorState::NonOwner] {
                    for &client_path in &[ClientPath::RawGit, ClientPath::Shim] {
                        // Server coverage is mandatory
                        combinations.push(DimensionCombination {
                            operation,
                            change_shape: Some(change_shape),
                            actor_state,
                            client_path,
                            repo_state: Some(RepoState::ExistingBranch),
                            layer: TestLayer::Server,
                        });

                        // Shim parity is optional but recommended
                        if matches!(client_path, ClientPath::Shim) {
                            combinations.push(DimensionCombination {
                                operation,
                                change_shape: Some(change_shape),
                                actor_state,
                                client_path,
                                repo_state: Some(RepoState::ExistingBranch),
                                layer: TestLayer::Shim,
                            });
                        }
                    }
                }
            }
        }
        
        combinations
    }

    fn generate_append_only_combinations() -> Vec<DimensionCombination> {
        let mut combinations = Vec::new();
        
        for &change_shape in &[ChangeShape::Append, ChangeShape::Rewrite, ChangeShape::Modify] {
            for &client_path in &[ClientPath::RawGit, ClientPath::Shim] {
                combinations.push(DimensionCombination {
                    operation: Operation::Push,
                    change_shape: Some(change_shape),
                    actor_state: ActorState::Allowed,
                    client_path,
                    repo_state: Some(RepoState::ExistingBranch),
                    layer: TestLayer::Server,
                });
            }
        }
        
        combinations
    }

    fn generate_signature_combinations() -> Vec<DimensionCombination> {
        let mut combinations = Vec::new();
        
        for &operation in &[Operation::Push, Operation::Merge] {
            for &actor_state in &[ActorState::Signed, ActorState::Unsigned, ActorState::WrongSigner] {
                for &client_path in &[ClientPath::RawGit, ClientPath::Shim] {
                    combinations.push(DimensionCombination {
                        operation,
                        change_shape: Some(ChangeShape::Modify),
                        actor_state,
                        client_path,
                        repo_state: Some(RepoState::ExistingBranch),
                        layer: TestLayer::Server,
                    });
                }
            }
        }
        
        combinations
    }
}

/// Coverage auditor for analyzing test completeness
pub struct CoverageAuditor {
    contracts: Vec<CoverageContract>,
}

impl CoverageAuditor {
    pub fn new() -> Self {
        Self {
            contracts: vec![
                CoverageContract::ownership(),
                CoverageContract::append_only(),
                CoverageContract::signatures(),
            ],
        }
    }

    pub fn with_contracts(contracts: Vec<CoverageContract>) -> Self {
        Self { contracts }
    }

    /// Audit coverage for all registered contracts
    pub fn audit(&self, matrices: &[ScenarioMatrix]) -> Vec<CoverageAnalysis> {
        self.contracts.iter()
            .map(|contract| self.audit_policy(contract, matrices))
            .collect()
    }

    /// Audit coverage for a specific policy
    fn audit_policy(&self, contract: &CoverageContract, matrices: &[ScenarioMatrix]) -> CoverageAnalysis {
        let policy_matrix = matrices.iter()
            .find(|m| m.policy_area == contract.policy_area);

        let empty_scenarios = vec![];
        let scenarios = policy_matrix
            .map(|m| &m.scenarios)
            .unwrap_or(&empty_scenarios);

        let mut missing_server_cases = Vec::new();
        let mut missing_shim_cases = Vec::new();

        for required_combo in &contract.required_combinations {
            if !self.is_combination_covered(required_combo, scenarios) {
                match required_combo.layer {
                    TestLayer::Server => missing_server_cases.push(required_combo.clone()),
                    TestLayer::Shim => missing_shim_cases.push(required_combo.clone()),
                    TestLayer::Unit => {}, // Unit tests handled separately
                }
            }
        }

        let orphan_scenarios = self.find_orphan_scenarios(contract, scenarios);
        let duplicate_scenarios = self.find_duplicate_scenarios(scenarios);

        CoverageAnalysis {
            policy_area: contract.policy_area.clone(),
            missing_server_cases,
            missing_shim_cases,
            orphan_scenarios,
            duplicate_scenarios,
        }
    }

    fn is_combination_covered(&self, combo: &DimensionCombination, scenarios: &[TestScenario]) -> bool {
        scenarios.iter().any(|scenario| {
            scenario.layer == combo.layer &&
            scenario.dimensions.operation == Some(combo.operation) &&
            scenario.dimensions.change_shape == combo.change_shape &&
            scenario.dimensions.actor_state == Some(combo.actor_state) &&
            scenario.dimensions.client_path == Some(combo.client_path)
        })
    }

    fn find_orphan_scenarios(&self, contract: &CoverageContract, scenarios: &[TestScenario]) -> Vec<String> {
        scenarios.iter()
            .filter(|scenario| {
                !self.is_scenario_required(scenario, contract)
            })
            .map(|s| s.id.clone())
            .collect()
    }

    fn is_scenario_required(&self, scenario: &TestScenario, contract: &CoverageContract) -> bool {
        contract.required_combinations.iter().any(|combo| {
            scenario.layer == combo.layer &&
            scenario.dimensions.operation == Some(combo.operation) &&
            scenario.dimensions.change_shape == combo.change_shape &&
            scenario.dimensions.actor_state == Some(combo.actor_state) &&
            scenario.dimensions.client_path == Some(combo.client_path)
        })
    }

    fn find_duplicate_scenarios(&self, scenarios: &[TestScenario]) -> Vec<(String, String)> {
        let mut seen_patterns: HashMap<String, String> = HashMap::new();
        let mut duplicates = Vec::new();

        for scenario in scenarios {
            let pattern = format!(
                "{:?}_{:?}_{:?}_{:?}_{:?}",
                scenario.layer,
                scenario.dimensions.operation,
                scenario.dimensions.change_shape,
                scenario.dimensions.actor_state,
                scenario.dimensions.client_path
            );

            if let Some(existing_id) = seen_patterns.get(&pattern) {
                duplicates.push((existing_id.clone(), scenario.id.clone()));
            } else {
                seen_patterns.insert(pattern, scenario.id.clone());
            }
        }

        duplicates
    }

    /// Generate a human-readable coverage report
    pub fn generate_report(&self, matrices: &[ScenarioMatrix]) -> String {
        let analyses = self.audit(matrices);
        let mut report = String::new();

        report.push_str("# Test Coverage Analysis\n\n");

        for analysis in analyses {
            report.push_str(&format!("## {:?} Policy\n\n", analysis.policy_area));

            if !analysis.missing_server_cases.is_empty() {
                report.push_str("### Missing Server Cases:\n");
                for case in &analysis.missing_server_cases {
                    report.push_str(&format!(
                        "- {:?} × {:?} × {:?} × {:?}\n",
                        case.operation,
                        case.change_shape.as_ref().map(|cs| format!("{:?}", cs)).unwrap_or("None".to_string()),
                        case.actor_state,
                        case.client_path
                    ));
                }
                report.push_str("\n");
            }

            if !analysis.missing_shim_cases.is_empty() {
                report.push_str("### Missing Shim Cases:\n");
                for case in &analysis.missing_shim_cases {
                    report.push_str(&format!(
                        "- {:?} × {:?} × {:?} × {:?}\n",
                        case.operation,
                        case.change_shape.as_ref().map(|cs| format!("{:?}", cs)).unwrap_or("None".to_string()),
                        case.actor_state,
                        case.client_path
                    ));
                }
                report.push_str("\n");
            }

            if !analysis.orphan_scenarios.is_empty() {
                report.push_str("### Orphan Scenarios:\n");
                for orphan in &analysis.orphan_scenarios {
                    report.push_str(&format!("- {}\n", orphan));
                }
                report.push_str("\n");
            }

            if !analysis.duplicate_scenarios.is_empty() {
                report.push_str("### Duplicate Scenarios:\n");
                for (existing, duplicate) in &analysis.duplicate_scenarios {
                    report.push_str(&format!("- {} duplicates {}\n", duplicate, existing));
                }
                report.push_str("\n");
            }

            if analysis.missing_server_cases.is_empty() && 
               analysis.missing_shim_cases.is_empty() &&
               analysis.orphan_scenarios.is_empty() &&
               analysis.duplicate_scenarios.is_empty() {
                report.push_str("✅ Full coverage achieved\n\n");
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::scenario::*;

    #[test]
    fn test_ownership_coverage_contract() {
        let contract = CoverageContract::ownership();
        assert_eq!(contract.policy_area, PolicyArea::Ownership);
        assert!(!contract.required_combinations.is_empty());
        
        // Should include basic server scenarios
        let has_server_nonowner_modify = contract.required_combinations.iter().any(|combo| {
            combo.layer == TestLayer::Server &&
            combo.operation == Operation::Push &&
            combo.change_shape == Some(ChangeShape::Modify) &&
            combo.actor_state == ActorState::NonOwner &&
            combo.client_path == ClientPath::RawGit
        });
        assert!(has_server_nonowner_modify);
    }

    #[test]
    fn test_coverage_auditor() {
        let auditor = CoverageAuditor::new();
        
        // Create a minimal scenario matrix for testing
        let scenario = TestScenario::new("test_ownership", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::Owner)
            .with_client_path(ClientPath::RawGit);

        let matrix = ScenarioMatrix::new(PolicyArea::Ownership)
            .add_scenario(scenario);

        let analyses = auditor.audit(&[matrix]);
        assert_eq!(analyses.len(), 3); // ownership, append-only, signatures
        
        let ownership_analysis = analyses.iter()
            .find(|a| a.policy_area == PolicyArea::Ownership)
            .unwrap();
        
        // Should be missing some combinations since we only added one scenario
        assert!(!ownership_analysis.missing_server_cases.is_empty());
    }

    #[test]
    fn test_coverage_report_generation() {
        let auditor = CoverageAuditor::new();
        let matrices = vec![];  // Empty matrices to show missing coverage
        
        let report = auditor.generate_report(&matrices);
        assert!(report.contains("# Test Coverage Analysis"));
        assert!(report.contains("Missing Server Cases"));
    }
}