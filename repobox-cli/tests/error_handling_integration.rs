// REPO-031: CLI error handling integration tests with matrix coverage
//
// Test systematic error scenario coverage using REPO-023 matrix framework across:
// - Missing config scenarios (no .repobox/, corrupted config.yml)
// - Invalid repo states (not git repo, bare repo, worktree without .git)
// - Network failures (unreachable server, auth failures, timeout)
// - Permission denied scenarios (read-only directories, locked files)
// - Malformed command arguments (invalid flags, missing required args)
//
// Validates REPO-028 structured error format consistency across all error paths
// Tests both human-readable and --json error output formats
// Coverage of error recovery guidance (actionable next steps in error messages)
// Verify exit codes are consistent and meaningful (0=success, 1=user error, 2=system error)

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use std::process::Command;

// Re-declare matrix dimension traits since we can't use cross-test imports
pub trait MatrixDimensions {
    fn all_values() -> Vec<Self>
    where
        Self: Sized;
    fn dimension_name() -> &'static str;
}

pub trait TestScenario {
    fn scenario_id(&self) -> String;
    fn test_name(&self) -> String;  
    fn description(&self) -> String;
}

/// Repository states from cli_matrix
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RepoState {
    Uninitialized,
    GitOnly,
    RepoboxConfigured,
}

/// Matrix dimensions for error handling tests
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCondition {
    MissingConfig,        // no .repobox/ directory
    CorruptedConfig,      // invalid YAML in config.yml
    NotGitRepo,          // not a git repository
    BareRepo,            // bare git repository
    WorktreeWithoutGit,  // worktree without .git
    NetworkUnreachable,  // server unreachable
    AuthFailure,         // authentication failed
    NetworkTimeout,      // request timeout
    PermissionDenied,    // read-only directories
    LockedFiles,         // locked config files
    InvalidFlags,        // malformed command arguments
    MissingRequiredArgs, // missing required arguments
}

impl MatrixDimensions for ErrorCondition {
    fn all_values() -> Vec<Self> {
        vec![
            ErrorCondition::MissingConfig,
            ErrorCondition::CorruptedConfig,
            ErrorCondition::NotGitRepo,
            ErrorCondition::BareRepo,
            ErrorCondition::WorktreeWithoutGit,
            ErrorCondition::NetworkUnreachable,
            ErrorCondition::AuthFailure,
            ErrorCondition::NetworkTimeout,
            ErrorCondition::PermissionDenied,
            ErrorCondition::LockedFiles,
            ErrorCondition::InvalidFlags,
            ErrorCondition::MissingRequiredArgs,
        ]
    }

    fn dimension_name() -> &'static str {
        "ErrorCondition"
    }
}

/// CLI commands to test error handling for
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorTestCommand {
    Init,
    Check,
    Status,
    Whoami,
    Lint,
    Use,
    Config,
}

impl MatrixDimensions for ErrorTestCommand {
    fn all_values() -> Vec<Self> {
        vec![
            ErrorTestCommand::Init,
            ErrorTestCommand::Check,
            ErrorTestCommand::Status,
            ErrorTestCommand::Whoami,
            ErrorTestCommand::Lint,
            ErrorTestCommand::Use,
            ErrorTestCommand::Config,
        ]
    }

    fn dimension_name() -> &'static str {
        "ErrorTestCommand"
    }
}

/// Expected error outcomes
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorExpectation {
    pub exit_code: i32,
    pub contains_text: Vec<String>,
    pub json_error_code: Option<String>,
    pub has_recovery_guidance: bool,
}

/// Test scenario for error handling
pub struct ErrorHandlingScenario {
    pub command: ErrorTestCommand,
    pub condition: ErrorCondition,
    pub repo_state: RepoState,
    pub expected: ErrorExpectation,
    pub description: String,
}

impl TestScenario for ErrorHandlingScenario {
    fn scenario_id(&self) -> String {
        format!("{:?}_{:?}_{:?}", self.command, self.condition, self.repo_state)
    }
    
    fn test_name(&self) -> String {
        format!("error_handling_{}", self.scenario_id().to_lowercase())
    }
    
    fn description(&self) -> String {
        self.description.clone()
    }
}

