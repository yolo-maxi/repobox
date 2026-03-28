//! CLI Integration Tests
//!
//! This module contains concrete integration tests for the repobox CLI using
//! the matrix framework defined in cli_matrix.rs.
//!
//! Tests are organized by command and cover both success and failure scenarios.

mod cli_matrix;

use cli_matrix::{
    CliCommand, CliTestScenario, ExpectedOutcome, IdentityState, MatrixCoverage, RepoState,
    SetupState,
};

use std::sync::Mutex;
use std::sync::OnceLock;

/// Initialize coverage tracking for all tests
static COVERAGE: OnceLock<Mutex<MatrixCoverage>> = OnceLock::new();

fn get_coverage() -> &'static Mutex<MatrixCoverage> {
    COVERAGE.get_or_init(|| Mutex::new(MatrixCoverage::new()))
}

/// Helper function to run a scenario and track coverage
fn run_scenario(scenario: CliTestScenario) -> Result<(), Box<dyn std::error::Error>> {
    let coverage = get_coverage();
    {
        let mut guard = coverage.lock().unwrap();
        guard.declare_scenario(&scenario);
    }
    
    let result = scenario.execute_and_verify()?;
    
    if result.success {
        let mut guard = coverage.lock().unwrap();
        guard.mark_executed(&scenario);
        Ok(())
    } else {
        Err(format!(
            "Scenario failed: {}\nExpected: {}\nActual output: {}\nStderr: {}",
            result.scenario.description,
            result.scenario.expected_outcome,
            result.output.stdout,
            result.output.stderr
        ).into())
    }
}

#[test]
fn test_status_command() -> Result<(), Box<dyn std::error::Error>> {
    // Test status command in various repository states
    
    // Success case: Clean repo with valid identity
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Status)
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Status command in clean repo with valid identity")
    )?;

    // No identity configured
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Status)
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::None)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::ErrorContains("No identity".to_string()))
            .description("Status command with no identity configured")
    )?;

    // Not a git repo
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Status)
            .repo_state(RepoState::NoRepo)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::ErrorContains("not a git repository".to_string()))
            .description("Status command outside git repository")
    )?;

    Ok(())
}

#[test]
fn test_init_command() -> Result<(), Box<dyn std::error::Error>> {
    // Test init command in various scenarios
    
    // Success case: Initialize in clean git repo
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Init)
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::NotConfigured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Initialize repobox in clean git repository")
    )?;

    // Error case: Not a git repository
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Init)
            .repo_state(RepoState::NoRepo)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::NotConfigured)
            .expected_outcome(ExpectedOutcome::ErrorContains("not a git repository".to_string()))
            .description("Initialize repobox outside git repository")
    )?;

    // Already has config - should require --force
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Init)
            .repo_state(RepoState::HasConfig)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::NotConfigured)
            .expected_outcome(ExpectedOutcome::ErrorContains("already exists".to_string()))
            .description("Initialize repobox where config already exists")
    )?;

    // Force overwrite existing config
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Init)
            .args(vec!["--force".to_string()])
            .repo_state(RepoState::HasConfig)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::NotConfigured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Force initialize repobox overwriting existing config")
    )?;

    Ok(())
}

#[test]
fn test_whoami_command() -> Result<(), Box<dyn std::error::Error>> {
    // Test whoami command identity reporting
    
    // Valid identity
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Whoami)
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::OutputContains("0x1234567890123456789012345678901234567890".to_string()))
            .description("Whoami with valid identity shows address")
    )?;

    // No identity configured
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Whoami)
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::None)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::ErrorContains("No identity".to_string()))
            .description("Whoami with no identity configured")
    )?;

    // Invalid identity file
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Whoami)
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Invalid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::ErrorContains("Invalid identity".to_string()))
            .description("Whoami with corrupted identity file")
    )?;

    Ok(())
}

#[test]
fn test_check_command() -> Result<(), Box<dyn std::error::Error>> {
    // Test check command for permission validation
    
    // Valid check with proper arguments
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Check)
            .args(vec![
                "@alice".to_string(),
                "push".to_string(),
                ">main".to_string(),
            ])
            .repo_state(RepoState::HasConfig)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Check command with valid identity, verb, and target")
    )?;

    // Missing arguments should show help
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Check)
            .args(vec!["@alice".to_string()]) // Missing verb and target
            .repo_state(RepoState::HasConfig)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::Help)
            .description("Check command with missing arguments shows help")
    )?;

    // No config file
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Check)
            .args(vec![
                "@alice".to_string(),
                "push".to_string(),
                ">main".to_string(),
            ])
            .repo_state(RepoState::NoConfig)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::ErrorContains("config".to_string()))
            .description("Check command without config file")
    )?;

    Ok(())
}

