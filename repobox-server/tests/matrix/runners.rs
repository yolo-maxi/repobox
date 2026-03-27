use std::process::{Command, Stdio};
use std::net::{SocketAddr, TcpListener};
use std::path::Path;
use std::time::Duration;
use std::thread;

use super::scenario::{TestScenario, ScenarioMatrix, ExpectedResult};
use super::fixtures::{TestFixture, TestActor};
use super::dimensions::ReasonCode;

/// Result of running a test scenario
#[derive(Debug)]
pub struct ScenarioResult {
    pub scenario_id: String,
    pub passed: bool,
    pub actual_result: ExpectedResult,
    pub actual_reason: Option<String>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}

impl ScenarioResult {
    pub fn success(scenario_id: String, execution_time_ms: u64) -> Self {
        Self {
            scenario_id,
            passed: true,
            actual_result: ExpectedResult::Allow,
            actual_reason: None,
            error_message: None,
            execution_time_ms,
        }
    }

    pub fn failure(scenario_id: String, error: String, execution_time_ms: u64) -> Self {
        Self {
            scenario_id,
            passed: false,
            actual_result: ExpectedResult::Reject,
            actual_reason: Some(error.clone()),
            error_message: Some(error),
            execution_time_ms,
        }
    }

    pub fn rejected(scenario_id: String, reason: String, execution_time_ms: u64) -> Self {
        Self {
            scenario_id,
            passed: true, // Rejection can be expected
            actual_result: ExpectedResult::Reject,
            actual_reason: Some(reason),
            error_message: None,
            execution_time_ms,
        }
    }
}

/// Test runner for server integration scenarios
pub struct ServerTestRunner {
    server_bin: String,
    timeout_seconds: u64,
}

impl ServerTestRunner {
    pub fn new() -> Self {
        Self {
            server_bin: "repobox-server".to_string(),
            timeout_seconds: 30,
        }
    }

    pub fn with_binary(mut self, binary_path: impl Into<String>) -> Self {
        self.server_bin = binary_path.into();
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Run a complete scenario matrix
    pub fn run_matrix(&self, matrix: &ScenarioMatrix) -> Vec<ScenarioResult> {
        let server_scenarios = matrix.get_server_scenarios();
        let mut results = Vec::new();

        for scenario in server_scenarios {
            let result = self.run_scenario(scenario);
            results.push(result);
        }

        results
    }

    /// Run a single test scenario
    pub fn run_scenario(&self, scenario: &TestScenario) -> ScenarioResult {
        let start_time = std::time::Instant::now();

        match self.execute_scenario(scenario) {
            Ok(result) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                self.validate_result(scenario, result, execution_time)
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                ScenarioResult::failure(scenario.id.clone(), format!("Execution failed: {}", e), execution_time)
            }
        }
    }

    fn execute_scenario(&self, scenario: &TestScenario) -> Result<TestExecutionResult, Box<dyn std::error::Error>> {
        // Create test fixture based on scenario.fixture
        let fixture = self.create_fixture(&scenario.fixture)?;
        
        // Set up test actor based on scenario dimensions
        let actor = self.create_actor(scenario)?;
        actor.configure_git(fixture.repo_path())?;

        // Start repobox server
        let server_addr = self.start_test_server(fixture.repo_path())?;
        
        // Execute the test action
        let result = self.execute_action(scenario, &fixture, &actor, &server_addr)?;

        Ok(result)
    }

    fn create_fixture(&self, fixture_name: &str) -> Result<TestFixture, Box<dyn std::error::Error>> {
        match fixture_name {
            "ownership_repo" => TestFixture::ownership_repo(),
            "append_only_repo" => TestFixture::append_only_repo(),
            "signature_repo" => TestFixture::signature_repo(),
            "empty_repo" => TestFixture::empty_repo(),
            _ => TestFixture::new(fixture_name),
        }
    }

    fn create_actor(&self, scenario: &TestScenario) -> Result<TestActor, Box<dyn std::error::Error>> {
        use super::dimensions::ActorState;
        
        match scenario.dimensions.actor_state {
            Some(ActorState::Owner) => Ok(TestActor::owner()),
            Some(ActorState::NonOwner) => Ok(TestActor::non_owner()),
            Some(ActorState::Unsigned) => Ok(TestActor::unsigned()),
            Some(ActorState::Signed) => Ok(TestActor::owner()), // Use owner as default signed actor
            Some(ActorState::WrongSigner) => Ok(TestActor::non_owner()), // Use non-owner as wrong signer
            _ => Ok(TestActor::owner()), // Default to owner
        }
    }

    fn start_test_server(&self, repo_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        // Find available port
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        drop(listener);

        let server_addr = format!("127.0.0.1:{}", addr.port());
        
        // Start server in background
        let mut server_cmd = Command::new(&self.server_bin)
            .arg("--bind")
            .arg(&server_addr)
            .arg("--data-dir")
            .arg(repo_path.parent().unwrap())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        // Give server time to start
        thread::sleep(Duration::from_millis(500));

        // Check if server is still running
        if let Ok(Some(exit_status)) = server_cmd.try_wait() {
            return Err(format!("Server failed to start, exit code: {:?}", exit_status).into());
        }

        Ok(server_addr)
    }

