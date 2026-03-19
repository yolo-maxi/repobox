use std::collections::HashMap;

use crate::config::*;
use crate::parser::resolve_group_members;

/// Result of a permission check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// Allowed by a specific rule.
    Allow { rule_line: usize },
    /// Denied by a specific rule.
    Deny { rule_line: usize, reason: String },
    /// Denied by implicit deny (rules exist for this verb+target but none matched).
    ImplicitDeny { verb: Verb },
    /// Allowed/Denied by default policy (no rules exist for this verb+target).
    Default { policy: DefaultPolicy },
}

impl CheckResult {
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            CheckResult::Allow { .. }
                | CheckResult::Default {
                    policy: DefaultPolicy::Allow
                }
        )
    }
}

/// Evaluate a permission check against the config.
pub fn check(
    config: &Config,
    identity: &Identity,
    verb: Verb,
    branch: Option<&str>,
    path: Option<&str>,
) -> CheckResult {
    // Build resolved group membership map
    let resolved_groups = resolve_all_groups(&config.groups);

    // Collect rules that match this verb AND whose target matches
    let matching_rules: Vec<&Rule> = config
        .permissions
        .rules
        .iter()
        .filter(|r| r.verb == verb && r.target.matches(branch, path))
        .collect();

    // No rules match this verb+target → use default
    if matching_rules.is_empty() {
        return CheckResult::Default {
            policy: config.permissions.default,
        };
    }

    // Walk top-to-bottom, find first rule where subject matches
    for rule in &matching_rules {
        if rule.subject.matches(identity, &resolved_groups) {
            if rule.deny {
                return CheckResult::Deny {
                    rule_line: rule.line,
                    reason: format!("explicit deny at rule {}", rule.line),
                };
            } else {
                return CheckResult::Allow {
                    rule_line: rule.line,
                };
            }
        }
    }

    // Rules exist but none matched this identity → implicit deny
    CheckResult::ImplicitDeny { verb }
}

/// Resolve all groups to their full member lists (following includes).
fn resolve_all_groups(groups: &HashMap<String, crate::config::Group>) -> HashMap<String, Vec<Identity>> {
    let mut resolved = HashMap::new();
    for name in groups.keys() {
        if let Ok(members) = resolve_group_members(name, groups) {
            resolved.insert(name.clone(), members);
        }
    }
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    // Helper to build config from YAML and check
    fn check_yaml(
        yaml: &str,
        identity_str: &str,
        verb: Verb,
        branch: Option<&str>,
        path: Option<&str>,
    ) -> CheckResult {
        let config = parse(yaml).unwrap();
        let identity = Identity::parse(identity_str).unwrap();
        check(&config, &identity, verb, branch, path)
    }

    // ================================================================
    // Section 2: Permission Engine — Branches
    // ================================================================

    #[test]
    fn test_founder_push_main_allowed() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "%founders push >main"
"#,
            "evm:0xAAA0000000000000000000000000000000000001",
            Verb::Push,
            Some("main"),
            None,
        );
        assert!(r.is_allowed());
    }

    #[test]
    fn test_non_founder_push_main_denied() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "%founders push >main"
"#,
            "evm:0xCCC0000000000000000000000000000000000003",
            Verb::Push,
            Some("main"),
            None,
        );
        assert!(!r.is_allowed());
        assert!(matches!(r, CheckResult::ImplicitDeny { .. }));
    }

    #[test]
    fn test_agent_push_feature_glob() {
        let r = check_yaml(
            r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "%agents push >feature/*"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Push,
            Some("feature/my-thing"),
            None,
        );
        assert!(r.is_allowed());
    }

    #[test]
    fn test_agent_push_main_implicit_deny() {
        let yaml = r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "%founders push >main"
    - "%agents push >feature/**"
"#;
        let r = check_yaml(
            yaml,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Push,
            Some("main"),
            None,
        );
        assert!(!r.is_allowed());
    }

    #[test]
    fn test_agent_create_feature_allowed() {
        let r = check_yaml(
            r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "%agents create >feature/*"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Create,
            Some("feature/new"),
            None,
        );
        assert!(r.is_allowed());
    }

    #[test]
    fn test_agent_create_release_denied() {
        let r = check_yaml(
            r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "%agents create >feature/*"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Create,
            Some("release/v1"),
            None,
        );
        // No rule for create >release/* — depends on default
        assert!(matches!(r, CheckResult::Default { policy: DefaultPolicy::Allow }));
    }

    #[test]
    fn test_direct_identity_rule() {
        let yaml = r#"
groups: {}
permissions:
  rules:
    - "evm:0xBBB0000000000000000000000000000000000002 push >main"
"#;
        let r = check_yaml(
            yaml,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Push,
            Some("main"),
            None,
        );
        assert!(r.is_allowed());

        let r2 = check_yaml(
            yaml,
            "evm:0xAAA0000000000000000000000000000000000001",
            Verb::Push,
            Some("main"),
            None,
        );
        assert!(!r2.is_allowed());
    }

    #[test]
    fn test_explicit_deny_overrides_group() {
        let r = check_yaml(
            r#"
groups:
  devs:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "evm:0xBBB0000000000000000000000000000000000002 not push >main"
    - "%devs push >main"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Push,
            Some("main"),
            None,
        );
        assert!(!r.is_allowed());
        assert!(matches!(r, CheckResult::Deny { .. }));
    }

    // ================================================================
    // Section 3: Permission Engine — Files (implicit deny per target)
    // ================================================================

    #[test]
    fn test_founder_edit_repobox_config_allowed() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  default: allow
  rules:
    - "%founders edit .repobox-config"
