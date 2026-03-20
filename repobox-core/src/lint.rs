//! Lint warnings for .repobox/config.yml configs.
//!
//! These are not errors — the config is valid — but likely footguns.

use crate::config::*;

/// A lint warning with severity and actionable message.
#[derive(Debug)]
pub struct LintWarning {
    pub severity: Severity,
    pub message: String,
    pub hint: String,
}

#[derive(Debug, PartialEq)]
pub enum Severity {
    Warning,
    Info,
}

impl std::fmt::Display for LintWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = match self.severity {
            Severity::Warning => "⚠️",
            Severity::Info => "💡",
        };
        write!(f, "{icon}  {}\n     {}", self.message, self.hint)
    }
}

/// Run all lint checks on a parsed config.
pub fn lint(config: &Config) -> Vec<LintWarning> {
    let mut warnings = Vec::new();

    check_shadowed_deny_after_allow(config, &mut warnings);
    check_own_before_deny(config, &mut warnings);
    check_duplicate_rules(config, &mut warnings);
    check_wildcard_edit_includes_config(config, &mut warnings);
    check_agents_edit_star_no_branch_scope(config, &mut warnings);
    check_deny_after_wildcard_subject(config, &mut warnings);
    check_branch_rules_no_file_rules_with_default_deny(config, &mut warnings);
    check_unused_groups(config, &mut warnings);

    warnings
}

/// Subject display helper
fn subject_name(s: &Subject) -> String {
    match s {
        Subject::All => "*".to_string(),
        Subject::Group(name) => name.clone(),
        Subject::Identity(id) => format!("{}", id),
    }
}

/// Check if two subjects are the same
fn same_subject(a: &Subject, b: &Subject) -> bool {
    match (a, b) {
        (Subject::All, Subject::All) => true,
        (Subject::Group(a), Subject::Group(b)) => a == b,
        (Subject::Identity(a), Subject::Identity(b)) => a == b,
        _ => false,
    }
}

/// Check if subject A is broader than or equal to B
fn subject_covers(broader: &Subject, narrower: &Subject) -> bool {
    match broader {
        Subject::All => true, // * covers everything
        _ => same_subject(broader, narrower),
    }
}

/// Check if target A covers target B (is broader or equal)
fn target_covers(broader: &Target, narrower: &Target) -> bool {
    // * covers everything
    let b_branch = broader.branch.as_deref().unwrap_or("*");
    let n_branch = narrower.branch.as_deref().unwrap_or("*");
    let b_path = broader.path.as_deref().unwrap_or("*");
    let n_path = narrower.path.as_deref().unwrap_or("*");

    let branch_covers = b_branch == "*" || b_branch == n_branch
        || (b_branch.ends_with("/**") && n_branch.starts_with(&b_branch[..b_branch.len()-3]));
    let path_covers = b_path == "*" || b_path == n_path
        || (b_path.ends_with("/**") && n_path.starts_with(&b_path[..b_path.len()-3]));

    branch_covers && path_covers
}

// ── Individual checks ──────────────────────────────────────────────────

/// 1. Allow rule shadows a later deny for the same subject+verb
fn check_shadowed_deny_after_allow(config: &Config, warnings: &mut Vec<LintWarning>) {
    let rules = &config.permissions.rules;
    for (i, allow_rule) in rules.iter().enumerate() {
        if allow_rule.deny {
            continue;
        }
        for later in &rules[i + 1..] {
            if !later.deny {
                continue;
            }
            if later.verb != allow_rule.verb && !is_own_expanded(allow_rule) {
                continue;
            }
            if subject_covers(&allow_rule.subject, &later.subject)
                && target_covers(&allow_rule.target, &later.target)
            {
                warnings.push(LintWarning {
                    severity: Severity::Warning,
                    message: format!(
                        "rule '{} not {} {}' is shadowed by earlier allow rule (line {})",
                        subject_name(&later.subject), later.verb,
                        target_display(&later.target), allow_rule.line
                    ),
                    hint: "Move the deny rule ABOVE the allow rule for it to take effect.".into(),
                });
            }
        }
    }
}

