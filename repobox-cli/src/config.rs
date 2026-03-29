// REPO-030: CLI configuration validation and migration framework
//
// This module provides configuration validation, migration, and initialization
// for repobox CLI configuration files. It ensures configuration integrity
// and provides helpful migration paths between versions.

use crate::errors::{CliError, CliResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// Configuration file version for migration tracking
const CURRENT_CONFIG_VERSION: &str = "1.0.0";

/// Reserved configuration key names that cannot be used for aliases
const RESERVED_KEYS: &[&str] = &[
    "version", "identities", "groups", "permissions", "repos", "defaults",
    "config", "validate", "migrate", "init", "diff", "help", "--help", "-h",
];

/// EVM address regex pattern
const EVM_ADDRESS_PATTERN: &str = r"^0x[a-fA-F0-9]{40}$";

/// Configuration validation and management
#[derive(Debug)]
pub struct ConfigManager {
    config_path: PathBuf,
    evm_address_regex: Regex,
}

/// Configuration file structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub version: String,
    pub identities: Option<HashMap<String, Identity>>,
    pub groups: Option<HashMap<String, Group>>,
    pub permissions: Option<Vec<Permission>>,
    pub repos: Option<HashMap<String, RepoConfig>>,
    pub defaults: Option<DefaultsConfig>,
    pub aliases: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Identity {
    pub name: String,
    pub email: String,
    pub signing_key: Option<String>,
    pub evm_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub name: String,
    pub members: Vec<String>,
    pub includes: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Permission {
    pub rule_id: String,
    pub actors: Vec<String>,
    pub verbs: Vec<String>,
    pub targets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoConfig {
    pub path: String,
    pub default_branch: Option<String>,
    pub protected_branches: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefaultsConfig {
    pub identity: Option<String>,
    pub signing_required: Option<bool>,
    pub auto_verify: Option<bool>,
}

/// Configuration validation error details
#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub error_type: String,
    pub message: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
}

/// Configuration validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

/// Configuration template types
#[derive(Debug, Clone)]
pub enum ConfigTemplate {
    Personal,
    Team,
    Enterprise,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: PathBuf) -> CliResult<Self> {
        let evm_address_regex = Regex::new(EVM_ADDRESS_PATTERN)
            .map_err(|e| CliError::config_error("invalid EVM address regex pattern", Some(e.to_string())))?;
        
        Ok(ConfigManager {
            config_path,
            evm_address_regex,
        })
    }

    /// Validate configuration file
    pub fn validate(&self, json_output: bool) -> CliResult<()> {
        
        let config = self.load_config()?;
        let result = self.validate_config(&config)?;
        
        if json_output {
            self.output_validation_json(&result)?;
        } else {
            self.output_validation_human(&result)?;
        }
        
        if !result.is_valid {
            return Err(CliError::config_error("configuration validation failed", None));
        }
        
        Ok(())
    }

    /// Migrate configuration from older version
    pub fn migrate(&self, from_version: &str) -> CliResult<()> {
        
        // Create backup before migration
        let backup_path = self.create_backup()?;
        println!("Created backup at: {}", backup_path.display());
        
        let config = self.load_config()?;
        let migrated_config = self.migrate_config(config, from_version)?;
        
        // Show diff
        let original_content = fs::read_to_string(&self.config_path)
            .map_err(|e| CliError::config_error("failed to read original config", Some(e.to_string())))?;
        let migrated_content = serde_yaml::to_string(&migrated_config)
            .map_err(|e| CliError::config_error("failed to serialize migrated config", Some(e.to_string())))?;
        
        self.show_migration_diff(&original_content, &migrated_content)?;
        
        // Save migrated config
        fs::write(&self.config_path, migrated_content)
            .map_err(|e| CliError::config_error("failed to save migrated config", Some(e.to_string())))?;
        
        // Validate migrated config
        let validation_result = self.validate_config(&migrated_config)?;
        if !validation_result.is_valid {
            return Err(CliError::config_error("migrated config failed validation", None));
        }
        
        println!("Migration completed successfully");
        Ok(())
    }

    /// Initialize configuration with template
    pub fn init(&self, template: ConfigTemplate) -> CliResult<()> {
        
        if self.config_path.exists() {
            return Err(CliError::config_error("configuration file already exists", None));
        }
        
        let config = self.create_template_config(template)?;
        let content = serde_yaml::to_string(&config)
            .map_err(|e| CliError::config_error("failed to serialize template config", Some(e.to_string())))?;
        
        // Create directory if needed
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CliError::config_error("failed to create config directory", Some(e.to_string())))?;
        }
        
        fs::write(&self.config_path, content)
            .map_err(|e| CliError::config_error("failed to write config file", Some(e.to_string())))?;
        
        println!("Configuration initialized at: {}", self.config_path.display());
        Ok(())
    }

    /// Show configuration diff against canonical patterns
    pub fn diff_canonical(&self, json_output: bool) -> CliResult<()> {
        
        let config = self.load_config()?;
        let deviations = self.find_canonical_deviations(&config)?;
        
        if json_output {
            self.output_deviations_json(&deviations)?;
        } else {
            self.output_deviations_human(&deviations)?;
        }
        
        Ok(())
    }

    /// Load configuration from file
    fn load_config(&self) -> CliResult<Config> {
        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| CliError::config_error("failed to read config file", Some(e.to_string())))?;
        
        let config: Config = serde_yaml::from_str(&content)
            .map_err(|e| CliError::config_error("failed to parse config file", Some(e.to_string())))?;
        
        Ok(config)
    }

    /// Validate configuration structure and rules
    fn validate_config(&self, config: &Config) -> CliResult<ValidationResult> {
        let mut errors = Vec::new();
        let warnings = Vec::new();
        
        // Check version
        if config.version.is_empty() {
            errors.push(ValidationError {
                error_type: "missing_version".to_string(),
                message: "Configuration version is missing".to_string(),
                location: Some("version".to_string()),
                suggestion: Some(format!("Add version: \"{}\"", CURRENT_CONFIG_VERSION)),
            });
        }
        
        // Validate groups for cycles
        if let Some(groups) = &config.groups {
            if let Err(cycle_errors) = self.validate_group_cycles(groups) {
                errors.extend(cycle_errors);
            }
        }
        
        // Validate permission rules
        if let Some(permissions) = &config.permissions {
            for (idx, perm) in permissions.iter().enumerate() {
                if let Err(perm_errors) = self.validate_permission_rule(perm, idx) {
                    errors.extend(perm_errors);
                }
            }
        }
        
        // Validate EVM addresses
        if let Some(identities) = &config.identities {
            for (name, identity) in identities.iter() {
                if let Some(addr) = &identity.evm_address {
                    if !self.evm_address_regex.is_match(addr) {
                        errors.push(ValidationError {
                            error_type: "invalid_evm_address".to_string(),
                            message: format!("Invalid EVM address format: {}", addr),
                            location: Some(format!("identities.{}.evm_address", name)),
                            suggestion: Some("EVM addresses must start with 0x and contain 40 hex characters".to_string()),
                        });
                    }
                }
            }
        }
        
        // Validate alias names
        if let Some(aliases) = &config.aliases {
            for alias_name in aliases.keys() {
                if RESERVED_KEYS.contains(&alias_name.as_str()) {
                    errors.push(ValidationError {
                        error_type: "reserved_alias_name".to_string(),
                        message: format!("Alias name '{}' is reserved", alias_name),
                        location: Some(format!("aliases.{}", alias_name)),
                        suggestion: Some("Choose a different alias name".to_string()),
                    });
                }
            }
        }
        
        // Check for unknown/deprecated keys
        // This would require more sophisticated schema analysis in a real implementation
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Validate group membership for cycles
    fn validate_group_cycles(&self, groups: &HashMap<String, Group>) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        
        for group_name in groups.keys() {
            if let Err(cycle_error) = self.check_group_cycle(groups, group_name, &mut visited, &mut path) {
                errors.push(cycle_error);
            }
        }
        
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// Check for cycles in group includes
    fn check_group_cycle(
        &self,
        groups: &HashMap<String, Group>,
        group_name: &str,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Result<(), ValidationError> {
        if path.contains(&group_name.to_string()) {
            return Err(ValidationError {
                error_type: "group_cycle".to_string(),
                message: format!("Group membership cycle detected: {}", path.join(" -> ")),
                location: Some(format!("groups.{}.includes", group_name)),
                suggestion: Some("Remove one of the circular includes".to_string()),
            });
        }
        
        if visited.contains(group_name) {
            return Ok(());
        }
        
        visited.insert(group_name.to_string());
        path.push(group_name.to_string());
        
        if let Some(group) = groups.get(group_name) {
            if let Some(includes) = &group.includes {
                for included in includes {
                    self.check_group_cycle(groups, included, visited, path)?;
                }
            }
        }
        
        path.pop();
        Ok(())
    }

    /// Validate permission rule syntax
    fn validate_permission_rule(&self, perm: &Permission, index: usize) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        // Validate verbs
        for verb in &perm.verbs {
            if !self.is_valid_verb(verb) {
                errors.push(ValidationError {
                    error_type: "invalid_permission_verb".to_string(),
                    message: format!("Invalid permission verb: {}", verb),
                    location: Some(format!("permissions[{}].verbs", index)),
                    suggestion: Some("Use valid verbs: push, pull, merge, create, delete".to_string()),
                });
            }
        }
        
        // Validate actors
        for actor in &perm.actors {
            if !self.is_valid_actor_pattern(actor) {
                errors.push(ValidationError {
                    error_type: "invalid_actor_pattern".to_string(),
                    message: format!("Invalid actor pattern: {}", actor),
                    location: Some(format!("permissions[{}].actors", index)),
                    suggestion: Some("Use valid patterns: identity names, group names, or * wildcard".to_string()),
                });
            }
        }
        
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    /// Check if verb is valid
    fn is_valid_verb(&self, verb: &str) -> bool {
        matches!(verb, "push" | "pull" | "merge" | "create" | "delete" | "admin")
    }

    /// Check if actor pattern is valid
    fn is_valid_actor_pattern(&self, actor: &str) -> bool {
        // Simple validation - in real implementation would check against actual identities/groups
        !actor.is_empty() && (actor == "*" || actor.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-'))
    }

    /// Create backup of current configuration
    fn create_backup(&self) -> CliResult<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = self.config_path.with_extension(format!("yml.backup.{}", timestamp));
        
        fs::copy(&self.config_path, &backup_path)
            .map_err(|e| CliError::config_error("failed to create backup", Some(e.to_string())))?;
        
        Ok(backup_path)
    }

    /// Migrate configuration between versions
    fn migrate_config(&self, mut config: Config, from_version: &str) -> CliResult<Config> {
        // Update version
        config.version = CURRENT_CONFIG_VERSION.to_string();
        
        // Version-specific migrations would go here
        match from_version {
            "0.9.0" => {
                // Example migration: rename old field
                // if config.old_field.is_some() {
                //     config.new_field = config.old_field.take();
                // }
            }
            _ => {
                // No migration needed for same version
            }
        }
        
        Ok(config)
    }

    /// Show diff between original and migrated content
    fn show_migration_diff(&self, original: &str, migrated: &str) -> CliResult<()> {
        println!("\nMigration diff:");
        println!("--- original");
        println!("+++ migrated");
        
        let original_lines: Vec<&str> = original.lines().collect();
        let migrated_lines: Vec<&str> = migrated.lines().collect();
        
        // Simple diff output - in real implementation would use proper diff algorithm
        for (i, (orig, mig)) in original_lines.iter().zip(migrated_lines.iter()).enumerate() {
            if orig != mig {
                println!("@@ -{} +{} @@", i + 1, i + 1);
                println!("-{}", orig);
                println!("+{}", mig);
            }
        }
        
        Ok(())
    }

    /// Create template configuration
    fn create_template_config(&self, template: ConfigTemplate) -> CliResult<Config> {
        match template {
            ConfigTemplate::Personal => Ok(Config {
                version: CURRENT_CONFIG_VERSION.to_string(),
                identities: Some({
                    let mut identities = HashMap::new();
                    identities.insert("me".to_string(), Identity {
                        name: "Your Name".to_string(),
                        email: "your.email@example.com".to_string(),
                        signing_key: None,
                        evm_address: None,
                    });
                    identities
                }),
                groups: None,
                permissions: Some(vec![Permission {
                    rule_id: "owner_all".to_string(),
                    actors: vec!["me".to_string()],
                    verbs: vec!["push".to_string(), "pull".to_string(), "merge".to_string()],
                    targets: vec!["*".to_string()],
                }]),
                repos: None,
                defaults: Some(DefaultsConfig {
                    identity: Some("me".to_string()),
                    signing_required: Some(false),
                    auto_verify: Some(true),
                }),
                aliases: None,
            }),
            ConfigTemplate::Team => Ok(Config {
                version: CURRENT_CONFIG_VERSION.to_string(),
                identities: Some({
                    let mut identities = HashMap::new();
                    identities.insert("founder1".to_string(), Identity {
                        name: "Founder One".to_string(),
                        email: "founder1@company.com".to_string(),
                        signing_key: None,
                        evm_address: None,
                    });
                    identities.insert("founder2".to_string(), Identity {
                        name: "Founder Two".to_string(),
                        email: "founder2@company.com".to_string(),
                        signing_key: None,
                        evm_address: None,
                    });
                    identities
                }),
                groups: Some({
                    let mut groups = HashMap::new();
                    groups.insert("founders".to_string(), Group {
                        name: "Founders".to_string(),
                        members: vec!["founder1".to_string(), "founder2".to_string()],
                        includes: None,
                    });
                    groups.insert("contributors".to_string(), Group {
                        name: "Contributors".to_string(),
                        members: vec![],
                        includes: None,
                    });
                    groups
                }),
                permissions: Some(vec![
                    Permission {
                        rule_id: "founders_admin".to_string(),
                        actors: vec!["founders".to_string()],
                        verbs: vec!["push".to_string(), "pull".to_string(), "merge".to_string(), "admin".to_string()],
                        targets: vec!["*".to_string()],
                    },
                    Permission {
                        rule_id: "contributors_read_write".to_string(),
                        actors: vec!["contributors".to_string()],
                        verbs: vec!["push".to_string(), "pull".to_string()],
                        targets: vec!["*".to_string()],
                    },
                ]),
                repos: None,
                defaults: Some(DefaultsConfig {
                    identity: None,
                    signing_required: Some(true),
                    auto_verify: Some(false),
                }),
                aliases: None,
            }),
            ConfigTemplate::Enterprise => Ok(Config {
                version: CURRENT_CONFIG_VERSION.to_string(),
                identities: Some(HashMap::new()),
                groups: Some({
                    let mut groups = HashMap::new();
                    groups.insert("engineering".to_string(), Group {
                        name: "Engineering".to_string(),
                        members: vec![],
                        includes: None,
                    });
                    groups.insert("security".to_string(), Group {
                        name: "Security".to_string(),
                        members: vec![],
                        includes: None,
                    });
                    groups.insert("admins".to_string(), Group {
                        name: "Administrators".to_string(),
                        members: vec![],
                        includes: Some(vec!["security".to_string()]),
                    });
                    groups
                }),
                permissions: Some(vec![
                    Permission {
                        rule_id: "admin_all".to_string(),
                        actors: vec!["admins".to_string()],
                        verbs: vec!["push".to_string(), "pull".to_string(), "merge".to_string(), "admin".to_string()],
                        targets: vec!["*".to_string()],
                    },
                    Permission {
                        rule_id: "engineering_development".to_string(),
                        actors: vec!["engineering".to_string()],
                        verbs: vec!["push".to_string(), "pull".to_string()],
                        targets: vec!["development/*".to_string()],
                    },
                ]),
                repos: None,
                defaults: Some(DefaultsConfig {
                    identity: None,
                    signing_required: Some(true),
                    auto_verify: Some(false),
                }),
                aliases: None,
            }),
        }
    }

    /// Find deviations from canonical configuration patterns
    fn find_canonical_deviations(&self, config: &Config) -> CliResult<Vec<String>> {
        let mut deviations = Vec::new();
        
        // Check for missing signing requirements
        if let Some(defaults) = &config.defaults {
            if defaults.signing_required != Some(true) {
                deviations.push("Canonical pattern: signing should be required by default".to_string());
            }
        }
        
        // Check for overly permissive permissions
        if let Some(permissions) = &config.permissions {
            for perm in permissions {
                if perm.actors.contains(&"*".to_string()) && perm.verbs.contains(&"admin".to_string()) {
                    deviations.push(format!("Rule '{}': avoid wildcard admin permissions", perm.rule_id));
                }
            }
        }
        
        // Check for missing identity constraints
        if let Some(identities) = &config.identities {
            for (name, identity) in identities {
                if identity.evm_address.is_none() {
                    deviations.push(format!("Identity '{}': consider adding EVM address for on-chain verification", name));
                }
            }
        }
        
        Ok(deviations)
    }

    /// Output validation result as JSON
    fn output_validation_json(&self, result: &ValidationResult) -> CliResult<()> {
        let output = serde_json::json!({
            "valid": result.is_valid,
            "errors": result.errors,
            "warnings": result.warnings
        });
        
        println!("{}", serde_json::to_string_pretty(&output)
            .map_err(|e| CliError::config_error("failed to format JSON output", Some(e.to_string())))?);
        
        Ok(())
    }

    /// Output validation result in human-readable format
    fn output_validation_human(&self, result: &ValidationResult) -> CliResult<()> {
        if result.is_valid {
            println!("✅ Configuration is valid");
        } else {
            println!("❌ Configuration validation failed");
        }
        
        for error in &result.errors {
            println!("\n🔴 Error: {}", error.message);
            if let Some(location) = &error.location {
                println!("   Location: {}", location);
            }
            if let Some(suggestion) = &error.suggestion {
                println!("   Suggestion: {}", suggestion);
            }
        }
        
        for warning in &result.warnings {
            println!("\n🟡 Warning: {}", warning.message);
            if let Some(location) = &warning.location {
                println!("   Location: {}", location);
            }
            if let Some(suggestion) = &warning.suggestion {
                println!("   Suggestion: {}", suggestion);
            }
        }
        
        Ok(())
    }

    /// Output deviations as JSON
    fn output_deviations_json(&self, deviations: &[String]) -> CliResult<()> {
        let output = serde_json::json!({
            "deviations": deviations
        });
        
        println!("{}", serde_json::to_string_pretty(&output)
            .map_err(|e| CliError::config_error("failed to format JSON output", Some(e.to_string())))?);
        
        Ok(())
    }

    /// Output deviations in human-readable format
    fn output_deviations_human(&self, deviations: &[String]) -> CliResult<()> {
        if deviations.is_empty() {
            println!("✅ Configuration follows canonical patterns");
        } else {
            println!("📋 Configuration deviations from canonical patterns:");
            for deviation in deviations {
                println!("  • {}", deviation);
            }
        }
        
        Ok(())
    }
}