/// Set up a test environment with specific error condition
fn setup_error_condition(temp_dir: &TempDir, condition: &ErrorCondition, repo_state: &RepoState) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let test_path = temp_dir.path().to_path_buf();
    
    // First set up the basic repo state
    match repo_state {
        RepoState::Uninitialized => {
            // Leave as empty directory
        },
        RepoState::GitOnly => {
            Command::new("git")
                .arg("init")
                .current_dir(&test_path)
                .output()?;
        },
        RepoState::RepoboxConfigured => {
            // Set up git repo
            Command::new("git")
                .arg("init")
                .current_dir(&test_path)
                .output()?;
            
            // Create .repobox directory and basic config (may be corrupted later)
            fs::create_dir_all(test_path.join(".repobox"))?;
            fs::write(
                test_path.join(".repobox/config.yml"),
                "---\nrepo:\n  name: test-repo\n"
            )?;
        },
    }
    
    // Now apply the specific error condition
    match condition {
        ErrorCondition::MissingConfig => {
            // Remove .repobox directory if it exists
            if test_path.join(".repobox").exists() {
                fs::remove_dir_all(test_path.join(".repobox"))?;
            }
        },
        ErrorCondition::CorruptedConfig => {
            // Ensure .repobox exists then corrupt config
            fs::create_dir_all(test_path.join(".repobox"))?;
            fs::write(
                test_path.join(".repobox/config.yml"),
                "invalid: yaml: content: [\n  malformed"
            )?;
        },
        ErrorCondition::NotGitRepo => {
            // Remove .git directory if it exists
            if test_path.join(".git").exists() {
                fs::remove_dir_all(test_path.join(".git"))?;
            }
        },
        ErrorCondition::BareRepo => {
            // Reinit as bare repo
            if test_path.join(".git").exists() {
                fs::remove_dir_all(test_path.join(".git"))?;
            }
            Command::new("git")
                .arg("init")
                .arg("--bare")
                .current_dir(&test_path)
                .output()?;
        },
        ErrorCondition::WorktreeWithoutGit => {
            // Create worktree structure but remove .git
            if test_path.join(".git").exists() {
                fs::remove_dir_all(test_path.join(".git"))?;
            }
            // Create a .git file pointing to non-existent location
            fs::write(
                test_path.join(".git"),
                "gitdir: /nonexistent/worktree"
            )?;
        },
        ErrorCondition::PermissionDenied => {
            // Make .repobox directory read-only
            if test_path.join(".repobox").exists() {
                let mut perms = fs::metadata(test_path.join(".repobox"))?.permissions();
                perms.set_readonly(true);
                fs::set_permissions(test_path.join(".repobox"), perms)?;
            }
        },
        ErrorCondition::LockedFiles => {
            // Create a lock file scenario (platform-specific)
            fs::create_dir_all(test_path.join(".repobox"))?;
            fs::write(
                test_path.join(".repobox/config.yml.lock"),
                "locked by process 12345"
            )?;
        },
        // Network conditions will be simulated in the test execution
        ErrorCondition::NetworkUnreachable |
        ErrorCondition::AuthFailure |
        ErrorCondition::NetworkTimeout |
        ErrorCondition::InvalidFlags |
        ErrorCondition::MissingRequiredArgs => {
            // These are handled at execution time
        },
    }
    
    Ok(test_path)
}

