#[cfg(test)]
mod ens_integration_tests {
    use crate::config::*;
    use crate::engine;
    use crate::parser;
    use crate::resolver::RemoteResolver;

    #[test]
    fn test_ens_names_in_config_parsing() {
        let config_yaml = r#"
groups:
  maintainers:
    - vitalik.eth
    - alice.eth
    - evm:0x1234567890123456789012345678901234567890

permissions:
  default: allow
  rules:
    - vitalik.eth push >main
    - alice.eth edit contracts/**
    - maintainers push >develop
"#;

        let config = parser::parse(config_yaml).expect("Should parse ENS config");
        
        // Check that group contains ENS identities
        let maintainers = &config.groups["maintainers"];
        assert_eq!(maintainers.members.len(), 3);
        
        // Check ENS identities are parsed correctly
        let vitalik = &maintainers.members[0];
        assert_eq!(vitalik.kind, IdentityKind::Ens);
        assert_eq!(vitalik.address, "vitalik.eth");
        
        let alice = &maintainers.members[1];
        assert_eq!(alice.kind, IdentityKind::Ens);
        assert_eq!(alice.address, "alice.eth");

        // Check EVM identity is preserved
        let evm_member = &maintainers.members[2];
        assert_eq!(evm_member.kind, IdentityKind::Evm);
        
        // Check rules reference ENS identities
        assert_eq!(config.permissions.rules.len(), 3);
        assert_eq!(config.permissions.default, DefaultPolicy::Allow);
    }

    #[test]
    fn test_ens_identity_parsing_variations() {
        // Explicit ENS prefix
        let ens_explicit = Identity::parse("ens:vitalik.eth").unwrap();
        assert_eq!(ens_explicit.kind, IdentityKind::Ens);
        assert_eq!(ens_explicit.address, "vitalik.eth");

        // Implicit ENS detection
        let ens_implicit = Identity::parse("alice.eth").unwrap();
        assert_eq!(ens_implicit.kind, IdentityKind::Ens);
        assert_eq!(ens_implicit.address, "alice.eth");

        // Various TLDs
        let box_domain = Identity::parse("test.box").unwrap();
        assert_eq!(box_domain.kind, IdentityKind::Ens);
        
        let app_domain = Identity::parse("example.app").unwrap();
        assert_eq!(app_domain.kind, IdentityKind::Ens);

        // Subdomains
        let subdomain = Identity::parse("sub.example.eth").unwrap();
        assert_eq!(subdomain.kind, IdentityKind::Ens);
        assert_eq!(subdomain.address, "sub.example.eth");
    }

    #[test]
    fn test_ens_validation_edge_cases() {
        // Valid names should pass
        assert!(Identity::parse("vitalik.eth").is_ok());
        assert!(Identity::parse("my-project.eth").is_ok());
        assert!(Identity::parse("a.eth").is_ok());
        
        // Invalid names should fail
        assert!(Identity::parse("localhost").is_err());
        assert!(Identity::parse("invalid").is_err());
        assert!(Identity::parse("test.invalid").is_err());
        assert!(Identity::parse("").is_err());
        assert!(Identity::parse(".eth").is_err());
        assert!(Identity::parse("test.").is_err());
        
        // Names with invalid characters
        assert!(Identity::parse("test@example.eth").is_err());
        assert!(Identity::parse("test space.eth").is_err());
        
        // Names that are too long
        let long_name = "a".repeat(64) + ".eth";
        assert!(Identity::parse(&long_name).is_err());
    }

    #[test]
    fn test_ens_permission_engine_integration() {
        let config_yaml = r#"
groups:
  team:
    - alice.eth
    - bob.eth

permissions:
  default: deny
  rules:
    - alice.eth push >main
    - team push >develop
    - bob.eth edit src/**
"#;

        let config = parser::parse(config_yaml).expect("Should parse ENS config");
        let alice = Identity::parse("alice.eth").unwrap();
        let bob = Identity::parse("bob.eth").unwrap();
        let charlie = Identity::parse("charlie.eth").unwrap();

        // Test without resolver (static check only)
        let result = engine::check(&config, &alice, Verb::Push, Some("main"), None);
        assert_eq!(result.is_allowed(), true);

        let result = engine::check(&config, &alice, Verb::Push, Some("develop"), None);
        assert_eq!(result.is_allowed(), true); // Should match via team group

        let result = engine::check(&config, &bob, Verb::Edit, None, Some("src/app.rs"));
        assert_eq!(result.is_allowed(), true);

        // Unauthorized action
        let result = engine::check(&config, &charlie, Verb::Push, Some("main"), None);
        assert_eq!(result.is_allowed(), false);
    }

    #[test]
    fn test_ens_canonical_representation() {
        let ens_id = Identity::parse("vitalik.eth").unwrap();
        assert_eq!(ens_id.canonical(), "ens:vitalik.eth");
        
        let ens_explicit = Identity::parse("ens:alice.eth").unwrap();
        assert_eq!(ens_explicit.canonical(), "ens:alice.eth");
        
        let evm_id = Identity::parse("evm:0x1234567890123456789012345678901234567890").unwrap();
        assert_eq!(evm_id.canonical(), "evm:0x1234567890123456789012345678901234567890");
    }

    #[test]
    fn test_resolver_creation_for_ens() {
        let resolver = RemoteResolver::new("https://repo.box/api");
        
        // Test resolver accepts ENS cache TTL configuration
        let resolver_with_ttl = RemoteResolver::with_ens_cache_ttl("https://repo.box/api", 120);
        assert_eq!(resolver_with_ttl.ens_cache_ttl, 120);
        
        // Default TTL should be 60 seconds
        assert_eq!(resolver.ens_cache_ttl, 60);
    }

    #[test]
    fn test_mixed_evm_ens_groups() {
        let config_yaml = r#"
groups:
  mixed_team:
    - vitalik.eth
    - evm:0x1234567890123456789012345678901234567890
    - alice.eth

permissions:
  default: deny
  rules:
    - mixed_team push >*
"#;

        let config = parser::parse(config_yaml).expect("Should parse mixed group config");
        let group = &config.groups["mixed_team"];
        
        assert_eq!(group.members.len(), 3);
        assert_eq!(group.members[0].kind, IdentityKind::Ens);
        assert_eq!(group.members[1].kind, IdentityKind::Evm);
        assert_eq!(group.members[2].kind, IdentityKind::Ens);
        
        // All should be able to push to any branch via group membership
        let vitalik = Identity::parse("vitalik.eth").unwrap();
        let evm_member = Identity::parse("evm:0x1234567890123456789012345678901234567890").unwrap();
        let alice = Identity::parse("alice.eth").unwrap();

        let result_vitalik = engine::check(&config, &vitalik, Verb::Push, Some("main"), None);
        let result_evm = engine::check(&config, &evm_member, Verb::Push, Some("main"), None);
        let result_alice = engine::check(&config, &alice, Verb::Push, Some("main"), None);

        assert_eq!(result_vitalik.is_allowed(), true);
        assert_eq!(result_evm.is_allowed(), true);
        assert_eq!(result_alice.is_allowed(), true);
    }
}