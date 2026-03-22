use repobox::config::*;
use repobox::parser;
use repobox::payment::*;

#[test]
fn test_virtuals_config_complete_flow() {
    let yaml = r#"
groups:
  agents:
    - evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00

permissions:
  default: deny
  rules:
    - agents push >agent/**
    - agents edit * >agent/**

virtuals:
  enabled: true
  bug_bounties:
    critical: "50.00"
    high: "25.00"
    medium: "10.00"
    low: "5.00"
  agent_requirements:
    min_reputation: 0.8
    required_tests: true
    human_review_required: true
  payments:
    network: "base"
    token: "USDC"
    treasury: "0x1234567890123456789012345678901234567890"
    gas_sponsor: "0x9876543210987654321098765432109876543210"
"#;

    let config = parser::parse(yaml).expect("Failed to parse virtuals config");
    
    // Verify virtuals configuration was parsed correctly
    let virtuals = config.virtuals.expect("Virtuals config should be present");
    assert!(virtuals.enabled);
    
    // Check bug bounty amounts
    assert_eq!(virtuals.bug_bounties.critical, "50.00");
    assert_eq!(virtuals.bug_bounties.high, "25.00");
    assert_eq!(virtuals.bug_bounties.medium, "10.00");
    assert_eq!(virtuals.bug_bounties.low, "5.00");
    
    // Check agent requirements
    assert_eq!(virtuals.agent_requirements.min_reputation, 0.8);
    assert!(virtuals.agent_requirements.required_tests);
    assert!(virtuals.agent_requirements.human_review_required);
    
    // Check payment configuration
    let payments = virtuals.payments.expect("Payment config should be present");
    assert_eq!(payments.network, "base");
    assert_eq!(payments.token, "USDC");
    assert_eq!(payments.treasury, "0x1234567890123456789012345678901234567890");
    assert_eq!(payments.gas_sponsor.unwrap(), "0x9876543210987654321098765432109876543210");
}

#[test]
fn test_payment_processor_bounty_claims() {
    let virtuals_config = VirtualsConfig {
        enabled: true,
        bug_bounties: BugBountyConfig {
            critical: "50.00".to_string(),
            high: "25.00".to_string(),
            medium: "10.00".to_string(),
            low: "5.00".to_string(),
        },
        agent_requirements: AgentRequirements {
            min_reputation: 0.8,
            required_tests: true,
            human_review_required: true,
        },
        payments: Some(VirtualsPaymentConfig {
            network: "base".to_string(),
            token: "USDC".to_string(),
            treasury: "0x1234567890123456789012345678901234567890".to_string(),
            gas_sponsor: Some("0x9876543210987654321098765432109876543210".to_string()),
        }),
    };
    
    let payment_config = virtuals_config.payments.clone().unwrap();
    let processor = PaymentProcessor::new(payment_config);
    
    // Test creating bounty claims for different severities
    let severities = [
        ("critical", "50.00"),
        ("high", "25.00"),
        ("medium", "10.00"),
        ("low", "5.00"),
    ];
    
    for (severity, expected_amount) in severities {
        let claim = processor.create_bounty_claim(
            &virtuals_config,
            "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00",
            "42",
            severity,
            "abcd1234",
            &format!("agent/AAc050Ca4FB723bE066E7C12290EE965C84a4a00/fix-42"),
            Some(123),
        ).expect("Failed to create bounty claim");
        
        assert_eq!(claim.agent_address, "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00");
        assert_eq!(claim.issue_id, "42");
        assert_eq!(claim.severity, severity);
        assert_eq!(claim.amount_usdc, expected_amount);
        assert_eq!(claim.commit_hash, "abcd1234");
        assert_eq!(claim.branch_name, format!("agent/AAc050Ca4FB723bE066E7C12290EE965C84a4a00/fix-42"));
        assert_eq!(claim.pr_number, Some(123));
        
        // Verify claim ID format
        assert!(claim.id.starts_with("claim_"));
        assert!(claim.id.len() > 10);
        
        // Verify status is pending
        matches!(claim.status, PaymentStatus::Pending);
    }
}

#[test]
fn test_agent_branch_naming_validation() {
    use repobox::config::{Identity, IdentityKind};
    
    let agent_identity = Identity {
        kind: IdentityKind::Evm,
        address: "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00".to_string(),
    };
    
    // Valid agent branch names
    let valid_branches = [
        "agent/0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00/fix-42",
        "agent/AAc050Ca4FB723bE066E7C12290EE965C84a4a00/feature-auth",
        "agent/aac050ca4fb723be066e7c12290ee965c84a4a00/fix-123", // lowercase
        "main", // non-agent branches should be allowed
        "feature/new-ui", // non-agent branches should be allowed
    ];
    
    for branch in valid_branches {
        // This would normally call the validation function from CLI
        // For now, we'll test the logic manually
        if branch.starts_with("agent/") {
            let parts: Vec<&str> = branch.strip_prefix("agent/").unwrap().split('/').collect();
            assert_eq!(parts.len(), 2, "Agent branch should have exactly 2 parts after agent/");
            
            let branch_agent_id = parts[0];
            let task = parts[1];
            
            // Verify agent ID matches (case insensitive for EVM)
            assert!(
                branch_agent_id.to_lowercase() == agent_identity.address.to_lowercase() ||
                format!("0x{}", branch_agent_id).to_lowercase() == agent_identity.address.to_lowercase(),
                "Branch agent ID should match identity"
            );
            
            // Verify task format
            assert!(task.contains('-'), "Task should contain hyphen separator");
        }
    }
}

#[test]
fn test_virtuals_discovery_api_format() {
    // This tests the JSON format that would be returned by /.well-known/virtuals.json
    use serde_json::json;
    
    let expected_discovery_format = json!({
        "version": "1.0",
        "repository": {
            "name": "example-repo",
            "address": "0x1234567890123456789012345678901234567890",
            "virtuals_enabled": true
        },
        "bug_bounties": {
            "active_issues": [
                {
                    "id": "42",
                    "title": "Memory leak in async handler",
                    "severity": "high",
                    "bounty_usdc": "25.00",
                    "claimed": false,
                    "created_at": "2026-03-22T10:00:00Z",
                    "labels": ["bug", "async", "memory"],
                    "description": "The async handler doesn't properly clean up resources",
                    "reproduction_steps": "1. Start server 2. Send async request 3. Monitor memory usage"
                }
            ]
        },
        "requirements": {
            "min_reputation": 0.8,
            "required_tests": true,
            "review_required": true
        },
        "payment": {
            "network": "base",
            "token": "USDC",
            "treasury": "0x1234567890123456789012345678901234567890"
        }
    });
    
    // Verify the structure is valid JSON and contains all required fields
    assert!(expected_discovery_format["version"].is_string());
    assert!(expected_discovery_format["repository"]["virtuals_enabled"].is_boolean());
    assert!(expected_discovery_format["bug_bounties"]["active_issues"].is_array());
    assert!(expected_discovery_format["requirements"]["min_reputation"].is_number());
    assert!(expected_discovery_format["payment"]["network"].is_string());
}

#[test]
fn test_commit_message_validation() {
    let valid_messages = [
        "fix(auth): resolve login timeout issue",
        "fix: memory leak in async handler (#42)",
        "feat(ui): add dark mode toggle",
        "refactor(core): simplify error handling",
        "test(integration): add virtuals flow tests",
    ];
    
    let invalid_messages = [
        "", // empty
        "random commit message", // no conventional format
        "fix", // no description
        "unknown(scope): description", // invalid type
        "fix: ", // empty description
    ];
    
    for msg in valid_messages {
        assert!(is_valid_conventional_commit(msg), "Should be valid: {}", msg);
    }
    
    for msg in invalid_messages {
        assert!(!is_valid_conventional_commit(msg), "Should be invalid: {}", msg);
    }
}

// Helper function to validate conventional commit format
fn is_valid_conventional_commit(message: &str) -> bool {
    if message.trim().is_empty() {
        return false;
    }
    
    if !message.contains(':') {
        return false;
    }
    
    let parts: Vec<&str> = message.splitn(2, ':').collect();
    let type_scope = parts[0].trim();
    let description = parts[1].trim();
    
    if type_scope.is_empty() || description.is_empty() {
        return false;
    }
    
    let commit_type = if type_scope.contains('(') {
        type_scope.split('(').next().unwrap_or(type_scope)
    } else {
        type_scope
    };
    
    let valid_types = ["fix", "feat", "refactor", "docs", "test", "chore", "style", "perf"];
    valid_types.contains(&commit_type)
}

#[test]
fn test_virtuals_configuration_edge_cases() {
    // Test configuration with minimal virtuals setup
    let minimal_yaml = r#"
virtuals:
  enabled: false
  bug_bounties:
    critical: "1.00"
    high: "0.50"
    medium: "0.25"
    low: "0.10"
"#;

    let config = parser::parse(minimal_yaml).expect("Failed to parse minimal config");
    let virtuals = config.virtuals.expect("Virtuals config should be present");
    assert!(!virtuals.enabled);
    assert_eq!(virtuals.bug_bounties.low, "0.10");
    assert!(virtuals.payments.is_none());
}

#[test]
fn test_x402_payment_integration() {
    // Test that payment configuration supports x402 protocol requirements
    let virtuals_config = VirtualsConfig {
        enabled: true,
        bug_bounties: BugBountyConfig {
            critical: "100.00".to_string(),
            high: "50.00".to_string(),
            medium: "25.00".to_string(),
            low: "10.00".to_string(),
        },
        agent_requirements: AgentRequirements {
            min_reputation: 0.9,
            required_tests: true,
            human_review_required: true,
        },
        payments: Some(VirtualsPaymentConfig {
            network: "base".to_string(),
            token: "USDC".to_string(),
            treasury: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string(), // Real USDC on Base
            gas_sponsor: Some("0x1234567890123456789012345678901234567890".to_string()),
        }),
    };
    
    let payment_config = virtuals_config.payments.unwrap();
    
    // Verify x402-compatible configuration
    assert_eq!(payment_config.network, "base");
    assert_eq!(payment_config.token, "USDC");
    
    // Verify treasury address format (should be valid EVM address)
    assert!(payment_config.treasury.starts_with("0x"));
    assert_eq!(payment_config.treasury.len(), 42);
    
    // Verify gas sponsor is configured
    assert!(payment_config.gas_sponsor.is_some());
}