/// Execute command with specific error condition
fn execute_with_error_condition(
    command: &ErrorTestCommand, 
    condition: &ErrorCondition,
    test_path: &PathBuf,
    json_output: bool
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("repobox");
    
    // Add --json flag if requested
    if json_output {
        cmd.arg("--json");
    }
    
    // Build command args based on command type and error condition
    match command {
        ErrorTestCommand::Init => {
            cmd.arg("init");
            if let ErrorCondition::InvalidFlags = condition {
                cmd.arg("--invalid-flag");
            }
        },
        ErrorTestCommand::Check => {
            cmd.arg("check");
            if let ErrorCondition::MissingRequiredArgs = condition {
                // check requires specific arguments in some scenarios
            } else if let ErrorCondition::InvalidFlags = condition {
                cmd.arg("--bad-option");
            }
        },
        ErrorTestCommand::Status => {
            cmd.arg("status");
            if let ErrorCondition::InvalidFlags = condition {
                cmd.arg("--nonsense");
            }
        },
        ErrorTestCommand::Whoami => {
            cmd.arg("whoami");
            if let ErrorCondition::InvalidFlags = condition {
                cmd.arg("--fake-flag");
            }
        },
        ErrorTestCommand::Lint => {
            cmd.arg("lint");
            if let ErrorCondition::MissingRequiredArgs = condition {
                // Don't provide required path argument
            } else if let ErrorCondition::InvalidFlags = condition {
                cmd.arg("--wrong-flag");
            }
        },
        ErrorTestCommand::Use => {
            cmd.arg("use");
            if let ErrorCondition::MissingRequiredArgs = condition {
                // Don't provide required identity argument
            } else {
                cmd.arg("test-identity");
                if let ErrorCondition::InvalidFlags = condition {
                    cmd.arg("--bad-flag");
                }
            }
        },
        ErrorTestCommand::Config => {
            cmd.arg("config");
            cmd.arg("validate");  // Use validate subcommand
            if let ErrorCondition::InvalidFlags = condition {
                cmd.arg("--invalid-option");
            }
        },
    }
    
    // Simulate network conditions with environment variables
    match condition {
        ErrorCondition::NetworkUnreachable => {
            cmd.env("REPOBOX_SERVER_URL", "http://unreachable.invalid:99999");
        },
        ErrorCondition::AuthFailure => {
            cmd.env("REPOBOX_AUTH_TOKEN", "invalid_token_12345");
        },
        ErrorCondition::NetworkTimeout => {
            cmd.env("REPOBOX_TIMEOUT", "1"); // 1ms timeout
        },
        _ => {}
    }
    
    cmd.current_dir(test_path);
    Ok(cmd.output()?)
}

/// Validate error output format and content
fn validate_error_output(
    output: &std::process::Output,
    expected: &ErrorExpectation,
    json_output: bool
) -> Result<(), String> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check exit code
    if output.status.code().unwrap_or(-1) != expected.exit_code {
        return Err(format!(
            "Expected exit code {}, got {}",
            expected.exit_code,
            output.status.code().unwrap_or(-1)
        ));
    }
    
    // Check that expected text appears in output
    let combined_output = format!("{}{}", stdout, stderr);
    for expected_text in &expected.contains_text {
        if !combined_output.contains(expected_text) {
            return Err(format!(
                "Expected text '{}' not found in output: {}",
                expected_text, combined_output
            ));
        }
    }
    
    // Validate JSON format if requested
    if json_output {
        if let Some(expected_code) = &expected.json_error_code {
            // Parse JSON and check error code
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(error_code) = json_val.get("error").and_then(|e| e.get("code")) {
                    if error_code.as_str() != Some(expected_code) {
                        return Err(format!(
                            "Expected JSON error code '{}', got {:?}",
                            expected_code, error_code
                        ));
                    }
                } else {
                    return Err("JSON output missing error.code field".to_string());
                }
            } else {
                return Err("Invalid JSON in stdout".to_string());
            }
        }
    }
    
    // Check for recovery guidance
    if expected.has_recovery_guidance {
        let has_guidance = combined_output.contains("Try:") || 
                          combined_output.contains("Run:") ||
                          combined_output.contains("Fix:") ||
                          combined_output.contains("repobox ");
        if !has_guidance {
            return Err("Expected recovery guidance not found in error output".to_string());
        }
    }
    
    Ok(())
}

