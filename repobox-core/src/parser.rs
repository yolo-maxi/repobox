use std::collections::HashMap;

use serde::Deserialize;

use crate::config::*;

/// Raw YAML structure — intermediate form before validation.
#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default)]
    groups: HashMap<String, RawGroup>,
    #[serde(default)]
    permissions: Option<RawPermissions>,
}

/// A group can be either:
///   - A plain list: `founders: [evm:0xAAA...]` (simple form)
///   - A mapping: `founders: { members: [...], includes: [...] }` (full form)
#[derive(Debug)]
struct RawGroup {
    members: Vec<String>,
    includes: Vec<String>,
}

impl<'de> Deserialize<'de> for RawGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        #[derive(Deserialize)]
        struct FullGroup {
            #[serde(default)]
            members: Vec<String>,
            #[serde(default)]
            includes: Vec<String>,
        }

        // Try as sequence first (simple form), then as mapping (full form)
        let value = serde_yaml::Value::deserialize(deserializer)?;
        match &value {
            serde_yaml::Value::Sequence(seq) => {
                let mut members = vec![];
                let mut includes = vec![];
                for v in seq {
                    let s = v
                        .as_str()
                        .ok_or_else(|| de::Error::custom("group entries must be strings"))?;
                    if s.starts_with("evm:") {
                        members.push(s.to_string());
                    } else {
                        // Bare word or group: prefix = group include
                        let name = s.strip_prefix("group:").unwrap_or(s);
                        includes.push(name.to_string());
                    }
                }
                Ok(RawGroup { members, includes })
            }
            serde_yaml::Value::Mapping(_) => {
                let full: FullGroup =
                    serde_yaml::from_value(value).map_err(de::Error::custom)?;
                Ok(RawGroup {
                    members: full.members,
                    includes: full.includes,
                })
            }
            _ => Err(de::Error::custom(
                "group must be a list of members or a mapping with 'members' key",
            )),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawPermissions {
    #[serde(default = "default_allow")]
    default: String,
    #[serde(default = "default_rules")]
    rules: serde_yaml::Value,
}

fn default_rules() -> serde_yaml::Value {
    serde_yaml::Value::Sequence(vec![])
}

fn default_allow() -> String {
    "allow".to_string()
}

/// Parse a .repobox.yml YAML string into a validated Config.
pub fn parse(yaml: &str) -> Result<Config, ConfigError> {
    let raw: RawConfig = serde_yaml::from_str(yaml)?;

    // Parse groups
    let mut groups = HashMap::new();
    for (name, raw_group) in &raw.groups {
        let members = raw_group
            .members
            .iter()
            .map(|s| Identity::parse(s))
            .collect::<Result<Vec<_>, _>>()?;

        groups.insert(
            name.clone(),
            Group {
                name: name.clone(),
                members,
                includes: raw_group.includes.clone(),
            },
        );
    }

    // Validate includes — check for unknown groups and circular deps
    validate_includes(&groups)?;

    // Parse permissions
    let permissions = match raw.permissions {
        Some(raw_perms) => {
            let default = match raw_perms.default.as_str() {
                "allow" => DefaultPolicy::Allow,
                "deny" => DefaultPolicy::Deny,
                other => {
                    return Err(ConfigError::InvalidRule(format!(
                        "invalid default policy: '{other}', expected 'allow' or 'deny'"
                    )));
                }
            };

            let mut rules = Vec::new();
            match &raw_perms.rules {
                // Format A: rules is a list (current format)
                serde_yaml::Value::Sequence(seq) => {
                    for (idx, value) in seq.iter().enumerate() {
                        let line = idx + 1;
                        let mut parsed = parse_rule_value(value, line, &groups)?;
                        rules.append(&mut parsed);
                    }
                }
                // Format B/C: rules is a mapping (subject-grouped)
                serde_yaml::Value::Mapping(map) => {
                    let mut line = 1;
                    for (key, val) in map {
                        let subject_str = key
                            .as_str()
                            .ok_or_else(|| ConfigError::InvalidRule("rule key must be a string".into()))?;
                        let subject = parse_subject(subject_str)?;

                        // Validate group reference
                        if let Subject::Group(name) = &subject {
                            if !groups.contains_key(name) {
                                return Err(ConfigError::UnknownGroup(name.clone()));
                            }
                        }

                        match val {
                            // Format B: subject → list of "verb target" strings
                            serde_yaml::Value::Sequence(entries) => {
                                for entry in entries {
                                    let s = entry.as_str().ok_or_else(|| {
                                        ConfigError::InvalidRule("rule entry must be a string".into())
                                    })?;
                                    // Parse "verb target" or "not verb target"
                                    let parts: Vec<&str> = s.split_whitespace().collect();
                                    if parts.is_empty() {
                                        return Err(ConfigError::InvalidRule("empty rule".into()));
                                    }
                                    let (deny, verb, target_start) = if parts[0] == "not" {
                                        if parts.len() < 3 {
                                            return Err(ConfigError::InvalidRule(format!(
                                                "deny rule needs at least 'not verb target', got: '{s}'"
                                            )));
                                        }
                                        let (_, verb) = parse_verb_str(parts[1])?;
                                        (true, verb, 2)
                                    } else {
                                        if parts.len() < 2 {
                                            return Err(ConfigError::InvalidRule(format!(
                                                "rule needs at least 'verb target', got: '{s}'"
                                            )));
                                        }
                                        let (deny, verb) = parse_verb_str(parts[0])?;
                                        (deny, verb, 1)
                                    };
                                    let target_str = parts[target_start..].join(" ");
                                    let target = Target::parse(&target_str)?;
                                    rules.push(Rule {
                                        subject: subject.clone(),
                                        verb,
                                        deny,
                                        target,
                                        line,
                                    });
                                    line += 1;
                                }
                            }
                            // Format C: subject → mapping of verb → targets
                            serde_yaml::Value::Mapping(verb_map) => {
                                for (vkey, vval) in verb_map {
                                    let verb_str = vkey.as_str().ok_or_else(|| {
                                        ConfigError::InvalidRule("verb key must be a string".into())
                                    })?;
                                    let (deny, verb) = parse_verb_str(verb_str)?;

                                    let targets = match vval {
                                        serde_yaml::Value::Sequence(tlist) => tlist.clone(),
                                        serde_yaml::Value::String(s) => {
                                            vec![serde_yaml::Value::String(s.clone())]
                                        }
                                        _ => {
                                            return Err(ConfigError::InvalidRule(
                                                "verb targets must be a list or string".into(),
                                            ));
                                        }
                                    };

                                    for tval in &targets {
                                        let t = tval.as_str().ok_or_else(|| {
                                            ConfigError::InvalidRule("target must be a string".into())
                                        })?;
                                        let target = Target::parse(t)?;
                                        rules.push(Rule {
                                            subject: subject.clone(),
                                            verb,
                                            deny,
                                            target,
                                            line,
                                        });
                                        line += 1;
                                    }
                                }
                            }
                            _ => {
                                return Err(ConfigError::InvalidRule(format!(
                                    "rules for '{subject_str}' must be a list or mapping"
                                )));
                            }
                        }
                    }
                }
                serde_yaml::Value::Null => {} // empty rules: OK
                _ => {
                    return Err(ConfigError::InvalidRule(
                        "rules must be a list or mapping".into(),
                    ));
                }
            }

            // Validate group references in rules
            for rule in &rules {
                if let Subject::Group(name) = &rule.subject {
                    if !groups.contains_key(name) {
                        return Err(ConfigError::UnknownGroup(name.clone()));
                    }
                }
            }

            Permissions { default, rules }
        }
        None => Permissions {
            default: DefaultPolicy::Allow,
            rules: Vec::new(),
        },
    };

    Ok(Config {
        groups,
        permissions,
    })
}

/// Parse a single rule entry — either a flat string or a nested object.
fn parse_rule_value(
    value: &serde_yaml::Value,
    line: usize,
    groups: &HashMap<String, Group>,
) -> Result<Vec<Rule>, ConfigError> {
    match value {
        // Flat rule: "founders edit *"
        serde_yaml::Value::String(s) => {
            let rule = parse_flat_rule(s, line)?;
            Ok(vec![rule])
        }
        // Nested rule: { "agents": { push: [">feature/**"], ... } }
        serde_yaml::Value::Mapping(map) => {
            let mut rules = Vec::new();
            for (key, val) in map {
                let subject_str = key
                    .as_str()
                    .ok_or_else(|| ConfigError::InvalidRule("rule key must be a string".into()))?;
                let subject = parse_subject(subject_str)?;

                // Validate group reference
                if let Subject::Group(name) = &subject {
                    if !groups.contains_key(name) {
                        return Err(ConfigError::UnknownGroup(name.clone()));
                    }
                }

                let verb_map = val.as_mapping().ok_or_else(|| {
                    ConfigError::InvalidRule(format!(
                        "expected verb mapping for subject '{subject_str}'"
                    ))
                })?;

                for (verb_key, targets_val) in verb_map {
                    let verb_str = verb_key.as_str().ok_or_else(|| {
                        ConfigError::InvalidRule("verb must be a string".into())
                    })?;

                    let (deny, verb) = parse_verb_str(verb_str)?;

                    let targets = targets_val.as_sequence().ok_or_else(|| {
                        ConfigError::InvalidRule(format!(
                            "expected list of targets for verb '{verb_str}'"
                        ))
                    })?;

                    for target_val in targets {
                        let target_str = target_val.as_str().ok_or_else(|| {
                            ConfigError::InvalidRule("target must be a string".into())
                        })?;
                        let target = Target::parse(target_str)?;
                        rules.push(Rule {
                            subject: subject.clone(),
                            verb,
                            deny,
                            target,
                            line,
                        });
                    }
                }
            }
            Ok(rules)
        }
        _ => Err(ConfigError::InvalidRule(
            "rule must be a string or mapping".into(),
        )),
    }
}

/// Parse a flat rule string like "founders edit *" or "agents not merge >main".
fn parse_flat_rule(s: &str, line: usize) -> Result<Rule, ConfigError> {
    let parts: Vec<&str> = s.split_whitespace().collect();

    if parts.len() < 3 {
        return Err(ConfigError::InvalidRule(format!(
            "rule needs at least 3 parts (subject verb target), got: '{s}'"
        )));
    }

    let subject = parse_subject(parts[0])?;

    let (deny, verb, target_start) = if parts[1] == "not" {
        if parts.len() < 4 {
            return Err(ConfigError::InvalidRule(format!(
                "deny rule needs 4 parts (subject not verb target), got: '{s}'"
            )));
        }
        let (_, verb) = parse_verb_str(parts[2])?;
        (true, verb, 3)
    } else {
        let (deny, verb) = parse_verb_str(parts[1])?;
        (deny, verb, 2)
    };

    // Remaining parts form the target (e.g. "contracts/** >main")
    let target_str = parts[target_start..].join(" ");
    let target = Target::parse(&target_str)?;

    Ok(Rule {
        subject,
        verb,
        deny,
        target,
        line,
    })
}

/// Parse a subject string: bare group name, "evm:0x...", or legacy "%groupname".
fn parse_subject(s: &str) -> Result<Subject, ConfigError> {
    if s == "*" {
        Ok(Subject::All)
    } else if s.starts_with("evm:") {
        Ok(Subject::Identity(Identity::parse(s)?))
    } else {
        // Bare word = group name. Strip legacy % prefix if present.
        let name = s.strip_prefix('%').unwrap_or(s);
        Ok(Subject::Group(name.to_string()))
    }
}

/// Parse a verb string, handling "not" prefix.
fn parse_verb_str(s: &str) -> Result<(bool, Verb), ConfigError> {
    if let Some(v) = s.strip_prefix("not ") {
        Ok((true, Verb::parse(v)?))
    } else if let Some(v) = s.strip_prefix("not_") {
        Ok((true, Verb::parse(v)?))
    } else {
        Ok((false, Verb::parse(s)?))
    }
}

/// Validate group includes: check for unknown references and circular dependencies.
fn validate_includes(groups: &HashMap<String, Group>) -> Result<(), ConfigError> {
    for (_name, group) in groups {
        for inc in &group.includes {
            if !groups.contains_key(inc) {
                return Err(ConfigError::UnknownGroup(inc.clone()));
            }
        }
    }

    // Check for cycles using DFS
    for name in groups.keys() {
        let mut visited = Vec::new();
        check_cycle(name, groups, &mut visited, 0)?;
    }

    Ok(())
}

fn check_cycle(
    name: &str,
    groups: &HashMap<String, Group>,
    path: &mut Vec<String>,
    depth: usize,
) -> Result<(), ConfigError> {
    const MAX_DEPTH: usize = 5;

    if depth > MAX_DEPTH {
        return Err(ConfigError::MaxIncludeDepth(MAX_DEPTH));
    }

    if path.contains(&name.to_string()) {
        path.push(name.to_string());
        let cycle = path.join(" → ");
        return Err(ConfigError::CircularDependency(cycle));
    }

    if let Some(group) = groups.get(name) {
        path.push(name.to_string());
        for inc in &group.includes {
            check_cycle(inc, groups, path, depth + 1)?;
        }
        path.pop();
    }

    Ok(())
}

/// Resolve all members of a group, following includes recursively.
pub fn resolve_group_members(
    name: &str,
    groups: &HashMap<String, Group>,
) -> Result<Vec<Identity>, ConfigError> {
    let mut result = Vec::new();
    let mut visited = Vec::new();
    resolve_members_inner(name, groups, &mut result, &mut visited, 0)?;
    Ok(result)
}

fn resolve_members_inner(
    name: &str,
    groups: &HashMap<String, Group>,
    result: &mut Vec<Identity>,
    visited: &mut Vec<String>,
    depth: usize,
) -> Result<(), ConfigError> {
    const MAX_DEPTH: usize = 5;

    if depth > MAX_DEPTH {
        return Err(ConfigError::MaxIncludeDepth(MAX_DEPTH));
    }

    if visited.contains(&name.to_string()) {
        return Ok(()); // Already resolved, skip (cycle already validated)
    }
    visited.push(name.to_string());

    let group = groups
        .get(name)
        .ok_or_else(|| ConfigError::UnknownGroup(name.to_string()))?;

    for member in &group.members {
        if !result.contains(member) {
            result.push(member.clone());
        }
    }

    for inc in &group.includes {
        resolve_members_inner(inc, groups, result, visited, depth + 1)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================================================================
    // Section 1: .repobox.yml Parser
    // ================================================================

    #[test]
    fn test_valid_yaml_one_group_one_rule() {
        let yaml = r#"
groups:
  founders:
    members:
      - evm:0xAAA0000000000000000000000000000000000001

permissions:
  default: allow
  rules:
    - "founders push >main"
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.groups.len(), 1);
        assert!(config.groups.contains_key("founders"));
        assert_eq!(config.groups["founders"].members.len(), 1);
        assert_eq!(config.permissions.rules.len(), 1);

        let rule = &config.permissions.rules[0];
        assert!(matches!(&rule.subject, Subject::Group(g) if g == "founders"));
        assert_eq!(rule.verb, Verb::Push);
        assert!(!rule.deny);
    }

    #[test]
    fn test_simple_group_list_form() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
    - evm:0xAAA0000000000000000000000000000000000002
  agents:
    - evm:0xBBB0000000000000000000000000000000000001

permissions:
  default: allow
  rules:
    - "founders push >*"
    - "agents push >feature/**"
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.groups.len(), 2);
        assert_eq!(config.groups["founders"].members.len(), 2);
        assert_eq!(config.groups["agents"].members.len(), 1);
        assert_eq!(config.permissions.rules.len(), 2);
    }

    #[test]
    fn test_mixed_group_forms() {
        // Simple list + full mapping in same config
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  staff:
    members:
      - evm:0xBBB0000000000000000000000000000000000002
    includes:
      - founders
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.groups["founders"].members.len(), 1);
        assert_eq!(config.groups["staff"].members.len(), 1);
        let members = resolve_group_members("staff", &config.groups).unwrap();
        assert_eq!(members.len(), 2);
    }

    #[test]
    fn test_inline_group_includes() {
        // bare name in a group list = include that group
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  agents:
    - evm:0xBBB0000000000000000000000000000000000001
    - founders
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.groups["agents"].members.len(), 1);
        assert_eq!(config.groups["agents"].includes.len(), 1);
        assert_eq!(config.groups["agents"].includes[0], "founders");
        let members = resolve_group_members("agents", &config.groups).unwrap();
        assert_eq!(members.len(), 2);
        assert!(members.contains(&Identity::parse("evm:0xAAA0000000000000000000000000000000000001").unwrap()));
        assert!(members.contains(&Identity::parse("evm:0xBBB0000000000000000000000000000000000001").unwrap()));
    }

    #[test]
    fn test_group_includes_resolve_membership() {
        let yaml = r#"
groups:
  admins:
    members:
      - evm:0xAAA0000000000000000000000000000000000001
  staff:
    members:
      - evm:0xBBB0000000000000000000000000000000000002
    includes:
      - admins
"#;
        let config = parse(yaml).unwrap();
        let members = resolve_group_members("staff", &config.groups).unwrap();
        assert_eq!(members.len(), 2);
        assert!(members.contains(&Identity::parse("evm:0xBBB0000000000000000000000000000000000002").unwrap()));
        assert!(members.contains(&Identity::parse("evm:0xAAA0000000000000000000000000000000000001").unwrap()));
    }

    #[test]
    fn test_evm_identity_stored_correctly() {
        let yaml = r#"
groups:
  founders:
    members:
      - evm:0xAAA0000000000000000000000000000000000001
"#;
        let config = parse(yaml).unwrap();
        let member = &config.groups["founders"].members[0];
        assert_eq!(member.kind, IdentityKind::Evm);
        assert_eq!(member.address, "0xAAA0000000000000000000000000000000000001");
    }

    #[test]
    fn test_rule_references_group() {
        let yaml = r#"
groups:
  founders:
    members:
      - evm:0xAAA0000000000000000000000000000000000001

permissions:
  rules:
    - "founders edit *"
"#;
        let config = parse(yaml).unwrap();
        assert!(matches!(&config.permissions.rules[0].subject, Subject::Group(g) if g == "founders"));
    }

    #[test]
    fn test_unknown_group_reference_errors() {
        let yaml = r#"
groups:
  founders:
    members:
      - evm:0xAAA0000000000000000000000000000000000001

permissions:
  rules:
    - "nonexistent edit *"
"#;
        let err = parse(yaml).unwrap_err();
        assert!(
            err.to_string().contains("unknown group: nonexistent"),
            "got: {err}"
        );
    }

    #[test]
    fn test_circular_dependency_detected() {
        let yaml = r#"
groups:
  a:
    members: []
    includes:
      - b
  b:
    members: []
    includes:
      - a
"#;
        let err = parse(yaml).unwrap_err();
        assert!(
            err.to_string().contains("circular dependency"),
            "got: {err}"
        );
    }

    #[test]
    fn test_max_include_depth_exceeded() {
        let yaml = r#"
groups:
  a:
    members: []
    includes: [b]
  b:
    members: []
    includes: [c]
  c:
    members: []
    includes: [d]
  d:
    members: []
    includes: [e]
  e:
    members: []
    includes: [f]
  f:
    members: []
    includes: [g]
  g:
    members: []
"#;
        let err = parse(yaml).unwrap_err();
        assert!(
            err.to_string().contains("max include depth exceeded"),
            "got: {err}"
        );
    }

    #[test]
    fn test_branch_target_parsed() {
        let target = Target::parse(">main").unwrap();
        assert_eq!(target.branch.as_deref(), Some("main"));
        assert!(target.path.is_none());
    }

    #[test]
    fn test_path_target_parsed() {
        let target = Target::parse("contracts/**").unwrap();
        assert!(target.branch.is_none());
        assert_eq!(target.path.as_deref(), Some("contracts/**"));
    }

    #[test]
    fn test_combined_target_parsed() {
        let target = Target::parse("contracts/** >main").unwrap();
        assert_eq!(target.branch.as_deref(), Some("main"));
        assert_eq!(target.path.as_deref(), Some("contracts/**"));
    }

    #[test]
    fn test_empty_permissions_produces_valid_config() {
        let yaml = r#"
groups:
  founders:
    members:
      - evm:0xAAA0000000000000000000000000000000000001

permissions:
  rules: []
"#;
        let config = parse(yaml).unwrap();
        assert!(config.permissions.rules.is_empty());
        assert_eq!(config.permissions.default, DefaultPolicy::Allow);
    }

    #[test]
    fn test_deny_rule_parsed() {
        let yaml = r#"
groups:
  agents:
    members:
      - evm:0xBBB0000000000000000000000000000000000002

permissions:
  rules:
    - "agents not edit .repobox.yml"
"#;
        let config = parse(yaml).unwrap();
        let rule = &config.permissions.rules[0];
        assert!(rule.deny);
        assert_eq!(rule.verb, Verb::Edit);
    }

    #[test]
    fn test_invalid_yaml_returns_parse_error() {
        let yaml = "this is not: valid: yaml: [";
        let err = parse(yaml);
        assert!(err.is_err());
    }

    #[test]
    fn test_default_deny_parsed() {
        let yaml = r#"
permissions:
  default: deny
  rules: []
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.permissions.default, DefaultPolicy::Deny);
    }

    #[test]
    fn test_default_allow_when_omitted() {
        let yaml = r#"
permissions:
  rules: []
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.permissions.default, DefaultPolicy::Allow);
    }

    #[test]
    fn test_flat_rule_parsed_correctly() {
        let yaml = r#"
groups:
  founders:
    members:
      - evm:0xAAA0000000000000000000000000000000000001

permissions:
  rules:
    - "founders edit *"
"#;
        let config = parse(yaml).unwrap();
        let rule = &config.permissions.rules[0];
        assert!(matches!(&rule.subject, Subject::Group(g) if g == "founders"));
        assert_eq!(rule.verb, Verb::Edit);
        assert_eq!(rule.target.path.as_deref(), Some("*"));
    }

    #[test]
    fn test_nested_rule_expands_correctly() {
        let yaml = r#"
groups:
  agents:
    members:
      - evm:0xBBB0000000000000000000000000000000000002

permissions:
  rules:
    - "agents":
        push:
          - ">feature/**"
          - ">fix/**"
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.permissions.rules.len(), 2);
        assert_eq!(config.permissions.rules[0].verb, Verb::Push);
        assert_eq!(
            config.permissions.rules[0].target.branch.as_deref(),
            Some("feature/**")
        );
        assert_eq!(
            config.permissions.rules[1].target.branch.as_deref(),
            Some("fix/**")
        );
    }

    #[test]
    fn test_mixed_flat_and_nested_rules() {
        let yaml = r#"
groups:
  founders:
    members:
      - evm:0xAAA0000000000000000000000000000000000001
  agents:
    members:
      - evm:0xBBB0000000000000000000000000000000000002

permissions:
  rules:
    - "founders edit *"
    - "agents":
        push:
          - ">feature/**"
        create:
          - ">feature/**"
"#;
        let config = parse(yaml).unwrap();
        // Flat rule first, then 2 expanded nested rules
        assert_eq!(config.permissions.rules.len(), 3);
        assert_eq!(config.permissions.rules[0].verb, Verb::Edit);
        assert_eq!(config.permissions.rules[1].verb, Verb::Push);
        assert_eq!(config.permissions.rules[2].verb, Verb::Create);
    }

    // ========== Format B: subject-grouped rules ==========

    #[test]
    fn test_subject_grouped_rules() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  agents:
    - evm:0xBBB0000000000000000000000000000000000002

