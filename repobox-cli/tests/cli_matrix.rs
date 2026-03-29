//! CLI Test Matrix Framework for repobox-cli
//!
//! This module provides a systematic, matrix-driven test framework for the repobox CLI.
//! It defines explicit dimensions for CLI testing and provides coverage auditing.
//!
//! ## Matrix Dimensions
//!
//! The CLI test matrix is structured around these dimensions:
//! - **Command**: The CLI command being tested (init, check, status, etc.)
//! - **Repo State**: State of the git repository (clean, dirty, no-repo, etc.)
//! - **Identity State**: Whether user has a valid identity configured
//! - **Setup State**: Whether repobox is set up as git interceptor
//! - **Expected Outcome**: Success, specific error, help text, etc.
//!
//! ## Coverage Contract
//!
//! Each test scenario must declare:
//! - Which matrix dimensions it covers
//! - Which combinations are intentionally excluded
//! - Success and failure paths for each command
//!
//! ## Usage
//!
//! ```rust
//! use cli_matrix::*;
//!
//! // Test a specific matrix cell
//! let scenario = CliTestScenario::new()
//!     .command(CliCommand::Check)
//!     .repo_state(RepoState::Clean)
//!     .identity_state(IdentityState::Valid)
//!     .setup_state(SetupState::Configured)
//!     .expected_outcome(ExpectedOutcome::Success);
//!
//! scenario.execute_and_verify().await?;
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// CLI commands that can be tested
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CliCommand {
    Init,
    Check,
    Lint,
    Status,
    Whoami,
    Use,
    Setup,
    Keys,
    Identity,
    Alias,
}

impl CliCommand {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Init,
            Self::Check,
            Self::Lint,
            Self::Status,
            Self::Whoami,
            Self::Use,
            Self::Setup,
            Self::Keys,
            Self::Identity,
            Self::Alias,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Check => "check",
            Self::Lint => "lint",
            Self::Status => "status",
            Self::Whoami => "whoami",
            Self::Use => "use",
            Self::Setup => "setup",
            Self::Keys => "keys",
            Self::Identity => "identity",
            Self::Alias => "alias",
        }
    }
}

/// Repository state for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RepoState {
    /// Clean git repo with no uncommitted changes
    Clean,
    /// Git repo with staged changes
    Staged,
    /// Git repo with unstaged changes
    Dirty,
    /// Git repo with untracked files
    Untracked,
    /// Not a git repository
    NoRepo,
    /// Git repo but unborn HEAD (no commits)
    Unborn,
    /// Git repo with existing .repobox/config.yml
    HasConfig,
    /// Git repo without .repobox/config.yml
    NoConfig,
}

impl RepoState {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Clean,
            Self::Staged,
            Self::Dirty,
            Self::Untracked,
            Self::NoRepo,
            Self::Unborn,
            Self::HasConfig,
            Self::NoConfig,
        ]
    }
}

/// Identity configuration state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityState {
    /// Valid identity configured
    Valid,
    /// No identity configured
    None,
    /// Invalid/corrupted identity
    Invalid,
    /// Identity exists but key file missing
    MissingKey,
}

impl IdentityState {
    pub fn all() -> Vec<Self> {
        vec![Self::Valid, Self::None, Self::Invalid, Self::MissingKey]
    }
}

/// Setup state for git interception
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SetupState {
    /// Repobox configured as git interceptor
    Configured,
    /// Not configured as git interceptor
    NotConfigured,
    /// Partially configured (e.g., PATH set but hooks missing)
    Partial,
    /// Fresh install state - never been installed before
    FreshInstall,
    /// Already installed state - attempting to install again
    AlreadyInstalled,
    /// Already removed state - attempting to remove again
    AlreadyRemoved,
    /// Missing backup state - trying to restore but no backup exists
    MissingBackup,
    /// Invalid backup path state - backup file is corrupted/missing
    InvalidBackupPath,
}

impl SetupState {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Configured,
            Self::NotConfigured, 
            Self::Partial,
            Self::FreshInstall,
            Self::AlreadyInstalled,
            Self::AlreadyRemoved,
            Self::MissingBackup,
            Self::InvalidBackupPath,
        ]
    }
}

