use std::collections::HashMap;
use super::dimensions::*;

/// Test scenario declaration for policy matrix testing
#[derive(Debug, Clone)]
pub struct TestScenario {
    pub id: String,
    pub policy_area: PolicyArea,
    pub layer: TestLayer,
    pub dimensions: TestDimensions,
    pub fixture: String,
    pub action: String,
    pub expected: ExpectedOutcome,
    pub parity_with: Option<String>,
    pub covers: Vec<String>,
    pub notes: Option<String>,
}

/// Dimensions for a specific test scenario
#[derive(Debug, Clone, Default)]
pub struct TestDimensions {
    pub operation: Option<Operation>,
    pub repo_state: Option<RepoState>,
    pub change_shape: Option<ChangeShape>,
    pub actor_state: Option<ActorState>,
    pub client_path: Option<ClientPath>,
    pub custom: HashMap<String, String>,
}

/// Expected outcome of a test scenario
#[derive(Debug, Clone)]
pub struct ExpectedOutcome {
    pub result: ExpectedResult,
    pub reason_code: Option<ReasonCode>,
    pub message_includes: Vec<String>,
}

impl TestScenario {
    pub fn new(
        id: impl Into<String>,
        policy_area: PolicyArea,
        layer: TestLayer,
    ) -> Self {
        Self {
            id: id.into(),
            policy_area,
            layer,
            dimensions: TestDimensions::default(),
            fixture: String::new(),
            action: String::new(),
            expected: ExpectedOutcome {
                result: ExpectedResult::Allow,
                reason_code: None,
                message_includes: Vec::new(),
            },
            parity_with: None,
            covers: Vec::new(),
            notes: None,
        }
    }

    pub fn with_operation(mut self, operation: Operation) -> Self {
        self.dimensions.operation = Some(operation);
        self
    }

    pub fn with_repo_state(mut self, repo_state: RepoState) -> Self {
        self.dimensions.repo_state = Some(repo_state);
        self
    }

    pub fn with_change_shape(mut self, change_shape: ChangeShape) -> Self {
        self.dimensions.change_shape = Some(change_shape);
        self
    }

    pub fn with_actor_state(mut self, actor_state: ActorState) -> Self {
        self.dimensions.actor_state = Some(actor_state);
        self
    }

    pub fn with_client_path(mut self, client_path: ClientPath) -> Self {
        self.dimensions.client_path = Some(client_path);
        self
    }

    pub fn with_fixture(mut self, fixture: impl Into<String>) -> Self {
        self.fixture = fixture.into();
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }

    pub fn expect_allow(mut self) -> Self {
        self.expected.result = ExpectedResult::Allow;
        self
    }

    pub fn expect_reject(mut self, reason_code: ReasonCode) -> Self {
        self.expected.result = ExpectedResult::Reject;
        self.expected.reason_code = Some(reason_code);
        self
    }