/// 2. "own" expanded rules before a specific deny for same subject
fn check_own_before_deny(config: &Config, warnings: &mut Vec<LintWarning>) {
    // This is caught by check_shadowed_deny_after_allow since own expands.
    // But we add a specific message if we detect a cluster of 8 same-subject
    // same-target allow rules (likely from own expansion) before a deny.
    let rules = &config.permissions.rules;
    let mut i = 0;
    while i + 8 <= rules.len() {
        // Check if rules[i..i+8] look like an own expansion
        if rules[i..i+8].iter().all(|r| {
            !r.deny && same_subject(&r.subject, &rules[i].subject)
                && target_eq(&r.target, &rules[i].target)
        }) {
            let own_subject = &rules[i].subject;
            let own_target = &rules[i].target;
            // Check for denies after this block
            for later in &rules[i + 8..] {
                if later.deny && same_subject(&later.subject, own_subject)
                    && target_covers(own_target, &later.target)
                {
                    warnings.push(LintWarning {
                        severity: Severity::Warning,
                        message: format!(
                            "'not {} {}' after 'own {}' — the deny will never trigger",
                            later.verb, target_display(&later.target),
                            target_display(own_target)
                        ),
                        hint: "Place exceptions BEFORE 'own'. Example:\n       - founders not force-push >main\n       - founders own >main".into(),
                    });
                }
            }
        }
        i += 1;
    }
}

/// 3. Duplicate rules (same subject, verb, target, deny)
fn check_duplicate_rules(config: &Config, warnings: &mut Vec<LintWarning>) {
    let rules = &config.permissions.rules;
    for (i, a) in rules.iter().enumerate() {
        for b in &rules[i + 1..] {
            if same_subject(&a.subject, &b.subject)
                && a.verb == b.verb
                && a.deny == b.deny
                && target_eq(&a.target, &b.target)
            {
                warnings.push(LintWarning {
                    severity: Severity::Warning,
                    message: format!(
                        "duplicate rule: '{} {} {}'",
                        subject_name(&a.subject), a.verb, target_display(&a.target)
                    ),
                    hint: "Remove the duplicate — it has no effect.".into(),
                });
                break; // Only report once per duplicate
            }
        }
    }
}

/// 4. Non-founders with `edit *` (no branch scope) — includes .repobox/config.yml
fn check_wildcard_edit_includes_config(config: &Config, warnings: &mut Vec<LintWarning>) {
    for rule in &config.permissions.rules {
        if rule.deny {
            continue;
        }
        if rule.verb != Verb::Edit {
            continue;
        }
        // Check if target is * (all files) without branch scoping
        let path = rule.target.path.as_deref().unwrap_or("*");
        if path != "*" {
            continue;
        }
        if rule.target.branch.is_some() {
            continue; // Branch-scoped edit * is fine
        }
        // Check it's not a founders-only group
        if let Subject::Group(name) = &rule.subject {
            if name == "founders" || name == "admins" || name == "owners" {
                continue; // Expected for privileged groups
            }
        }
        warnings.push(LintWarning {
            severity: Severity::Warning,
            message: format!(
                "'{} edit *' grants access to ALL files including .repobox/config.yml",
                subject_name(&rule.subject)
            ),
            hint: "Scope to branches: 'edit * >feature/**' or add 'not edit ./.repobox/config.yml'".into(),
        });
    }
}

/// 5. agents edit * without branch scoping
fn check_agents_edit_star_no_branch_scope(config: &Config, warnings: &mut Vec<LintWarning>) {
    // Already covered by check_wildcard_edit_includes_config
    // This is intentionally a no-op to avoid duplicate warnings
}

/// 6. Deny after wildcard subject * — the deny is unreachable
fn check_deny_after_wildcard_subject(config: &Config, warnings: &mut Vec<LintWarning>) {
    let rules = &config.permissions.rules;
    for (i, wildcard_rule) in rules.iter().enumerate() {
        if !matches!(&wildcard_rule.subject, Subject::All) {
            continue;
        }
        if wildcard_rule.deny {
            continue; // Wildcard deny is fine
        }
        // Check for any deny below with a narrower subject + same verb + covered target
        for later in &rules[i + 1..] {
            if !later.deny {
                continue;
            }
            if later.verb == wildcard_rule.verb
                && target_covers(&wildcard_rule.target, &later.target)
            {
                warnings.push(LintWarning {
                    severity: Severity::Warning,
                    message: format!(
                        "'not {} {}' after '* {} {}' — wildcard already allows everyone",
                        later.verb, target_display(&later.target),
                        wildcard_rule.verb, target_display(&wildcard_rule.target)
                    ),
                    hint: "Place the deny ABOVE the wildcard allow, or use the wildcard for deny instead.".into(),
                });
            }
        }
    }
}