#[test]
fn test_lint_command() -> Result<(), Box<dyn std::error::Error>> {
    // Test lint command for config validation
    
    // Valid config file
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Lint)
            .repo_state(RepoState::HasConfig)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Lint command with valid config file")
    )?;

    // No config file
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Lint)
            .repo_state(RepoState::NoConfig)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::ErrorContains("config".to_string()))
            .description("Lint command without config file")
    )?;

    // Not in git repository
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Lint)
            .repo_state(RepoState::NoRepo)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::ErrorContains("not a git repository".to_string()))
            .description("Lint command outside git repository")
    )?;

    Ok(())
}

#[test]
fn test_setup_command() -> Result<(), Box<dyn std::error::Error>> {
    // Test setup command for git interceptor configuration
    
    // Initial setup
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Setup)
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::NotConfigured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Setup repobox as git interceptor")
    )?;

    // Remove setup
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Setup)
            .args(vec!["--remove".to_string()])
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Remove repobox git interceptor setup")
    )?;

    // Binary replacement (dangerous operation)
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Setup)
            .args(vec!["--replace-binary".to_string()])
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::NotConfigured)
            .expected_outcome(ExpectedOutcome::OutputContains("WARNING".to_string()))
            .description("Setup with binary replacement shows warning")
    )?;

    Ok(())
}

#[test]
fn test_use_command() -> Result<(), Box<dyn std::error::Error>> {
    // Test use command for identity switching
    
    // Use valid alias
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Use)
            .args(vec!["test-key".to_string()])
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Use command with valid alias")
    )?;

    // Use EVM address directly
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Use)
            .args(vec!["evm:0x1234567890123456789012345678901234567890".to_string()])
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Use command with EVM address")
    )?;

    // Use non-existent alias
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Use)
            .args(vec!["non-existent".to_string()])
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::ErrorContains("not found".to_string()))
            .description("Use command with non-existent alias")
    )?;

    Ok(())
}

#[test]
fn test_keys_command() -> Result<(), Box<dyn std::error::Error>> {
    // Test keys management command
    
    // List keys
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Keys)
            .args(vec!["list".to_string()])
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::Valid)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("List stored keys")
    )?;

    // Generate new key
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Keys)
            .args(vec!["generate".to_string()])
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::None)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Generate new key pair")
    )?;

    // Generate with alias
    run_scenario(
        CliTestScenario::new()
            .command(CliCommand::Keys)
            .args(vec!["generate".to_string(), "--alias".to_string(), "new-key".to_string()])
            .repo_state(RepoState::Clean)
            .identity_state(IdentityState::None)
            .setup_state(SetupState::Configured)
            .expected_outcome(ExpectedOutcome::Success)
            .description("Generate new key pair with alias")
    )?;

    Ok(())
}

#[test]
fn test_excluded_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    // Document intentional exclusions from the matrix
    
    {
        let mut guard = get_coverage().lock().unwrap();
        guard.declare_scenario(
            &CliTestScenario::new()
                .command(CliCommand::Setup)
                .repo_state(RepoState::Unborn)
                .identity_state(IdentityState::MissingKey)
                .setup_state(SetupState::Partial)
                .expected_outcome(ExpectedOutcome::Success)
                .description("Setup command in unborn repo with missing key - excluded scenario")
                .exclude("Unborn repos with missing keys are edge case not worth testing".to_string())
                .exclude("Setup command doesn't depend on commit history".to_string())
        );

        guard.declare_scenario(
            &CliTestScenario::new()
                .command(CliCommand::Check)
                .repo_state(RepoState::Dirty)
                .identity_state(IdentityState::Invalid)
                .setup_state(SetupState::Partial)
                .expected_outcome(ExpectedOutcome::Success)
                .description("Check command with dirty repo and invalid identity - excluded")
                .exclude("Check command behavior doesn't depend on repo dirty state".to_string())
                .exclude("Invalid identity should fail regardless of other states".to_string())
        );
    }

    Ok(())
}

/// Print coverage report at end of tests
#[test]
fn test_coverage_report() {
    let coverage = get_coverage();
    let guard = coverage.lock().unwrap();
    let report = guard.coverage_report();
    report.print_report();
    
    // Assert minimum coverage thresholds
    assert!(
        report.declaration_coverage_percent() >= 5.0,
        "Declaration coverage too low: {:.1}%", 
        report.declaration_coverage_percent()
    );
    
    assert!(
        report.execution_coverage_percent() >= 80.0,
        "Execution coverage too low: {:.1}%",
        report.execution_coverage_percent()
    );
}

/// Integration test that verifies actual CLI binary existence and basic execution
#[test]
fn test_cli_binary_exists() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;
    
    // Build the CLI binary path
    let cli_binary = std::env::current_dir()?
        .parent()
        .ok_or("No parent directory")?
        .join("target/debug/repobox");

    // Test basic help output
    let output = Command::new(&cli_binary)
        .arg("--help")
        .output()?;
        
    assert!(output.status.success(), "CLI binary should respond to --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Git permission layer for AI agents"), 
            "Help output should contain expected description");
    
    Ok(())
}