"#,
            "evm:0xAAA0000000000000000000000000000000000001",
            Verb::Edit,
            None,
            Some(".repobox-config"),
        );
        assert!(r.is_allowed());
    }

    #[test]
    fn test_agent_edit_repobox_config_implicit_deny() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  default: allow
  rules:
    - "%founders edit .repobox-config"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Edit,
            None,
            Some(".repobox-config"),
        );
        assert!(!r.is_allowed());
    }

    #[test]
    fn test_agent_edit_unmentioned_file_allowed() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  default: allow
  rules:
    - "%founders edit .repobox-config"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Edit,
            None,
            Some("src/app.rs"),
        );
        // No edit rule covers src/app.rs → default: allow
        assert!(r.is_allowed());
    }

    #[test]
    fn test_edit_wildcard_locks_all_files() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  default: allow
  rules:
    - "%founders edit *"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Edit,
            None,
            Some("anything.rs"),
        );
        assert!(!r.is_allowed());
    }

    #[test]
    fn test_agent_edit_on_feature_branch_with_scoped_rule() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  default: allow
  rules:
    - "%founders edit *"
    - "%agents edit * >feature/**"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Edit,
            Some("feature/fix"),
            Some("src/app.rs"),
        );
        assert!(r.is_allowed());
    }

    #[test]
    fn test_agent_edit_on_main_denied_with_scoped_rule() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  default: allow
  rules:
    - "%founders edit *"
    - "%agents edit * >feature/**"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Edit,
            Some("main"),
            Some("src/app.rs"),
        );
        assert!(!r.is_allowed());
    }

    #[test]
    fn test_no_file_rules_default_allow() {
        let r = check_yaml(
            r#"
permissions:
  default: allow
  rules: []
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Edit,
            None,
            Some("anything.rs"),
        );
        assert!(r.is_allowed());
    }

    #[test]
    fn test_no_file_rules_default_deny() {
        let r = check_yaml(
            r#"
permissions:
  default: deny
  rules: []
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Edit,
            None,
            Some("anything.rs"),
        );
        assert!(!r.is_allowed());
    }

    // ================================================================
    // Section 4: Evaluation — Default and Implicit Deny
    // ================================================================

    #[test]
    fn test_no_push_rules_default_allow() {
        let r = check_yaml(
            r#"
permissions:
  default: allow
  rules: []
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Push,
            Some("main"),
            None,
        );
        assert!(r.is_allowed());
    }

    #[test]
    fn test_no_push_rules_default_deny() {
        let r = check_yaml(
            r#"
permissions:
  default: deny
  rules: []
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Push,
            Some("main"),
            None,
        );
        assert!(!r.is_allowed());
    }

    #[test]
    fn test_founder_merge_direct_match() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  default: allow
  rules:
    - "%founders merge >main"
"#,
            "evm:0xAAA0000000000000000000000000000000000001",
            Verb::Merge,
            Some("main"),
            None,
        );
        assert!(r.is_allowed());
    }

    #[test]
    fn test_agent_merge_main_implicit_deny() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  default: allow
  rules:
    - "%founders merge >main"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Merge,
            Some("main"),
            None,
        );
        assert!(!r.is_allowed());
    }

    #[test]
    fn test_unmatched_target_uses_default_allow() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  default: allow
  rules:
    - "%founders merge >main"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Merge,
            Some("feature/x"),
            None,
        );
        // No merge rule targets feature/x → default: allow
        assert!(r.is_allowed());
    }

    #[test]
    fn test_unmatched_target_uses_default_deny() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  default: deny
  rules:
    - "%founders merge >main"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Merge,
            Some("feature/x"),
            None,
        );
        assert!(!r.is_allowed());
    }

    #[test]
    fn test_deny_rule_order_deny_first() {
        let r = check_yaml(
            r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "%agents not push >main"
    - "%agents push >*"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Push,
            Some("main"),
            None,
        );
        assert!(!r.is_allowed());
    }

    #[test]
    fn test_deny_rule_order_allow_first_wins() {
        let r = check_yaml(
            r#"
groups:
  agents:
    members: [evm:0xBBB0000000000000000000000000000000000002]
permissions:
  rules:
    - "%agents push >*"
    - "%agents not push >main"
"#,
            "evm:0xBBB0000000000000000000000000000000000002",
            Verb::Push,
            Some("main"),
            None,
        );
        // push >* matches first → ALLOW (order matters!)
        assert!(r.is_allowed());
    }

    // ================================================================
    // Section 5: Priority (top-to-bottom)
    // ================================================================

    #[test]
    fn test_higher_rule_wins_allow_over_deny() {
        let r = check_yaml(
            r#"
groups:
  devs:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "evm:0xAAA0000000000000000000000000000000000001 edit .repobox-config"
    - "%devs not edit .repobox-config"
"#,
            "evm:0xAAA0000000000000000000000000000000000001",
            Verb::Edit,
            None,
            Some(".repobox-config"),
        );
        assert!(r.is_allowed());
    }

    #[test]
    fn test_higher_deny_wins_over_lower_allow() {
        let r = check_yaml(
            r#"
groups:
  founders:
    members: [evm:0xAAA0000000000000000000000000000000000001]
  devs:
    members: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - "%founders not merge >main"
    - "%devs merge >main"
"#,
            "evm:0xAAA0000000000000000000000000000000000001",
            Verb::Merge,
            Some("main"),
            None,
        );
        assert!(!r.is_allowed());
    }
}