    pub fn with_message_includes(mut self, messages: Vec<impl Into<String>>) -> Self {
        self.expected.message_includes = messages.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn with_parity_to(mut self, parity_scenario_id: impl Into<String>) -> Self {
        self.parity_with = Some(parity_scenario_id.into());
        self
    }

    pub fn covers(mut self, coverage_items: Vec<impl Into<String>>) -> Self {
        self.covers = coverage_items.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Generate a canonical test name from dimensions
    pub fn canonical_name(&self) -> String {
        let layer = format!("{:?}", self.layer).to_lowercase();
        let policy = format!("{:?}", self.policy_area).to_lowercase();
        
        let operation = self.dimensions.operation
            .map(|op| format!("{:?}", op).to_lowercase())
            .unwrap_or_else(|| "unknown_op".to_string());
        
        let change_shape = self.dimensions.change_shape
            .map(|cs| format!("{:?}", cs).to_lowercase())
            .unwrap_or_else(|| "unknown_change".to_string());
        
        let actor_state = self.dimensions.actor_state
            .map(|as_| format!("{:?}", as_).to_lowercase())
            .unwrap_or_else(|| "unknown_actor".to_string());
        
        let client_path = self.dimensions.client_path
            .map(|cp| format!("{:?}", cp).to_lowercase())
            .unwrap_or_else(|| "unknown_client".to_string());
        
        let result = format!("{:?}", self.expected.result).to_lowercase();
        
        format!("{}/{}/{}_{}_{}_{}_{}", 
                layer, policy, operation, change_shape, actor_state, client_path, result)
    }
}

/// Collection of test scenarios for a policy family
#[derive(Debug, Clone)]
pub struct ScenarioMatrix {
    pub policy_area: PolicyArea,
    pub scenarios: Vec<TestScenario>,
}

impl ScenarioMatrix {
    pub fn new(policy_area: PolicyArea) -> Self {
        Self {
            policy_area,
            scenarios: Vec::new(),
        }
    }

    pub fn add_scenario(mut self, scenario: TestScenario) -> Self {
        self.scenarios.push(scenario);
        self
    }

    pub fn add_scenarios(mut self, scenarios: Vec<TestScenario>) -> Self {
        self.scenarios.extend(scenarios);
        self
    }

    pub fn get_server_scenarios(&self) -> Vec<&TestScenario> {
        self.scenarios.iter()
            .filter(|s| matches!(s.layer, TestLayer::Server))
            .collect()
    }

    pub fn get_shim_scenarios(&self) -> Vec<&TestScenario> {
        self.scenarios.iter()
            .filter(|s| matches!(s.layer, TestLayer::Shim))
            .collect()
    }

    pub fn get_unit_scenarios(&self) -> Vec<&TestScenario> {
        self.scenarios.iter()
            .filter(|s| matches!(s.layer, TestLayer::Unit))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_builder() {
        let scenario = TestScenario::new("test_ownership_modify_reject", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::RawGit)
            .expect_reject(ReasonCode::OwnershipViolation)
            .with_fixture("owned_file_repo")
            .with_action("push_modification")
            .covers(vec!["ownership enforcement", "raw git bypass resistance"])
            .with_notes("Non-owner attempts to modify owned file");

        assert_eq!(scenario.id, "test_ownership_modify_reject");
        assert_eq!(scenario.policy_area, PolicyArea::Ownership);
        assert_eq!(scenario.layer, TestLayer::Server);
        assert!(matches!(scenario.dimensions.operation, Some(Operation::Push)));
        assert!(matches!(scenario.expected.result, ExpectedResult::Reject));
        assert!(matches!(scenario.expected.reason_code, Some(ReasonCode::OwnershipViolation)));
    }

    #[test]
    fn test_canonical_naming() {
        let scenario = TestScenario::new("manual_id", PolicyArea::Ownership, TestLayer::Server)
            .with_operation(Operation::Push)
            .with_change_shape(ChangeShape::Modify)
            .with_actor_state(ActorState::NonOwner)
            .with_client_path(ClientPath::RawGit)
            .expect_reject(ReasonCode::OwnershipViolation);

        let name = scenario.canonical_name();
        assert_eq!(name, "server/ownership/push_modify_nonowner_rawgit_reject");
    }

    #[test]
    fn test_scenario_matrix() {
        let matrix = ScenarioMatrix::new(PolicyArea::Ownership)
            .add_scenario(
                TestScenario::new("server_test", PolicyArea::Ownership, TestLayer::Server)
            )
            .add_scenario(
                TestScenario::new("shim_test", PolicyArea::Ownership, TestLayer::Shim)
            );

        assert_eq!(matrix.scenarios.len(), 2);
        assert_eq!(matrix.get_server_scenarios().len(), 1);
        assert_eq!(matrix.get_shim_scenarios().len(), 1);
        assert_eq!(matrix.get_unit_scenarios().len(), 0);
    }
}