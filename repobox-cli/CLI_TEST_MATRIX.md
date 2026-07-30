# CLI Test Matrix Framework

This document describes the test matrix framework for the repobox CLI, implementing the requirements from REPO-023.

## Overview

The CLI test matrix provides systematic, auditable coverage for repobox CLI commands. It's designed to:

- **Define explicit matrix dimensions** for comprehensive testing
- **Track coverage** of command/state combinations  
- **Document intentional exclusions** vs accidental gaps
- **Support both success and failure scenarios**
- **Enable deterministic testing** for agent-friendly CLI behavior

## Matrix Dimensions

The framework organizes tests around four key dimensions:

### 1. Command (`CliCommand`)
All primary CLI commands that users and agents interact with:
- `init` - Initialize repobox in git repository
- `check` - Validate permissions for identity/verb/target
- `lint` - Validate .repobox/config.yml file
- `status` - Show identity, groups, permissions summary
- `whoami` - Display current identity
- `use` - Switch active identity by alias/address
- `setup` - Configure git interceptor (PATH shim)
- `keys` - Manage EVM key pairs
- `identity` - Set/manage identity
- `alias` - Manage local aliases
- `config` - Manage and validate configuration

### 2. Repository State (`RepoState`)
Git repository conditions that affect CLI behavior:
- `Clean` - Clean git repo, no uncommitted changes
- `Staged` - Git repo with staged changes
- `Dirty` - Git repo with unstaged changes  
- `Untracked` - Git repo with untracked files
- `NoRepo` - Not a git repository
- `Unborn` - Git repo with no commits (unborn HEAD)
- `HasConfig` - Git repo with existing .repobox/config.yml
- `NoConfig` - Git repo without .repobox/config.yml

### 3. Identity State (`IdentityState`)
User identity configuration status:
- `Valid` - Valid identity configured with key file
- `None` - No identity configured
- `Invalid` - Corrupted/malformed identity file
- `MissingKey` - Identity config exists but key file missing

### 4. Setup State (`SetupState`)
Git interceptor configuration status:
- `Configured` - Repobox set up as git interceptor
- `NotConfigured` - Not configured as interceptor
- `Partial` - Partially configured (e.g., PATH set, hooks missing)
- `FreshInstall` - Fresh environment with no prior setup
- `AlreadyInstalled` - Setup command has already been completed
- `AlreadyRemoved` - Setup command already removed
- `MissingBackup` - Existing setup with missing backup markers
- `InvalidBackupPath` - Setup metadata is present but corrupted

## Usage

### Writing Tests

Use the `CliTestScenario` builder to define test cases:

```rust
use cli_matrix::*;

let scenario = CliTestScenario::new()
    .command(CliCommand::Check)
    .args(vec!["@alice".to_string(), "push".to_string(), ">main".to_string()])
    .repo_state(RepoState::Clean)
    .identity_state(IdentityState::Valid)
    .setup_state(SetupState::Configured)
    .expected_outcome(ExpectedOutcome::Success)
    .description("Check command with valid identity and clean repo");

// Execute and verify
let result = scenario.execute_and_verify()?;
assert!(result.success);
```

### Expected Outcomes

The framework supports various outcome assertions:

```rust
// Success cases
ExpectedOutcome::Success

// Error cases with specific exit codes
ExpectedOutcome::ErrorCode(1)

// Output content validation
ExpectedOutcome::OutputContains("Current identity:".to_string())
ExpectedOutcome::ErrorContains("No identity configured".to_string())

// Help text detection
ExpectedOutcome::Help

// Interactive prompts
ExpectedOutcome::Prompt
```

### Coverage Tracking

The framework tracks which matrix combinations are tested:

```rust
let mut coverage = MatrixCoverage::new();

// Scenarios are automatically registered when executed
run_scenario(scenario)?;

// Generate coverage report
let report = coverage.coverage_report();
report.print_report();
```