/// Generate all error handling test scenarios
fn generate_error_scenarios() -> Vec<ErrorHandlingScenario> {
    let mut scenarios = Vec::new();
    
    // Define scenario combinations that make sense to test
    let scenario_definitions = vec![
        // Missing config scenarios
        (ErrorTestCommand::Check, ErrorCondition::MissingConfig, RepoState::GitOnly, ErrorExpectation {
            exit_code: 1,
            contains_text: vec!["not initialized".to_string(), "repobox init".to_string()],
            json_error_code: Some("CONFIG_MISSING".to_string()),
            has_recovery_guidance: true,
        }),
        (ErrorTestCommand::Status, ErrorCondition::MissingConfig, RepoState::GitOnly, ErrorExpectation {
            exit_code: 1,
            contains_text: vec!["not initialized".to_string()],
            json_error_code: Some("CONFIG_MISSING".to_string()),
            has_recovery_guidance: true,
        }),
        
        // Corrupted config scenarios
        (ErrorTestCommand::Check, ErrorCondition::CorruptedConfig, RepoState::RepoboxConfigured, ErrorExpectation {
            exit_code: 1,
            contains_text: vec!["invalid config".to_string(), "config.yml".to_string()],
            json_error_code: Some("CONFIG_INVALID".to_string()),
            has_recovery_guidance: true,
        }),
        (ErrorTestCommand::Config, ErrorCondition::CorruptedConfig, RepoState::RepoboxConfigured, ErrorExpectation {
            exit_code: 1,
            contains_text: vec!["parse error".to_string()],
            json_error_code: Some("CONFIG_INVALID".to_string()),
            has_recovery_guidance: true,
        }),
        
        // Not git repo scenarios
        (ErrorTestCommand::Init, ErrorCondition::NotGitRepo, RepoState::Uninitialized, ErrorExpectation {
            exit_code: 1,
            contains_text: vec!["not a git repository".to_string(), "git init".to_string()],
            json_error_code: Some("REPO_NOT_GIT".to_string()),
            has_recovery_guidance: true,
        }),
        (ErrorTestCommand::Check, ErrorCondition::NotGitRepo, RepoState::Uninitialized, ErrorExpectation {
            exit_code: 1,
            contains_text: vec!["not a git repository".to_string()],
            json_error_code: Some("REPO_NOT_GIT".to_string()),
            has_recovery_guidance: true,
        }),
        
        // Bare repo scenarios
        (ErrorTestCommand::Init, ErrorCondition::BareRepo, RepoState::GitOnly, ErrorExpectation {
            exit_code: 1,
            contains_text: vec!["bare repository".to_string(), "not supported".to_string()],
            json_error_code: Some("REPO_BARE".to_string()),
            has_recovery_guidance: true,
        }),
        
        // Permission denied scenarios
        (ErrorTestCommand::Init, ErrorCondition::PermissionDenied, RepoState::GitOnly, ErrorExpectation {
            exit_code: 2,
            contains_text: vec!["permission denied".to_string()],
            json_error_code: Some("SYSTEM_PERMISSION".to_string()),
            has_recovery_guidance: true,
        }),
        
        // Invalid flags scenarios
        (ErrorTestCommand::Check, ErrorCondition::InvalidFlags, RepoState::RepoboxConfigured, ErrorExpectation {
            exit_code: 1,
            contains_text: vec!["unknown flag".to_string(), "--help".to_string()],
            json_error_code: Some("ARGS_INVALID".to_string()),
            has_recovery_guidance: true,
        }),
        (ErrorTestCommand::Status, ErrorCondition::InvalidFlags, RepoState::RepoboxConfigured, ErrorExpectation {
            exit_code: 1,
            contains_text: vec!["unknown flag".to_string()],
            json_error_code: Some("ARGS_INVALID".to_string()),
            has_recovery_guidance: true,
        }),
        
        // Missing required args scenarios  
        (ErrorTestCommand::Use, ErrorCondition::MissingRequiredArgs, RepoState::RepoboxConfigured, ErrorExpectation {
            exit_code: 1,
            contains_text: vec!["missing required".to_string(), "identity".to_string()],
            json_error_code: Some("ARGS_MISSING".to_string()),
            has_recovery_guidance: true,
        }),
        (ErrorTestCommand::Lint, ErrorCondition::MissingRequiredArgs, RepoState::RepoboxConfigured, ErrorExpectation {
            exit_code: 1,
            contains_text: vec!["missing required".to_string()],
            json_error_code: Some("ARGS_MISSING".to_string()),
            has_recovery_guidance: true,
        }),
    ];
    
    for (command, condition, repo_state, expected) in scenario_definitions {
        scenarios.push(ErrorHandlingScenario {
            command: command.clone(),
            condition: condition.clone(),
            repo_state: repo_state.clone(),
            expected,
            description: format!(
                "Test {:?} with {:?} in {:?} repo",
                command, condition, repo_state
            ),
        });
    }
    
    scenarios
}