/// 7. Branch rules exist for a group but no file rules (with default: deny)
fn check_branch_rules_no_file_rules_with_default_deny(config: &Config, warnings: &mut Vec<LintWarning>) {
    if config.permissions.default != DefaultPolicy::Deny {
        return;
    }

    // Collect groups that have branch verbs
    let mut has_branch_rule: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    let mut has_file_rule: std::collections::HashMap<String, bool> = std::collections::HashMap::new();

    for rule in &config.permissions.rules {
        if rule.deny {
            continue;
        }
        let name = subject_name(&rule.subject);
        if rule.verb.is_branch_verb() {
            has_branch_rule.insert(name.clone(), true);
        }
        if rule.verb.is_file_verb() {
            has_file_rule.insert(name.clone(), true);
        }
    }

    for (name, _) in &has_branch_rule {
        if !has_file_rule.contains_key(name) && name != "*" {
            warnings.push(LintWarning {
                severity: Severity::Info,
                message: format!(
                    "'{name}' has branch rules (push/merge/create) but no file rules (edit/write/append)"
                ),
                hint: format!("With default: deny, {name} can push but can't modify any files. Add file rules or switch to default: allow."),
            });
        }
    }
}

/// 8. Groups defined but never used in rules
fn check_unused_groups(config: &Config, warnings: &mut Vec<LintWarning>) {
    let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();

    for rule in &config.permissions.rules {
        if let Subject::Group(name) = &rule.subject {
            used.insert(name.clone());
        }
    }

    // Also check includes
    for group in config.groups.values() {
        for inc in &group.includes {
            used.insert(inc.clone());
        }
    }

    for name in config.groups.keys() {
        if !used.contains(name) {
            warnings.push(LintWarning {
                severity: Severity::Info,
                message: format!("group '{name}' is defined but never used in any rule"),
                hint: "Remove the group or add rules that reference it.".into(),
            });
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

fn target_display(t: &Target) -> String {
    match (&t.branch, &t.path) {
        (Some(b), Some(p)) => format!("{p} >{b}"),
        (Some(b), None) => format!(">{b}"),
        (None, Some(p)) => p.clone(),
        (None, None) => "*".to_string(),
    }
}

fn target_eq(a: &Target, b: &Target) -> bool {
    a.branch == b.branch && a.path == b.path
}

fn is_own_expanded(_rule: &Rule) -> bool {
    // Can't reliably detect this after expansion, so return false
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn lint_yaml(yaml: &str) -> Vec<LintWarning> {
        let config = parser::parse(yaml).unwrap();
        lint(&config)
    }

    fn has_warning_containing(warnings: &[LintWarning], text: &str) -> bool {
        warnings.iter().any(|w| w.message.contains(text) || w.hint.contains(text))
    }

    // ── 1. Shadowed deny after allow ──

    #[test]
    fn test_shadowed_deny_same_subject() {
        let w = lint_yaml(r#"
groups:
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: deny
  rules:
    - agents push >*
    - agents not push >main
"#);
        assert!(has_warning_containing(&w, "shadowed"), "expected shadowed warning, got: {w:?}");
    }

    #[test]
    fn test_deny_before_allow_no_warning() {
        let w = lint_yaml(r#"
groups:
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: deny
  rules:
    - agents not push >main
    - agents push >*
"#);
        assert!(!has_warning_containing(&w, "shadowed"), "should NOT warn when deny is first: {w:?}");
    }

    // ── 2. Own before deny ──

    #[test]
    fn test_own_before_deny_warns() {
        let w = lint_yaml(r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: deny
  rules:
    - founders own >main
    - founders not force-push >main
"#);
        assert!(has_warning_containing(&w, "own") || has_warning_containing(&w, "shadowed"),
            "expected own/shadowed warning, got: {w:?}");
    }

    #[test]
    fn test_deny_before_own_no_warning() {
        let w = lint_yaml(r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: deny
  rules:
    - founders not force-push >main
    - founders own >main
"#);
        let own_warnings: Vec<_> = w.iter().filter(|w| w.message.contains("own") || w.message.contains("force-push")).collect();
        assert!(own_warnings.is_empty(), "should NOT warn when deny is before own: {own_warnings:?}");
    }

    // ── 3. Duplicate rules ──

    #[test]
    fn test_duplicate_rules_warn() {
        let w = lint_yaml(r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: deny
  rules:
    - founders push >main
    - founders push >main
"#);
        assert!(has_warning_containing(&w, "duplicate"), "expected duplicate warning, got: {w:?}");
    }

    // ── 4. Wildcard edit includes config ──

    #[test]
    fn test_agents_edit_star_warns() {
        let w = lint_yaml(r#"
groups:
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: deny
  rules:
    - agents edit *
"#);
        assert!(has_warning_containing(&w, ".repobox/config.yml"), "expected config warning, got: {w:?}");
    }

    #[test]
    fn test_agents_edit_star_scoped_no_warning() {
        let w = lint_yaml(r#"
groups:
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: deny
  rules:
    - agents edit * >feature/**
"#);
        assert!(!has_warning_containing(&w, ".repobox/config.yml"),
            "should NOT warn about scoped edit: {w:?}");
    }

    #[test]
    fn test_founders_edit_star_no_warning() {
        let w = lint_yaml(r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: deny
  rules:
    - founders edit *
"#);
        assert!(!has_warning_containing(&w, ".repobox/config.yml"),
            "should NOT warn about founders edit *: {w:?}");
    }

    // ── 5. Wildcard allow then deny ──

    #[test]
    fn test_wildcard_allow_then_deny_warns() {
        let w = lint_yaml(r#"
groups:
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: deny
  rules:
    - "* push >*"
    - agents not push >main
"#);
        assert!(has_warning_containing(&w, "wildcard") || has_warning_containing(&w, "shadowed"),
            "expected wildcard/shadowed warning, got: {w:?}");
    }

    // ── 6. Branch rules but no file rules with default: deny ──

    #[test]
    fn test_branch_no_file_rules_default_deny() {
        let w = lint_yaml(r#"
groups:
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: deny
  rules:
    - agents push >feature/**
    - agents create >feature/**
"#);
        assert!(has_warning_containing(&w, "no file rules"),
            "expected missing file rules warning, got: {w:?}");
    }

    #[test]
    fn test_branch_no_file_rules_default_allow_no_warning() {
        let w = lint_yaml(r#"
groups:
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: allow
  rules:
    - agents push >feature/**
    - agents create >feature/**
"#);
        assert!(!has_warning_containing(&w, "no file rules"),
            "should NOT warn with default: allow: {w:?}");
    }

    // ── 7. Unused groups ──

    #[test]
    fn test_unused_group_warns() {
        let w = lint_yaml(r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  ghosts:
    - evm:0xCCC0000000000000000000000000000000000003
permissions:
  default: deny
  rules:
    - founders push >*
"#);
        assert!(has_warning_containing(&w, "ghosts"), "expected unused group warning, got: {w:?}");
    }

    #[test]
    fn test_included_group_not_unused() {
        let w = lint_yaml(r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  all:
    - founders
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: deny
  rules:
    - all push >*
"#);
        assert!(!has_warning_containing(&w, "founders"),
            "included group should not be flagged as unused: {w:?}");
    }

    // ── 8. Clean config = no warnings ──

    #[test]
    fn test_clean_config_no_warnings() {
        let w = lint_yaml(r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: deny
  rules:
    - founders not force-push >main
    - founders own >main
    - agents not push >main
    - agents push >feature/**
    - agents create >feature/**
    - agents edit * >feature/**
    - agents append ./.repobox/config.yml
"#);
        // Filter out only warnings (not info)
        let warns: Vec<_> = w.iter().filter(|w| w.severity == Severity::Warning).collect();
        assert!(warns.is_empty(), "clean config should have no warnings, got: {warns:?}");
    }
}