    fn execute_action(
        &self,
        scenario: &TestScenario,
        fixture: &TestFixture,
        actor: &TestActor,
        server_addr: &str,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error>> {
        use super::dimensions::{Operation, ChangeShape, ClientPath};

        match scenario.dimensions.operation {
            Some(Operation::Push) => self.execute_push(scenario, fixture, actor, server_addr),
            Some(Operation::CreateBranch) => self.execute_branch_create(scenario, fixture, actor, server_addr),
            Some(Operation::DeleteBranch) => self.execute_branch_delete(scenario, fixture, actor, server_addr),
            Some(Operation::ForcePush) => self.execute_force_push(scenario, fixture, actor, server_addr),
            Some(Operation::Merge) => self.execute_merge(scenario, fixture, actor, server_addr),
            _ => Err("Unsupported operation".into()),
        }
    }

    fn execute_push(
        &self,
        scenario: &TestScenario,
        fixture: &TestFixture,
        actor: &TestActor,
        server_addr: &str,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error>> {
        use super::dimensions::ChangeShape;

        // Prepare changes based on scenario change shape
        match scenario.dimensions.change_shape {
            Some(ChangeShape::Create) => {
                fixture.add_file("new_file.txt", "new content")?;
            }
            Some(ChangeShape::Modify) => {
                fixture.modify_file("docs/spec.md", "# Modified specification\n")?;
            }
            Some(ChangeShape::Delete) => {
                fixture.delete_file("README.md")?;
            }
            Some(ChangeShape::Rename) => {
                fixture.rename_file("README.md", "README_renamed.md")?;
            }
            Some(ChangeShape::Append) => {
                fixture.append_to_file("logs/events.log", "2026-03-27 08:30:00 New event\n")?;
            }
            Some(ChangeShape::Rewrite) => {
                fixture.modify_file("logs/events.log", "2026-03-27 09:00:00 Complete rewrite\n")?;
            }
            _ => {
                fixture.modify_file("README.md", "# Updated project\n")?;
            }
        }

        // Create commit
        let commit_hash = match actor.private_key {
            Some(_) => fixture.commit_signed("Test commit", actor.private_key.as_ref().unwrap())?,
            None => fixture.commit("Test commit")?,
        };

        // Execute push based on client path
        let push_result = match scenario.dimensions.client_path {
            Some(ClientPath::RawGit) => self.raw_git_push(fixture, server_addr),
            Some(ClientPath::Shim) => self.shim_push(fixture, server_addr),
            _ => self.raw_git_push(fixture, server_addr),
        };

        match push_result {
            Ok(_) => Ok(TestExecutionResult::Allowed),
            Err(e) => Ok(TestExecutionResult::Rejected(e.to_string())),
        }
    }

    fn execute_branch_create(
        &self,
        _scenario: &TestScenario,
        fixture: &TestFixture,
        _actor: &TestActor,
        server_addr: &str,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error>> {
        // Create new branch and push
        Command::new("git")
            .current_dir(fixture.repo_path())
            .args(&["checkout", "-b", "new-feature"])
            .output()?;

        fixture.add_file("feature.txt", "feature content")?;
        fixture.commit("Add feature")?;

        let push_result = self.raw_git_push(fixture, server_addr);
        match push_result {
            Ok(_) => Ok(TestExecutionResult::Allowed),
            Err(e) => Ok(TestExecutionResult::Rejected(e.to_string())),
        }
    }

    fn execute_branch_delete(
        &self,
        _scenario: &TestScenario,
        _fixture: &TestFixture,
        _actor: &TestActor,
        _server_addr: &str,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error>> {
        // TODO: Implement branch deletion test
        Ok(TestExecutionResult::Allowed)
    }

    fn execute_force_push(
        &self,
        _scenario: &TestScenario,
        _fixture: &TestFixture,
        _actor: &TestActor,
        _server_addr: &str,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error>> {
        // TODO: Implement force push test
        Ok(TestExecutionResult::Allowed)
    }

    fn execute_merge(
        &self,
        _scenario: &TestScenario,
        _fixture: &TestFixture,
        _actor: &TestActor,
        _server_addr: &str,
    ) -> Result<TestExecutionResult, Box<dyn std::error::Error>> {
        // TODO: Implement merge test
        Ok(TestExecutionResult::Allowed)
    }

    fn raw_git_push(&self, fixture: &TestFixture, server_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let remote_url = format!("http://{}/test-repo", server_addr);
        
        let output = Command::new("git")
            .current_dir(fixture.repo_path())
            .args(&["push", &remote_url, "HEAD:main"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Git push failed: {}", stderr).into());
        }

        Ok(())
    }

    fn shim_push(&self, _fixture: &TestFixture, _server_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement shim-based push using repobox CLI
        Ok(())
    }

    fn validate_result(&self, scenario: &TestScenario, result: TestExecutionResult, execution_time_ms: u64) -> ScenarioResult {
        match (&scenario.expected.result, &result) {
            (ExpectedResult::Allow, TestExecutionResult::Allowed) => {
                ScenarioResult::success(scenario.id.clone(), execution_time_ms)
            }
            (ExpectedResult::Reject, TestExecutionResult::Rejected(reason)) => {
                // Check if reason matches expected reason code
                if let Some(expected_reason) = &scenario.expected.reason_code {
                    if self.reason_matches(expected_reason, reason) {
                        ScenarioResult::rejected(scenario.id.clone(), reason.clone(), execution_time_ms)
                    } else {
                        ScenarioResult::failure(
                            scenario.id.clone(),
                            format!("Wrong rejection reason. Expected: {:?}, Got: {}", expected_reason, reason),
                            execution_time_ms,
                        )
                    }
                } else {
                    ScenarioResult::rejected(scenario.id.clone(), reason.clone(), execution_time_ms)
                }
            }
            (ExpectedResult::Allow, TestExecutionResult::Rejected(reason)) => {
                ScenarioResult::failure(
                    scenario.id.clone(),
                    format!("Expected allow but got reject: {}", reason),
                    execution_time_ms,
                )
            }
            (ExpectedResult::Reject, TestExecutionResult::Allowed) => {
                ScenarioResult::failure(
                    scenario.id.clone(),
                    "Expected reject but operation was allowed".to_string(),
                    execution_time_ms,
                )
            }
        }
    }

    fn reason_matches(&self, expected: &ReasonCode, actual: &str) -> bool {
        let expected_str = match expected {
            ReasonCode::OwnershipViolation => "ownership",
            ReasonCode::AppendViolation => "append",
            ReasonCode::SignatureRequired => "signature",
            ReasonCode::NonFfDenied => "non-fast-forward",
            ReasonCode::BranchCreateDenied => "branch",
            ReasonCode::UnauthorizedAccess => "unauthorized",
            ReasonCode::PolicyViolation => "policy",
        };
        
        actual.to_lowercase().contains(expected_str)
    }
}

/// Result of executing a test action
#[derive(Debug)]
enum TestExecutionResult {
    Allowed,
    Rejected(String),
}

/// Test runner for shim scenarios
pub struct ShimTestRunner {
    cli_bin: String,
}

impl ShimTestRunner {
    pub fn new() -> Self {
        Self {
            cli_bin: "repobox".to_string(),
        }
    }

    pub fn with_binary(mut self, binary_path: impl Into<String>) -> Self {
        self.cli_bin = binary_path.into();
        self
    }

    /// Run shim scenarios from a matrix
    pub fn run_matrix(&self, matrix: &ScenarioMatrix) -> Vec<ScenarioResult> {
        let shim_scenarios = matrix.get_shim_scenarios();
        let mut results = Vec::new();

        for scenario in shim_scenarios {
            let result = self.run_scenario(scenario);
            results.push(result);
        }

        results
    }

    /// Run a single shim scenario
    pub fn run_scenario(&self, scenario: &TestScenario) -> ScenarioResult {
        let start_time = std::time::Instant::now();

        // TODO: Implement shim scenario execution
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        ScenarioResult::success(scenario.id.clone(), execution_time)
    }
}

/// Generate a test report from scenario results
pub fn generate_test_report(results: &[ScenarioResult]) -> String {
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;
    
    let mut report = String::new();
    report.push_str(&format!("# Test Results\n\n"));
    report.push_str(&format!("**Summary:** {}/{} passed ({:.1}%)\n\n", passed, total, (passed as f64 / total as f64) * 100.0));
    
    if failed > 0 {
        report.push_str("## Failed Tests\n\n");
        for result in results.iter().filter(|r| !r.passed) {
            report.push_str(&format!("- **{}**: {}\n", result.scenario_id, result.error_message.as_ref().unwrap_or(&"Unknown error".to_string())));
        }
        report.push_str("\n");
    }
    
    report.push_str("## All Results\n\n");
    for result in results {
        let status = if result.passed { "✅" } else { "❌" };
        report.push_str(&format!("- {} **{}** ({}ms)\n", status, result.scenario_id, result.execution_time_ms));
        if let Some(reason) = &result.actual_reason {
            report.push_str(&format!("  - Reason: {}\n", reason));
        }
    }
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::scenario::*;
    use crate::matrix::dimensions::*;

    #[test]
    fn test_scenario_result_creation() {
        let success = ScenarioResult::success("test_1".to_string(), 100);
        assert!(success.passed);
        assert!(matches!(success.actual_result, ExpectedResult::Allow));
        
        let failure = ScenarioResult::failure("test_2".to_string(), "error".to_string(), 200);
        assert!(!failure.passed);
        assert!(matches!(failure.actual_result, ExpectedResult::Reject));
    }

    #[test]
    fn test_test_report_generation() {
        let results = vec![
            ScenarioResult::success("test_1".to_string(), 100),
            ScenarioResult::failure("test_2".to_string(), "failed".to_string(), 200),
        ];
        
        let report = generate_test_report(&results);
        assert!(report.contains("1/2 passed"));
        assert!(report.contains("Failed Tests"));
        assert!(report.contains("test_2"));
    }
}