permissions:
  default: allow
  rules:
    founders:
      - push >main
      - merge >main
      - delete >*
    agents:
      - push >feature/**
      - not edit ./.repobox.yml
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.permissions.rules.len(), 5);

        // founders rules
        assert!(matches!(&config.permissions.rules[0].subject, Subject::Group(g) if g == "founders"));
        assert_eq!(config.permissions.rules[0].verb, Verb::Push);
        assert!(!config.permissions.rules[0].deny);

        assert_eq!(config.permissions.rules[1].verb, Verb::Merge);
        assert_eq!(config.permissions.rules[2].verb, Verb::Delete);

        // agents rules
        assert!(matches!(&config.permissions.rules[3].subject, Subject::Group(g) if g == "agents"));
        assert_eq!(config.permissions.rules[3].verb, Verb::Push);

        assert_eq!(config.permissions.rules[4].verb, Verb::Edit);
        assert!(config.permissions.rules[4].deny);
    }

    // ========== Format C: fully nested verb-mapping rules ==========

    #[test]
    fn test_verb_mapping_rules() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001

permissions:
  default: allow
  rules:
    founders:
      push:
        - ">main"
        - ">develop"
      merge:
        - ">main"
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.permissions.rules.len(), 3);
        assert_eq!(config.permissions.rules[0].verb, Verb::Push);
        assert_eq!(config.permissions.rules[1].verb, Verb::Push);
        assert_eq!(config.permissions.rules[2].verb, Verb::Merge);
    }

    // ========== Mixed B + C within same config ==========

    #[test]
    fn test_mixed_subject_grouped_and_verb_mapping() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  agents:
    - evm:0xBBB0000000000000000000000000000000000002

permissions:
  default: allow
  rules:
    founders:
      - push >*
      - merge >*
    agents:
      push:
        - ">feature/**"
      create:
        - ">feature/**"
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.permissions.rules.len(), 4);
        // founders: 2 rules from list format
        assert!(matches!(&config.permissions.rules[0].subject, Subject::Group(g) if g == "founders"));
        // agents: 2 rules from verb-mapping format
        assert!(matches!(&config.permissions.rules[2].subject, Subject::Group(g) if g == "agents"));
        assert_eq!(config.permissions.rules[2].verb, Verb::Push);
        assert_eq!(config.permissions.rules[3].verb, Verb::Create);
    }

    // ========== Wildcard subject * ==========

    #[test]
    fn test_wildcard_subject_flat() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: deny
  rules:
    - founders push >*
    - "* not push >main"
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.permissions.rules.len(), 2);
        assert!(matches!(&config.permissions.rules[1].subject, Subject::All));
        assert!(config.permissions.rules[1].deny);
    }

    #[test]
    fn test_wildcard_subject_matches_everyone() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: deny
  rules:
    - "* push >dev"
"#;
        let config = parse(yaml).unwrap();
        let random = Identity::parse("evm:0xCCC0000000000000000000000000000000000003").unwrap();
        let founder = Identity::parse("evm:0xAAA0000000000000000000000000000000000001").unwrap();

        use crate::engine;

        // Both match — * means everyone
        let r = engine::check(&config, &random, Verb::Push, Some("dev"), None);
        assert!(r.is_allowed(), "random identity should be allowed by wildcard");
        let r = engine::check(&config, &founder, Verb::Push, Some("dev"), None);
        assert!(r.is_allowed(), "founder should be allowed by wildcard");
    }

    #[test]
    fn test_wildcard_deny_with_exceptions() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  agents:
    - evm:0xBBB0000000000000000000000000000000000002
permissions:
  default: deny
  rules:
    - founders push >main
    - "* not push >main"
    - "* push >dev"
"#;
        let config = parse(yaml).unwrap();
        let founder = Identity::parse("evm:0xAAA0000000000000000000000000000000000001").unwrap();
        let agent = Identity::parse("evm:0xBBB0000000000000000000000000000000000002").unwrap();

        use crate::engine;

        // Founder can push to main (matched by first rule before wildcard deny)
        let r = engine::check(&config, &founder, Verb::Push, Some("main"), None);
        assert!(r.is_allowed(), "founder should push to main");

        // Agent cannot push to main (wildcard deny catches them)
        let r = engine::check(&config, &agent, Verb::Push, Some("main"), None);
        assert!(!r.is_allowed(), "agent should NOT push to main");

        // Both can push to dev
        let r = engine::check(&config, &founder, Verb::Push, Some("dev"), None);
        assert!(r.is_allowed(), "founder should push to dev");
        let r = engine::check(&config, &agent, Verb::Push, Some("dev"), None);
        assert!(r.is_allowed(), "agent should push to dev");
    }
}