/// Expected test outcomes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpectedOutcome {
    /// Command should succeed (exit code 0)
    Success,
    /// Command should fail with specific exit code
    ErrorCode(i32),
    /// Command should show help text
    Help,
    /// Command should prompt for input (interactive)
    Prompt,
    /// Command should output specific text pattern
    OutputContains(String),
    /// Command should fail with specific error message
    ErrorContains(String),
}

impl fmt::Display for ExpectedOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::ErrorCode(code) => write!(f, "error({})", code),
            Self::Help => write!(f, "help"),
            Self::Prompt => write!(f, "prompt"),
            Self::OutputContains(text) => write!(f, "output_contains({})", text),
            Self::ErrorContains(text) => write!(f, "error_contains({})", text),
        }
    }
}

/// A single test scenario in the CLI matrix
#[derive(Debug, Clone)]
pub struct CliTestScenario {
    pub command: CliCommand,
    pub args: Vec<String>,
    pub repo_state: RepoState,
    pub identity_state: IdentityState,
    pub setup_state: SetupState,
    pub expected_outcome: ExpectedOutcome,
    pub description: String,
    pub exclusions: Vec<String>,
}

impl CliTestScenario {
    pub fn new() -> Self {
        Self {
            command: CliCommand::Status,
            args: vec![],
            repo_state: RepoState::Clean,
            identity_state: IdentityState::Valid,
            setup_state: SetupState::Configured,
            expected_outcome: ExpectedOutcome::Success,
            description: String::new(),
            exclusions: vec![],
        }
    }

    pub fn command(mut self, cmd: CliCommand) -> Self {
        self.command = cmd;
        self
    }

    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn repo_state(mut self, state: RepoState) -> Self {
        self.repo_state = state;
        self
    }

    pub fn identity_state(mut self, state: IdentityState) -> Self {
        self.identity_state = state;
        self
    }

    pub fn setup_state(mut self, state: SetupState) -> Self {
        self.setup_state = state;
        self
    }

    pub fn expected_outcome(mut self, outcome: ExpectedOutcome) -> Self {
        self.expected_outcome = outcome;
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn exclude(mut self, reason: impl Into<String>) -> Self {
        self.exclusions.push(reason.into());
        self
    }

    /// Execute the scenario and verify the outcome
    pub fn execute_and_verify(&self) -> Result<TestResult, Box<dyn std::error::Error>> {
        let test_env = TestEnvironment::new(self)?;
        let output = test_env.run_cli_command(self.command, &self.args)?;
        
        let matches = match &self.expected_outcome {
            ExpectedOutcome::Success => output.status.success(),
            ExpectedOutcome::ErrorCode(expected_code) => {
                output.status.code() == Some(*expected_code)
            }
            ExpectedOutcome::Help => {
                output.stdout.contains("USAGE:") || output.stdout.contains("--help")
            }
            ExpectedOutcome::Prompt => {
                // For now, just check if command is waiting for input
                !output.status.success() && output.stderr.is_empty()
            }
            ExpectedOutcome::OutputContains(text) => {
                output.stdout.contains(text)
            }
            ExpectedOutcome::ErrorContains(text) => {
                output.stderr.contains(text)
            }
        };

        Ok(TestResult {
            scenario: self.clone(),
            success: matches,
            output,
        })
    }

    /// Generate matrix key for coverage tracking
    pub fn matrix_key(&self) -> String {
        format!(
            "{}_{:?}_{:?}_{:?}",
            self.command.as_str(),
            self.repo_state,
            self.identity_state,
            self.setup_state
        )
    }
}

/// Test execution environment
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub repo_path: PathBuf,
}

impl TestEnvironment {
    pub fn new(scenario: &CliTestScenario) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().to_owned();

        let mut env = Self { temp_dir, repo_path };
        env.setup_repo_state(scenario.repo_state)?;
        env.setup_identity_state(scenario.identity_state)?;
        env.setup_setup_state(scenario.setup_state)?;