### Documenting Exclusions

Mark scenarios as intentionally excluded with explanations:

```rust
let scenario = CliTestScenario::new()
    .command(CliCommand::Setup)
    .repo_state(RepoState::Unborn) 
    .identity_state(IdentityState::MissingKey)
    .setup_state(SetupState::Partial)
    .exclude("Setup doesn't depend on commit history".to_string())
    .exclude("Edge case not worth testing complexity".to_string());

coverage.declare_scenario(&scenario); // Register exclusion without execution
```

## Test Environment

The framework creates isolated test environments for each scenario:

- **Temporary directories** for each test
- **Git repository setup** matching specified RepoState
- **Identity configuration** in test-specific directories  
- **Environment variables** to isolate config from system

### Repository Setup

Based on `RepoState`, the framework automatically:

- Initializes git repos with proper config
- Creates staged/dirty/untracked files as needed
- Sets up .repobox/config.yml for HasConfig state
- Leaves directories as non-repos for NoRepo state

### Identity Setup

Based on `IdentityState`, the framework:

- Creates valid identity.json + key files for Valid state
- Leaves identity directory empty for None state
- Creates malformed JSON for Invalid state  
- Creates identity config without key file for MissingKey state

## Coverage Analysis

The framework provides detailed coverage analysis:

### Metrics

- **Total possible combinations**: 11 commands × 8 repo states × 4 identity states × 8 setup states = 2816
- **Declaration coverage**: % of combinations with declared tests
- **Execution coverage**: % of declared tests that execute successfully

### Reports

Coverage reports show:

```
=== CLI Matrix Coverage Report ===
Total possible combinations: 960
Declared scenarios: 45
Executed scenarios: 42
Declaration coverage: 4.7%
Execution coverage: 93.3%

Missing executions (declared but not executed):
  - check_Dirty_Invalid_Partial
  - status_NoRepo_MissingKey_Configured
  
Declared exclusions:
  - setup_Unborn_MissingKey_Partial: ["Setup doesn't depend on commit history"]
```

## Integration with CI

Add coverage thresholds to prevent regression:

```rust
#[test]
fn test_coverage_thresholds() {
    let coverage = get_coverage();
    let report = coverage.coverage_report();
    
    // Ensure minimum coverage levels
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
```

## Best Practices

### Focus on User-Facing Behavior

Test what users and agents actually experience:
- Command success/failure
- Error message content and actionability
- Help text availability
- Deterministic output for scripting

### Document Exclusions

When skipping matrix combinations, document why:
- Edge cases not worth implementation complexity
- States that don't affect command behavior  
- Combinations that can't occur in practice

### Test Both Paths

For each command, test:
- **Happy path**: Expected success scenarios
- **Error paths**: Various failure modes with helpful messages
- **Edge cases**: Boundary conditions and unusual states

### Keep Tests Fast

- Use tempfile for isolation
- Avoid network calls in unit tests
- Mock external dependencies where possible
- Parallelize independent scenarios

## Current Coverage

As of implementation, the framework covers:

- **11 primary CLI commands** (all user-facing commands)
- **8 repository states** (clean to dirty to no-repo)
- **4 identity states** (valid to missing to corrupted)  
- **8 setup states** (configured, not configured, partial, fresh install, already installed, already removed, missing backup, invalid backup path)

Initial test suite implements ~45 scenarios covering the most important combinations for each command, with intentional exclusions documented for edge cases.

## Future Extensions

The framework is designed for easy extension:

### New Commands
Add to `CliCommand` enum and implement test scenarios

### New Dimensions  
Add new enum types (e.g., `NetworkState`, `ConfigState`) and extend scenario builder

### New Outcomes
Add outcome types for specific assertions (e.g., `JsonOutputValid`, `MetricsContain`)

### Advanced Fixtures
Support for multi-repo scenarios, concurrent operations, or complex git histories