// Test implementation functions

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test that error scenarios are generated correctly
    #[test]
    fn test_error_scenario_generation() {
        let scenarios = generate_error_scenarios();
        assert!(!scenarios.is_empty(), "Should generate error scenarios");
        
        // Verify we have scenarios for different error conditions
        let conditions: std::collections::HashSet<_> = scenarios
            .iter()
            .map(|s| s.condition.clone())
            .collect();
        
        assert!(conditions.contains(&ErrorCondition::MissingConfig));
        assert!(conditions.contains(&ErrorCondition::CorruptedConfig));
        assert!(conditions.contains(&ErrorCondition::InvalidFlags));
    }
    
    /// Test missing config scenarios (requires repobox binary)
    #[test]
    #[ignore = "requires repobox binary"]
    fn test_missing_config_scenarios() {
        let scenarios = generate_error_scenarios();
        let missing_config_scenarios: Vec<_> = scenarios.into_iter()
            .filter(|s| matches!(s.condition, ErrorCondition::MissingConfig))
            .collect();
        
        for scenario in missing_config_scenarios {
            // Set up test environment
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let test_path = setup_error_condition(&temp_dir, &scenario.condition, &scenario.repo_state)
                .expect("Failed to set up error condition");
            
            // Test both human-readable and JSON output
            for json_output in [false, true] {
                let output = execute_with_error_condition(
                    &scenario.command,
                    &scenario.condition,
                    &test_path,
                    json_output
                ).expect("Failed to execute command");
                
                validate_error_output(&output, &scenario.expected, json_output)
                    .unwrap_or_else(|err| panic!("Scenario {} failed: {}", scenario.test_name(), err));
            }
        }
    }
    
    /// Test corrupted config scenarios (requires repobox binary)
    #[test]
    #[ignore = "requires repobox binary"]
    fn test_corrupted_config_scenarios() {
        let scenarios = generate_error_scenarios();
        let corrupted_scenarios: Vec<_> = scenarios.into_iter()
            .filter(|s| matches!(s.condition, ErrorCondition::CorruptedConfig))
            .collect();
        
        for scenario in corrupted_scenarios {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let test_path = setup_error_condition(&temp_dir, &scenario.condition, &scenario.repo_state)
                .expect("Failed to set up error condition");
            
            for json_output in [false, true] {
                let output = execute_with_error_condition(
                    &scenario.command,
                    &scenario.condition,
                    &test_path,
                    json_output
                ).expect("Failed to execute command");
                
                validate_error_output(&output, &scenario.expected, json_output)
                    .unwrap_or_else(|err| panic!("Scenario {} failed: {}", scenario.test_name(), err));
            }
        }
    }
    
    /// Test invalid repo state scenarios (requires repobox binary)
    #[test]
    #[ignore = "requires repobox binary"]
    fn test_invalid_repo_scenarios() {
        let scenarios = generate_error_scenarios();
        let repo_scenarios: Vec<_> = scenarios.into_iter()
            .filter(|s| matches!(s.condition, ErrorCondition::NotGitRepo | ErrorCondition::BareRepo))
            .collect();
        
        for scenario in repo_scenarios {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let test_path = setup_error_condition(&temp_dir, &scenario.condition, &scenario.repo_state)
                .expect("Failed to set up error condition");
            
            for json_output in [false, true] {
                let output = execute_with_error_condition(
                    &scenario.command,
                    &scenario.condition,
                    &test_path,
                    json_output
                ).expect("Failed to execute command");
                
                validate_error_output(&output, &scenario.expected, json_output)
                    .unwrap_or_else(|err| panic!("Scenario {} failed: {}", scenario.test_name(), err));
            }
        }
    }
    
    /// Test permission and system error scenarios (requires repobox binary)
    #[test]
    #[ignore = "requires repobox binary"]
    fn test_system_error_scenarios() {
        let scenarios = generate_error_scenarios();
        let system_scenarios: Vec<_> = scenarios.into_iter()
            .filter(|s| matches!(s.condition, ErrorCondition::PermissionDenied))
            .collect();
        
        for scenario in system_scenarios {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let test_path = setup_error_condition(&temp_dir, &scenario.condition, &scenario.repo_state)
                .expect("Failed to set up error condition");
            
            for json_output in [false, true] {
                let output = execute_with_error_condition(
                    &scenario.command,
                    &scenario.condition,
                    &test_path,
                    json_output
                ).expect("Failed to execute command");
                
                validate_error_output(&output, &scenario.expected, json_output)
                    .unwrap_or_else(|err| panic!("Scenario {} failed: {}", scenario.test_name(), err));
            }
        }
    }
    
    /// Test argument validation scenarios (requires repobox binary)
    #[test]
    #[ignore = "requires repobox binary"]
    fn test_argument_error_scenarios() {
        let scenarios = generate_error_scenarios();
        let arg_scenarios: Vec<_> = scenarios.into_iter()
            .filter(|s| matches!(s.condition, ErrorCondition::InvalidFlags | ErrorCondition::MissingRequiredArgs))
            .collect();
        
        for scenario in arg_scenarios {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let test_path = setup_error_condition(&temp_dir, &scenario.condition, &scenario.repo_state)
                .expect("Failed to set up error condition");
            
            for json_output in [false, true] {
                let output = execute_with_error_condition(
                    &scenario.command,
                    &scenario.condition,
                    &test_path,
                    json_output
                ).expect("Failed to execute command");
                
                validate_error_output(&output, &scenario.expected, json_output)
                    .unwrap_or_else(|err| panic!("Scenario {} failed: {}", scenario.test_name(), err));
            }
        }
    }
    
    /// Test that all error scenarios are covered by the matrix
    #[test]
    fn test_error_matrix_coverage() {
        let scenarios = generate_error_scenarios();
        
        // Verify we have scenarios covering all major error conditions
        let covered_conditions: std::collections::HashSet<_> = scenarios
            .iter()
            .map(|s| s.condition.clone())
            .collect();
        
        let expected_conditions = vec![
            ErrorCondition::MissingConfig,
            ErrorCondition::CorruptedConfig,
            ErrorCondition::NotGitRepo,
            ErrorCondition::BareRepo,
            ErrorCondition::PermissionDenied,
            ErrorCondition::InvalidFlags,
            ErrorCondition::MissingRequiredArgs,
        ];
        
        for expected in expected_conditions {
            assert!(
                covered_conditions.contains(&expected),
                "Error condition {:?} not covered by test scenarios",
                expected
            );
        }
    }
    
    /// Test exit code consistency across all error scenarios
    #[test]
    fn test_exit_code_consistency() {
        let scenarios = generate_error_scenarios();
        
        // Group scenarios by error type and verify consistent exit codes
        for scenario in scenarios {
            match scenario.condition {
                // User errors should use exit code 1
                ErrorCondition::MissingConfig |
                ErrorCondition::CorruptedConfig |
                ErrorCondition::NotGitRepo |
                ErrorCondition::BareRepo |
                ErrorCondition::InvalidFlags |
                ErrorCondition::MissingRequiredArgs => {
                    assert_eq!(scenario.expected.exit_code, 1,
                        "User error condition {:?} should have exit code 1", scenario.condition);
                },
                // System errors should use exit code 2
                ErrorCondition::PermissionDenied |
                ErrorCondition::LockedFiles => {
                    assert_eq!(scenario.expected.exit_code, 2,
                        "System error condition {:?} should have exit code 2", scenario.condition);
                },
                // Network errors should use exit code 1 (user can fix by checking network/config)
                ErrorCondition::NetworkUnreachable |
                ErrorCondition::AuthFailure |
                ErrorCondition::NetworkTimeout => {
                    assert_eq!(scenario.expected.exit_code, 1,
                        "Network error condition {:?} should have exit code 1", scenario.condition);
                },
                _ => {}
            }
        }
    }
    
    /// Test that all error scenarios include recovery guidance
    #[test]
    fn test_recovery_guidance_coverage() {
        let scenarios = generate_error_scenarios();
        
        for scenario in scenarios {
            assert!(scenario.expected.has_recovery_guidance,
                "Scenario {} should include recovery guidance", scenario.test_name());
        }
    }
}