        Ok(env)
    }

    fn setup_repo_state(&mut self, state: RepoState) -> Result<(), Box<dyn std::error::Error>> {
        match state {
            RepoState::NoRepo => {
                // Do nothing - leave as regular directory
            }
            RepoState::Clean | RepoState::Staged | RepoState::Dirty | RepoState::Untracked => {
                // Initialize git repo
                Command::new("git")
                    .arg("init")
                    .current_dir(&self.repo_path)
                    .output()?;

                Command::new("git")
                    .args(["config", "user.email", "test@example.com"])
                    .current_dir(&self.repo_path)
                    .output()?;

                Command::new("git")
                    .args(["config", "user.name", "Test User"])
                    .current_dir(&self.repo_path)
                    .output()?;

                // Create initial commit unless Unborn state
                if !matches!(state, RepoState::Unborn) {
                    std::fs::write(self.repo_path.join("README.md"), "# Test repo\n")?;
                    Command::new("git")
                        .args(["add", "README.md"])
                        .current_dir(&self.repo_path)
                        .output()?;
                    Command::new("git")
                        .args(["commit", "-m", "Initial commit"])
                        .current_dir(&self.repo_path)
                        .output()?;
                }

                // Set up specific state
                match state {
                    RepoState::Staged => {
                        std::fs::write(self.repo_path.join("staged.txt"), "staged content")?;
                        Command::new("git")
                            .args(["add", "staged.txt"])
                            .current_dir(&self.repo_path)
                            .output()?;
                    }
                    RepoState::Dirty => {
                        std::fs::write(self.repo_path.join("dirty.txt"), "dirty content")?;
                    }
                    RepoState::Untracked => {
                        std::fs::write(self.repo_path.join("untracked.txt"), "untracked content")?;
                    }
                    _ => {}
                }
            }
            RepoState::Unborn => {
                // Initialize git repo but don't create initial commit
                Command::new("git")
                    .arg("init")
                    .current_dir(&self.repo_path)
                    .output()?;
                    
                Command::new("git")
                    .args(["config", "user.email", "test@example.com"])
                    .current_dir(&self.repo_path)
                    .output()?;

                Command::new("git")
                    .args(["config", "user.name", "Test User"])
                    .current_dir(&self.repo_path)
                    .output()?;
            }
            RepoState::HasConfig => {
                // Initialize git repo with .repobox/config.yml
                Command::new("git")
                    .arg("init")
                    .current_dir(&self.repo_path)
                    .output()?;

                std::fs::create_dir_all(self.repo_path.join(".repobox"))?;
                std::fs::write(
                    self.repo_path.join(".repobox/config.yml"),
                    "# Test config\ngroups:\n  founders: []\npermissions:\n  default: allow\n"
                )?;
            }
            RepoState::NoConfig => {
                // Same as Clean but explicitly no config file
                // (handled by default case above)
            }
        }

        Ok(())
    }

    fn setup_identity_state(&mut self, state: IdentityState) -> Result<(), Box<dyn std::error::Error>> {
        // Create .repobox directory in temp location for identity files
        let repobox_dir = self.repo_path.join(".repobox-test");
        std::fs::create_dir_all(&repobox_dir)?;

        match state {
            IdentityState::Valid => {
                // Create valid identity file
                let identity_config = r#"{
                    "current": "test-key",
                    "keys": {
                        "test-key": {
                            "address": "0x1234567890123456789012345678901234567890",
                            "key_file": "test-key.pem"
                        }
                    }
                }"#;
                std::fs::write(repobox_dir.join("identity.json"), identity_config)?;
                
                // Create dummy key file
                std::fs::write(repobox_dir.join("test-key.pem"), "dummy-key-content")?;
            }
            IdentityState::None => {
                // No identity files
            }
            IdentityState::Invalid => {
                // Create malformed identity file
                std::fs::write(repobox_dir.join("identity.json"), "invalid json")?;
            }
            IdentityState::MissingKey => {
                // Valid identity config but missing key file
                let identity_config = r#"{
                    "current": "missing-key",
                    "keys": {
                        "missing-key": {
                            "address": "0x1234567890123456789012345678901234567890",
                            "key_file": "missing.pem"
                        }
                    }
                }"#;
                std::fs::write(repobox_dir.join("identity.json"), identity_config)?;
                // Don't create the key file
            }
        }

        Ok(())
    }

    fn setup_setup_state(&mut self, state: SetupState) -> Result<(), Box<dyn std::error::Error>> {
        match state {
            SetupState::Configured => {
                // For testing purposes, just create a marker file
                std::fs::write(self.repo_path.join(".repobox-setup"), "configured")?;
            }
            SetupState::NotConfigured => {
                // Default state - no setup
            }
            SetupState::Partial => {
                // Create partial setup marker
                std::fs::write(self.repo_path.join(".repobox-setup"), "partial")?;
            }
            SetupState::FreshInstall => {
                // Clean state with no previous installation markers
                // This is the default for new test environments
            }
            SetupState::AlreadyInstalled => {
                // Create markers indicating installation already completed
                std::fs::write(self.repo_path.join(".repobox-setup"), "installed")?;
                std::fs::write(self.repo_path.join(".repobox-installed"), "true")?;
            }
            SetupState::AlreadyRemoved => {
                // Create markers indicating removal already completed
                std::fs::write(self.repo_path.join(".repobox-removed"), "true")?;
            }
            SetupState::MissingBackup => {
                // Setup state where backup files are expected but missing
                std::fs::write(self.repo_path.join(".repobox-setup"), "installed")?;
                // Intentionally don't create backup files
            }
            SetupState::InvalidBackupPath => {
                // Setup state with corrupted/invalid backup files
                std::fs::write(self.repo_path.join(".repobox-setup"), "installed")?;
                std::fs::write(self.repo_path.join(".repobox-backup"), "corrupted-data")?;
            }
        }

        Ok(())
    }

    pub fn run_cli_command(
        &self,
        command: CliCommand,
        args: &[String],
    ) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        // Build the CLI binary path (assuming it's in target/debug)
        let cli_binary = std::env::current_dir()?
            .parent()
            .ok_or("No parent directory")?
            .join("target/debug/repobox");

        let mut cmd = Command::new(&cli_binary);
        cmd.current_dir(&self.repo_path);
        cmd.arg(command.as_str());
        cmd.args(args);

        // Set test environment variables
        cmd.env("REPOBOX_CONFIG_DIR", self.repo_path.join(".repobox-test"));
        
        let output = cmd.output()?;

        Ok(CommandOutput {
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// CLI command execution result
#[derive(Debug)]
pub struct CommandOutput {
    pub status: std::process::ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

/// Test execution result
#[derive(Debug)]
pub struct TestResult {
    pub scenario: CliTestScenario,
    pub success: bool,
    pub output: CommandOutput,
}

/// Coverage tracking for the CLI matrix
#[derive(Default)]
pub struct MatrixCoverage {
    declared_scenarios: HashSet<String>,
    executed_scenarios: HashSet<String>,
    exclusions: HashMap<String, Vec<String>>,
}

impl MatrixCoverage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a test scenario as declared in the test suite
    pub fn declare_scenario(&mut self, scenario: &CliTestScenario) {
        let key = scenario.matrix_key();
        self.declared_scenarios.insert(key.clone());
        
        if !scenario.exclusions.is_empty() {
            self.exclusions.insert(key, scenario.exclusions.clone());
        }
    }

    /// Mark a scenario as executed
    pub fn mark_executed(&mut self, scenario: &CliTestScenario) {
        self.executed_scenarios.insert(scenario.matrix_key());
    }

    /// Calculate total possible matrix combinations
    pub fn total_combinations() -> usize {
        CliCommand::all().len()
            * RepoState::all().len()
            * IdentityState::all().len()
            * SetupState::all().len()
    }

    /// Generate coverage report
    pub fn coverage_report(&self) -> CoverageReport {
        let total_possible = Self::total_combinations();
        let declared_count = self.declared_scenarios.len();
        let executed_count = self.executed_scenarios.len();
        
        let missing_declarations: HashSet<_> = self.executed_scenarios
            .difference(&self.declared_scenarios)
            .cloned()
            .collect();

        let missing_executions: HashSet<_> = self.declared_scenarios
            .difference(&self.executed_scenarios)
            .cloned()
            .collect();

        CoverageReport {
            total_possible,
            declared_count,
            executed_count,
            missing_declarations,
            missing_executions,
            exclusions: self.exclusions.clone(),
        }
    }
}

/// Coverage analysis report
#[derive(Debug)]
pub struct CoverageReport {
    pub total_possible: usize,
    pub declared_count: usize,
    pub executed_count: usize,
    pub missing_declarations: HashSet<String>,
    pub missing_executions: HashSet<String>,
    pub exclusions: HashMap<String, Vec<String>>,
}

impl CoverageReport {
    pub fn declaration_coverage_percent(&self) -> f64 {
        (self.declared_count as f64 / self.total_possible as f64) * 100.0
    }

    pub fn execution_coverage_percent(&self) -> f64 {
        (self.executed_count as f64 / self.declared_count.max(1) as f64) * 100.0
    }

    pub fn print_report(&self) {
        println!("=== CLI Matrix Coverage Report ===");
        println!("Total possible combinations: {}", self.total_possible);
        println!("Declared scenarios: {}", self.declared_count);
        println!("Executed scenarios: {}", self.executed_count);
        println!("Declaration coverage: {:.1}%", self.declaration_coverage_percent());
        println!("Execution coverage: {:.1}%", self.execution_coverage_percent());
        
        if !self.missing_declarations.is_empty() {
            println!("\nMissing declarations (executed but not declared):");
            for scenario in &self.missing_declarations {
                println!("  - {}", scenario);
            }
        }

        if !self.missing_executions.is_empty() {
            println!("\nMissing executions (declared but not executed):");
            for scenario in &self.missing_executions {
                println!("  - {}", scenario);
            }
        }

        if !self.exclusions.is_empty() {
            println!("\nDeclared exclusions:");
            for (scenario, reasons) in &self.exclusions {
                println!("  - {}: {:?}", scenario, reasons);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_dimensions() {
        // Verify all enum variants are accounted for
        assert_eq!(CliCommand::all().len(), 10);
        assert_eq!(RepoState::all().len(), 8);
        assert_eq!(IdentityState::all().len(), 4);
        assert_eq!(SetupState::all().len(), 3);
    }

    #[test]
    fn test_scenario_builder() {
        let scenario = CliTestScenario::new()
            .command(CliCommand::Init)
            .repo_state(RepoState::NoRepo)
            .identity_state(IdentityState::None)
            .setup_state(SetupState::NotConfigured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Initialize repobox in new directory");

        assert_eq!(scenario.command, CliCommand::Init);
        assert_eq!(scenario.repo_state, RepoState::NoRepo);
        assert_eq!(scenario.identity_state, IdentityState::None);
        assert_eq!(scenario.setup_state, SetupState::NotConfigured);
    }

    #[test]
    fn test_matrix_key_generation() {
        let scenario = CliTestScenario::new()
            .command(CliCommand::Check)
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured);

        let key = scenario.matrix_key();
        assert!(key.starts_with("check_"));
        assert!(key.contains("Clean"));
        assert!(key.contains("Valid"));
        assert!(key.contains("Configured"));
    }

    #[test]
    fn test_coverage_tracking() {
        let mut coverage = MatrixCoverage::new();
        
        let scenario = CliTestScenario::new()
            .command(CliCommand::Status)
            .description("Basic status command");
            
        coverage.declare_scenario(&scenario);
        assert_eq!(coverage.declared_scenarios.len(), 1);
        
        coverage.mark_executed(&scenario);
        assert_eq!(coverage.executed_scenarios.len(), 1);
        
        let report = coverage.coverage_report();
        assert_eq!(report.declared_count, 1);
        assert_eq!(report.executed_count, 1);
    }

    #[test]
    fn test_total_combinations() {
        // 10 commands * 8 repo states * 4 identity states * 3 setup states = 960
        assert_eq!(MatrixCoverage::total_combinations(), 960);
    }
}