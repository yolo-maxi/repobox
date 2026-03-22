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
    #[serde(default)]
    x402: Option<RawX402Config>,
    #[serde(default)]
    virtuals: Option<RawVirtualsConfig>,
}

/// A group can be:
///   - A plain list: `founders: [evm:0xAAA..., other-group]` (static)
///   - A mapping with `members`/`includes`: `founders: { members: [...] }` (static)
///   - A mapping with `resolver`: (dynamic — http or onchain)
#[derive(Debug)]
struct RawGroup {
    members: Vec<String>,
    includes: Vec<String>,
    resolver: Option<RawResolver>,
}

#[derive(Debug)]
enum RawResolver {
    Http { url: String, cache_ttl: u64 },
    Onchain { chain: u64, contract: String, function: String, cache_ttl: u64 },
}

impl<'de> Deserialize<'de> for RawGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;

        // Try as sequence first (simple form), then as mapping
        let value = serde_yaml::Value::deserialize(deserializer)?;
        match &value {
            serde_yaml::Value::Sequence(seq) => {
                let mut members = vec![];
                let mut includes = vec![];
                for v in seq {
                    let s = v
                        .as_str()
                        .ok_or_else(|| de::Error::custom("group entries must be strings"))?;
                    if s.starts_with("evm:") || s.starts_with("ens:") || is_ens_name(s) {
                        members.push(s.to_string());
                    } else {
                        let name = s.strip_prefix("group:").unwrap_or(s);
                        includes.push(name.to_string());
                    }
                }
                Ok(RawGroup { members, includes, resolver: None })
            }
            serde_yaml::Value::Mapping(map) => {
                // Check if this is a resolver group
                if let Some(resolver_val) = map.get(&serde_yaml::Value::String("resolver".into())) {
                    let resolver_type = resolver_val.as_str()
                        .ok_or_else(|| de::Error::custom("resolver must be a string"))?;

                    let get_str = |key: &str| -> Result<String, D::Error> {
                        map.get(&serde_yaml::Value::String(key.into()))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .ok_or_else(|| de::Error::custom(format!("resolver requires '{key}' field")))
                    };
                    let get_u64 = |key: &str, default: u64| -> u64 {
                        map.get(&serde_yaml::Value::String(key.into()))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(default)
                    };

                    let resolver = match resolver_type {
                        "http" => {
                            let url = get_str("url")?;
                            let cache_ttl = get_u64("cache_ttl", 300);
                            RawResolver::Http { url, cache_ttl }
                        }
                        "onchain" => {
                            let chain = map.get(&serde_yaml::Value::String("chain".into()))
                                .and_then(|v| v.as_u64())
                                .ok_or_else(|| de::Error::custom("onchain resolver requires 'chain' field"))?;
                            let contract = get_str("contract")?;
                            let function = get_str("function").unwrap_or_else(|_| "isMember".to_string());
                            let cache_ttl = get_u64("cache_ttl", 300);
                            RawResolver::Onchain { chain, contract, function, cache_ttl }
                        }
                        other => return Err(de::Error::custom(format!("unknown resolver type: '{other}' (expected 'http' or 'onchain')"))),
                    };

                    Ok(RawGroup { members: vec![], includes: vec![], resolver: Some(resolver) })
                } else {
                    // Static group with members/includes mapping
                    #[derive(Deserialize)]
                    struct FullGroup {
                        #[serde(default)]
                        members: Vec<String>,
                        #[serde(default)]
                        includes: Vec<String>,
                    }
                    let full: FullGroup =
                        serde_yaml::from_value(value).map_err(de::Error::custom)?;
                    Ok(RawGroup {
                        members: full.members,
                        includes: full.includes,
                        resolver: None,
                    })
                }
            }
            _ => Err(de::Error::custom(
                "group must be a list, a mapping with members, or a resolver config",
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

#[derive(Debug, Deserialize)]
struct RawX402Config {
    read_price: String,
    recipient: String,
    network: String,
}

#[derive(Debug, Deserialize)]
struct RawVirtualsConfig {
    #[serde(default = "default_true")]
    enabled: bool,
    bug_bounties: RawBugBountyConfig,
    #[serde(default)]
    agent_requirements: Option<RawAgentRequirements>,
    #[serde(default)]
    payments: Option<RawVirtualsPaymentConfig>,
}

#[derive(Debug, Deserialize)]
struct RawBugBountyConfig {
    critical: String,
    high: String,
    medium: String,
    low: String,
}

#[derive(Debug, Deserialize)]
struct RawAgentRequirements {
    #[serde(default = "default_min_reputation")]
    min_reputation: f64,
    #[serde(default = "default_true")]
    required_tests: bool,
    #[serde(default = "default_true")]
    human_review_required: bool,
}

#[derive(Debug, Deserialize)]
struct RawVirtualsPaymentConfig {
    network: String,
    token: String,
    treasury: String,
    gas_sponsor: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_min_reputation() -> f64 {
    0.8
}

/// Parse a .repobox/config.yml YAML string into a validated Config.
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

        let resolver = match &raw_group.resolver {
            Some(RawResolver::Http { url, cache_ttl }) => Some(GroupResolver::Http {
                url: url.clone(),
                cache_ttl: *cache_ttl,
            }),
            Some(RawResolver::Onchain { chain, contract, function, cache_ttl }) => Some(GroupResolver::Onchain {
                chain: *chain,
                contract: contract.clone(),
                function: function.clone(),
                cache_ttl: *cache_ttl,
            }),
            None => None,
        };

        groups.insert(
            name.clone(),
            Group {
                name: name.clone(),
                members,
                includes: raw_group.includes.clone(),
                resolver,
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
                                    let (deny, verb_str, target_start) = if parts[0] == "not" {
                                        if parts.len() < 3 {
                                            return Err(ConfigError::InvalidRule(format!(
                                                "deny rule needs at least 'not verb target', got: '{s}'"
                                            )));
                                        }
                                        (true, parts[1], 2)
                                    } else {
                                        if parts.len() < 2 {
                                            return Err(ConfigError::InvalidRule(format!(
                                                "rule needs at least 'verb target', got: '{s}'"
                                            )));
                                        }
                                        (false, parts[0], 1)
                                    };
                                    let target_str = parts[target_start..].join(" ");
                                    let target = Target::parse(&target_str)?;

                                    // "own" expands to all verbs
                                    // Read is always repo-level (>*) regardless of own's target
                                    if verb_str == "own" {
                                        rules.push(Rule {
                                            subject: subject.clone(),
                                            verb: Verb::Read,
                                            deny,
                                            target: own_read_target(),
                                            line,
                                        });
                                        for &verb in OWN_WRITE_VERBS {
                                            rules.push(Rule {
                                                subject: subject.clone(),
                                                verb,
                                                deny,
                                                target: target.clone(),
                                                line,
                                            });
                                        }
                                    } else {
                                        let (deny2, verb) = parse_verb_str(verb_str)?;
                                        rules.push(Rule {
                                            subject: subject.clone(),
                                            verb,
                                            deny: deny || deny2,
                                            target,
                                            line,
                                        });
                                    }
                                    line += 1;
                                }
                            }
                            // Format C: subject → mapping of verb → targets
                            serde_yaml::Value::Mapping(verb_map) => {
                                for (vkey, vval) in verb_map {
                                    let verb_str = vkey.as_str().ok_or_else(|| {
                                        ConfigError::InvalidRule("verb key must be a string".into())
                                    })?;
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

                                        // "own" expands to all verbs
                                        // Read is always repo-level (>*) regardless of own's target
                                        if verb_str == "own" || verb_str == "not own" {
                                            let deny = verb_str.starts_with("not");
                                            rules.push(Rule {
                                                subject: subject.clone(),
                                                verb: Verb::Read,
                                                deny,
                                                target: own_read_target(),
                                                line,
                                            });
                                            for &verb in OWN_WRITE_VERBS {
                                                rules.push(Rule {
                                                    subject: subject.clone(),
                                                    verb,
                                                    deny,
                                                    target: target.clone(),
                                                    line,
                                                });
                                            }
                                        } else {
                                            let (deny, verb) = parse_verb_str(verb_str)?;
                                            rules.push(Rule {
                                                subject: subject.clone(),
                                                verb,
                                                deny,
                                                target,
                                                line,
                                            });
                                        }
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

    // Parse x402 config
    let x402 = raw.x402.map(|raw_x402| X402Config {
        read_price: raw_x402.read_price,
        recipient: raw_x402.recipient,
        network: raw_x402.network,
    });

    // Parse virtuals config
    let virtuals = if let Some(raw_virtuals) = raw.virtuals {
        let agent_requirements = raw_virtuals.agent_requirements.unwrap_or(RawAgentRequirements {
            min_reputation: default_min_reputation(),
            required_tests: default_true(),
            human_review_required: default_true(),
        });

        // Validate bounty amounts are valid decimals
        validate_usdc_amount(&raw_virtuals.bug_bounties.critical, "critical")?;
        validate_usdc_amount(&raw_virtuals.bug_bounties.high, "high")?;
        validate_usdc_amount(&raw_virtuals.bug_bounties.medium, "medium")?;
        validate_usdc_amount(&raw_virtuals.bug_bounties.low, "low")?;

        // Validate min_reputation is between 0.0 and 1.0
        if agent_requirements.min_reputation < 0.0 || agent_requirements.min_reputation > 1.0 {
            return Err(ConfigError::InvalidRule(format!(
                "agent min_reputation must be between 0.0 and 1.0, got: {}",
                agent_requirements.min_reputation
            )));
        }

        // Validate payment config if present
        if let Some(ref payments) = raw_virtuals.payments {
            validate_payment_config(payments)?;
        }

        Some(VirtualsConfig {
            enabled: raw_virtuals.enabled,
            bug_bounties: BugBountyConfig {
                critical: raw_virtuals.bug_bounties.critical,
                high: raw_virtuals.bug_bounties.high,
                medium: raw_virtuals.bug_bounties.medium,
                low: raw_virtuals.bug_bounties.low,
            },
            agent_requirements: AgentRequirements {
                min_reputation: agent_requirements.min_reputation,
                required_tests: agent_requirements.required_tests,
                human_review_required: agent_requirements.human_review_required,
            },
            payments: raw_virtuals.payments.map(|p| VirtualsPaymentConfig {
                network: p.network,
                token: p.token,
                treasury: p.treasury,
                gas_sponsor: p.gas_sponsor,
            }),
        })
    } else {
        None
    };

    Ok(Config {
        groups,
        permissions,
        x402,
        virtuals,
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
            let rules = parse_flat_rule(s, line)?;
            Ok(rules)
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

                        // "own" expands to all verbs
                        // Read is always repo-level (>*) regardless of own's target
                        if verb_str == "own" || verb_str == "not own" {
                            let deny = verb_str.starts_with("not");
                            rules.push(Rule {
                                subject: subject.clone(),
                                verb: Verb::Read,
                                deny,
                                target: own_read_target(),
                                line,
                            });
                            for &verb in OWN_WRITE_VERBS {
                                rules.push(Rule {
                                    subject: subject.clone(),
                                    verb,
                                    deny,
                                    target: target.clone(),
                                    line,
                                });
                            }
                            continue;
                        }

                        let (deny, verb) = parse_verb_str(verb_str)?;
                        
                        // Semantic validation: reject 'create' with branch targets
                        if verb == Verb::Create && target.branch.is_some() {
                            return Err(ConfigError::InvalidRule(format!(
                                "'create' is for files only - use 'branch' for creating branches. \
                                Change 'create {}' to 'branch {}'",
                                target_str, target_str
                            )));
                        }
                        
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
/// All verbs that `own` expands to.
/// Write-oriented verbs expanded by `own`. Read is handled separately
/// because it's always repo-level (>*) regardless of the own target.
const OWN_WRITE_VERBS: &[Verb] = &[
    Verb::Push, Verb::Merge, Verb::Branch, Verb::Delete, Verb::ForcePush,
    Verb::Edit, Verb::Write, Verb::Append, Verb::Create,
];

/// The repo-level read target, always >*
fn own_read_target() -> Target {
    Target { branch: Some("*".to_string()), path: None }
}

fn parse_flat_rule(s: &str, line: usize) -> Result<Vec<Rule>, ConfigError> {
    let parts: Vec<&str> = s.split_whitespace().collect();

    if parts.len() < 3 {
        return Err(ConfigError::InvalidRule(format!(
            "rule needs at least 3 parts (subject verb target), got: '{s}'"
        )));
    }

    let subject = parse_subject(parts[0])?;

    let (deny, verb_str, target_start) = if parts[1] == "not" {
        if parts.len() < 4 {
            return Err(ConfigError::InvalidRule(format!(
                "deny rule needs 4 parts (subject not verb target), got: '{s}'"
            )));
        }
        (true, parts[2], 3)
    } else {
        (false, parts[1], 2)
    };

    // Remaining parts form the target (e.g. "contracts/** >main")
    let target_str = parts[target_start..].join(" ");
    let target = Target::parse(&target_str)?;

    // "own" expands to all verbs
    // Read is always repo-level (>*) regardless of own's target
    if verb_str == "own" {
        let mut rules = vec![Rule {
            subject: subject.clone(),
            verb: Verb::Read,
            deny,
            target: own_read_target(),
            line,
        }];
        rules.extend(OWN_WRITE_VERBS.iter().map(|&verb| Rule {
            subject: subject.clone(),
            verb,
            deny,
            target: target.clone(),
            line,
        }));
        return Ok(rules);
    }

    let (deny2, verb) = parse_verb_str(verb_str)?;
    let deny = deny || deny2;

    // Semantic validation: reject 'create' with branch targets
    if verb == Verb::Create && target.branch.is_some() {
        return Err(ConfigError::InvalidRule(format!(
            "'create' is for files only - use 'branch' for creating branches. \
            Change 'create {}' to 'branch {}'",
            target_str, target_str
        )));
    }

    Ok(vec![Rule {
        subject,
        verb,
        deny,
        target,
        line,
    }])
}

/// Parse a subject string: bare group name, "evm:0x...", or legacy "%groupname".
fn parse_subject(s: &str) -> Result<Subject, ConfigError> {
    if s == "*" {
        Ok(Subject::All)
    } else if s.starts_with("evm:") || s.starts_with("ens:") || is_ens_name(s) {
        Ok(Subject::Identity(Identity::parse(s)?))
    } else {
        // Bare word = group name. Strip legacy % prefix if present.
        let name = s.strip_prefix('%').unwrap_or(s);
        Ok(Subject::Group(name.to_string()))
    }
}

// Helper function for ENS name detection (copied from config.rs)
fn is_ens_name(s: &str) -> bool {
    if s.is_empty() || s.starts_with('.') || s.ends_with('.') {
        return false;
    }
    
    let dot_pos = s.find('.');
    if dot_pos.is_none() {
        return false;
    }
    
    let dot_index = dot_pos.unwrap();
    if dot_index == 0 {
        return false; // Starts with dot
    }
    
    s.ends_with(".eth") || s.ends_with(".box") || 
    s.ends_with(".com") || s.ends_with(".xyz") || 
    s.ends_with(".org") || s.ends_with(".io") || 
    s.ends_with(".dev") || s.ends_with(".app")
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

/// Validate that a string represents a valid USDC amount.
fn validate_usdc_amount(amount: &str, field_name: &str) -> Result<(), ConfigError> {
    if amount.parse::<f64>().is_err() {
        return Err(ConfigError::InvalidRule(format!(
            "invalid {} bounty amount: '{}' (must be a valid decimal number)",
            field_name, amount
        )));
    }

    let parsed: f64 = amount.parse().unwrap();
    if parsed < 0.0 {
        return Err(ConfigError::InvalidRule(format!(
            "{} bounty amount cannot be negative: '{}'",
            field_name, amount
        )));
    }

    if parsed > 10000.0 {
        return Err(ConfigError::InvalidRule(format!(
            "{} bounty amount seems too large: '{}' (max 10000 USDC)",
            field_name, amount
        )));
    }

    Ok(())
}

/// Validate payment configuration.
fn validate_payment_config(payments: &RawVirtualsPaymentConfig) -> Result<(), ConfigError> {
    // Validate network
    let valid_networks = ["base", "ethereum", "polygon", "arbitrum", "optimism"];
    if !valid_networks.contains(&payments.network.as_str()) {
        return Err(ConfigError::InvalidRule(format!(
            "unsupported payment network: '{}' (supported: {})",
            payments.network,
            valid_networks.join(", ")
        )));
    }

    // Validate treasury address
    if !payments.treasury.starts_with("0x") || payments.treasury.len() != 42 {
        return Err(ConfigError::InvalidRule(format!(
            "treasury must be a valid EVM address: '{}'",
            payments.treasury
        )));
    }

    // Validate gas_sponsor address if present
    if let Some(ref gas_sponsor) = payments.gas_sponsor {
        if !gas_sponsor.starts_with("0x") || gas_sponsor.len() != 42 {
            return Err(ConfigError::InvalidRule(format!(
                "gas_sponsor must be a valid EVM address: '{}'",
                gas_sponsor
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================================================================
    // Section 1: .repobox/config.yml Parser
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
    - "agents not edit .repobox/config.yml"
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
        branch:
          - ">feature/**"
"#;
        let config = parse(yaml).unwrap();
        // Flat rule first, then 2 expanded nested rules
        assert_eq!(config.permissions.rules.len(), 3);
        assert_eq!(config.permissions.rules[0].verb, Verb::Edit);
        assert_eq!(config.permissions.rules[1].verb, Verb::Push);
        assert_eq!(config.permissions.rules[2].verb, Verb::Branch);
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
      - not edit ./.repobox/config.yml
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
      branch:
        - ">feature/**"
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.permissions.rules.len(), 4);
        // founders: 2 rules from list format
        assert!(matches!(&config.permissions.rules[0].subject, Subject::Group(g) if g == "founders"));
        // agents: 2 rules from verb-mapping format
        assert!(matches!(&config.permissions.rules[2].subject, Subject::Group(g) if g == "agents"));
        assert_eq!(config.permissions.rules[2].verb, Verb::Push);
        assert_eq!(config.permissions.rules[3].verb, Verb::Branch);
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

    // ========== "own" verb expansion ==========

    #[test]
    fn test_own_expands_to_all_verbs() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: deny
  rules:
    - founders own >main
"#;
        let config = parse(yaml).unwrap();
        // own expands to 9 verbs (including new branch and create verbs)
        assert_eq!(config.permissions.rules.len(), 10);

        let founder = Identity::parse("evm:0xAAA0000000000000000000000000000000000001").unwrap();
        use crate::engine;

        // All verbs should be allowed
        assert!(engine::check(&config, &founder, Verb::Read, Some("main"), None).is_allowed());
        assert!(engine::check(&config, &founder, Verb::Push, Some("main"), None).is_allowed());
        assert!(engine::check(&config, &founder, Verb::Merge, Some("main"), None).is_allowed());
        assert!(engine::check(&config, &founder, Verb::Branch, Some("main"), None).is_allowed());
        assert!(engine::check(&config, &founder, Verb::Delete, Some("main"), None).is_allowed());
        assert!(engine::check(&config, &founder, Verb::ForcePush, Some("main"), None).is_allowed());
        assert!(engine::check(&config, &founder, Verb::Edit, Some("main"), Some("any.txt")).is_allowed());
        assert!(engine::check(&config, &founder, Verb::Write, Some("main"), Some("any.txt")).is_allowed());
        assert!(engine::check(&config, &founder, Verb::Append, Some("main"), Some("any.txt")).is_allowed());
        assert!(engine::check(&config, &founder, Verb::Create, Some("main"), Some("any.txt")).is_allowed());
    }

    #[test]
    fn test_own_format_b() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: deny
  rules:
    founders:
      - own >main
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.permissions.rules.len(), 10);
    }

    #[test]
    fn test_own_format_c() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: deny
  rules:
    founders:
      own:
        - ">main"
        - ">dev"
"#;
        let config = parse(yaml).unwrap();
        // 9 verbs × 2 targets = 18 rules, plus 2 read rules = 20 rules
        assert_eq!(config.permissions.rules.len(), 20);
    }

    #[test]
    fn test_create_parses_as_file_verb_only() {
        let verb = Verb::parse("create").unwrap();
        assert_eq!(verb, Verb::Create);
        assert!(verb.is_file_verb());
        assert!(!verb.is_branch_verb());
    }

    #[test]  
    fn test_branch_parses_as_branch_verb_only() {
        let verb = Verb::parse("branch").unwrap();
        assert_eq!(verb, Verb::Branch);
        assert!(verb.is_branch_verb());
        assert!(!verb.is_file_verb());
    }

    #[test]
    fn test_create_with_branch_target_rejected() {
        let yaml = r#"
groups:
  devs: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - devs create >feature/test
"#;
        let result = parse(yaml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("'create' is for files only"));
        assert!(err.contains("use 'branch'"));
    }

    #[test]
    fn test_create_with_file_target_allowed() {
        let yaml = r#"
groups:
  devs: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - devs create ./src/**
"#;
        let result = parse(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_branch_with_branch_target_allowed() {
        let yaml = r#"
groups:
  devs: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - devs branch >feature/**
"#;
        let result = parse(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_own_expands_to_include_both_create_and_branch() {
        let yaml = r#"
groups:
  owners: [evm:0xAAA0000000000000000000000000000000000001]
permissions:
  rules:
    - owners own >main
"#;
        let config = parse(yaml).unwrap();
        // Should have 10 rules total (read + 9 write verbs including both branch and create)
        assert_eq!(config.permissions.rules.len(), 10);
        
        let verbs: Vec<_> = config.permissions.rules.iter().map(|r| r.verb).collect();
        assert!(verbs.contains(&Verb::Branch));
        assert!(verbs.contains(&Verb::Create));
    }

    #[test]
    fn test_own_with_deny_exception() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
permissions:
  default: deny
  rules:
    - founders not force-push >main
    - founders own >main
"#;
        let config = parse(yaml).unwrap();
        let founder = Identity::parse("evm:0xAAA0000000000000000000000000000000000001").unwrap();
        use crate::engine;

        // force-push denied (first match wins — deny comes before own)
        assert!(!engine::check(&config, &founder, Verb::ForcePush, Some("main"), None).is_allowed());
        // everything else allowed
        assert!(engine::check(&config, &founder, Verb::Push, Some("main"), None).is_allowed());
        assert!(engine::check(&config, &founder, Verb::Merge, Some("main"), None).is_allowed());
        assert!(engine::check(&config, &founder, Verb::Edit, Some("main"), Some("f.txt")).is_allowed());
    }

    // ========== Remote resolver group parsing ==========

    #[test]
    fn test_http_resolver_group() {
        let yaml = r#"
groups:
  company:
    resolver: http
    url: https://api.example.com/groups/company
    cache_ttl: 60
permissions:
  default: allow
  rules:
    - company push >*
"#;
        let config = parse(yaml).unwrap();
        let group = &config.groups["company"];
        assert!(group.members.is_empty());
        assert!(group.includes.is_empty());
        match &group.resolver {
            Some(GroupResolver::Http { url, cache_ttl }) => {
                assert_eq!(url, "https://api.example.com/groups/company");
                assert_eq!(*cache_ttl, 60);
            }
            other => panic!("expected Http resolver, got: {other:?}"),
        }
    }

    #[test]
    fn test_onchain_resolver_group() {
        let yaml = r#"
groups:
  holders:
    resolver: onchain
    chain: 8453
    contract: "0xDDD0000000000000000000000000000000000004"
    function: isMember
    cache_ttl: 300
permissions:
  default: allow
  rules:
    - holders push >*
"#;
        let config = parse(yaml).unwrap();
        let group = &config.groups["holders"];
        match &group.resolver {
            Some(GroupResolver::Onchain { chain, contract, function, cache_ttl }) => {
                assert_eq!(*chain, 8453);
                assert_eq!(contract, "0xDDD0000000000000000000000000000000000004");
                assert_eq!(function, "isMember");
                assert_eq!(*cache_ttl, 300);
            }
            other => panic!("expected Onchain resolver, got: {other:?}"),
        }
    }

    #[test]
    fn test_onchain_default_function_and_ttl() {
        let yaml = r#"
groups:
  holders:
    resolver: onchain
    chain: 1
    contract: "0xDDD0000000000000000000000000000000000004"
permissions:
  default: allow
  rules: []
"#;
        let config = parse(yaml).unwrap();
        match &config.groups["holders"].resolver {
            Some(GroupResolver::Onchain { function, cache_ttl, .. }) => {
                assert_eq!(function, "isMember"); // default
                assert_eq!(*cache_ttl, 300); // default
            }
            other => panic!("expected Onchain resolver, got: {other:?}"),
        }
    }

    #[test]
    fn test_hybrid_via_composition() {
        let yaml = r#"
groups:
  token-holders:
    resolver: onchain
    chain: 8453
    contract: "0xDDD0000000000000000000000000000000000004"
    function: isMember
  company:
    resolver: http
    url: https://api.example.com/groups/company
  hybrid:
    - evm:0xAAA0000000000000000000000000000000000001
    - token-holders
    - company
permissions:
  default: allow
  rules:
    - hybrid push >*
"#;
        let config = parse(yaml).unwrap();
        let hybrid = &config.groups["hybrid"];
        assert_eq!(hybrid.members.len(), 1); // direct member
        assert_eq!(hybrid.includes.len(), 2); // token-holders + company
        assert!(hybrid.resolver.is_none()); // no resolver on hybrid itself
    }

    #[test]
    fn test_unknown_resolver_type_errors() {
        let yaml = r#"
groups:
  bad:
    resolver: graphql
    url: https://example.com
permissions:
  default: allow
  rules: []
"#;
        let err = parse(yaml);
        assert!(err.is_err());
        let msg = format!("{}", err.unwrap_err());
        assert!(msg.contains("graphql"), "error should mention bad resolver type: {msg}");
    }

    #[test]
    fn test_static_and_resolver_groups_coexist() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  dao:
    resolver: onchain
    chain: 8453
    contract: "0xDDD0000000000000000000000000000000000004"
    function: isMember
permissions:
  default: deny
  rules:
    - founders own >*
    - dao push >feature/**
"#;
        let config = parse(yaml).unwrap();
        assert!(config.groups["founders"].resolver.is_none());
        assert!(config.groups["dao"].resolver.is_some());
        assert_eq!(config.permissions.rules.len(), 11); // 10 from own + 1
    }

    #[test]
    fn test_ens_identity_parsing() {
        let id = Identity::parse("ens:vitalik.eth").unwrap();
        assert_eq!(id.kind, IdentityKind::Ens);
        assert_eq!(id.address, "vitalik.eth");
        assert_eq!(id.canonical(), "ens:vitalik.eth");
        
        // Implicit ENS detection
        let id = Identity::parse("vitalik.eth").unwrap();
        assert_eq!(id.kind, IdentityKind::Ens);
        assert_eq!(id.address, "vitalik.eth");
    }
    
    #[test]
    fn test_ens_name_validation() {
        // Valid ENS names
        assert!(Identity::parse("vitalik.eth").is_ok());
        assert!(Identity::parse("test.box").is_ok());
        assert!(Identity::parse("example.com").is_ok());
        assert!(Identity::parse("my-name.eth").is_ok());
        
        // Invalid ENS names
        assert!(Identity::parse("localhost").is_err()); // no TLD
        assert!(Identity::parse("invalid.xyz123").is_err()); // bad TLD
        assert!(Identity::parse("-invalid.eth").is_err()); // leading hyphen
        assert!(Identity::parse("invalid-.eth").is_err()); // trailing hyphen
        
        // Still support legacy EVM format
        assert!(Identity::parse("0x1234567890123456789012345678901234567890").is_ok());
        assert!(Identity::parse("evm:0x1234567890123456789012345678901234567890").is_ok());
    }
    
    #[test]
    fn test_mixed_group_members() {
        let yaml = r#"
groups:
  mixed:
    - evm:0x1234567890123456789012345678901234567890
    - ens:vitalik.eth
    - alice.eth
permissions:
  default: allow
  rules:
    - mixed push >main
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.groups["mixed"].members.len(), 3);
        
        let members = &config.groups["mixed"].members;
        assert_eq!(members[0].kind, IdentityKind::Evm);
        assert_eq!(members[1].kind, IdentityKind::Ens);
        assert_eq!(members[1].address, "vitalik.eth");
        assert_eq!(members[2].kind, IdentityKind::Ens);
        assert_eq!(members[2].address, "alice.eth");
    }
    
    #[test] 
    fn test_ens_in_permission_rules() {
        let yaml = r#"
permissions:
  default: deny
  rules:
    - "vitalik.eth push >main"
    - "ens:alice.eth edit contracts/**"
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.permissions.rules.len(), 2);
        
        let rule1 = &config.permissions.rules[0];
        match &rule1.subject {
            Subject::Identity(id) => {
                assert_eq!(id.kind, IdentityKind::Ens);
                assert_eq!(id.address, "vitalik.eth");
            }
            _ => panic!("Expected Identity subject"),
        }
        
        let rule2 = &config.permissions.rules[1];
        match &rule2.subject {
            Subject::Identity(id) => {
                assert_eq!(id.kind, IdentityKind::Ens);
                assert_eq!(id.address, "alice.eth");
            }
            _ => panic!("Expected Identity subject"),
        }
    }

    #[test]
    fn test_parse_subject_ens_debug() {
        // Test the exact case from the bug report
        let result = parse_subject("vitalik.eth");
        assert!(result.is_ok(), "Failed to parse vitalik.eth: {:?}", result.err());
        
        let subject = result.unwrap();
        match subject {
            Subject::Identity(id) => {
                assert_eq!(id.kind, IdentityKind::Ens);
                assert_eq!(id.address, "vitalik.eth");
            }
            Subject::Group(name) => panic!("❌ BUG: vitalik.eth incorrectly parsed as Group: {}", name),
            Subject::All => panic!("❌ BUG: vitalik.eth incorrectly parsed as All"),
        }
        
        // Test is_ens_name directly
        assert!(is_ens_name("vitalik.eth"), "is_ens_name should return true for vitalik.eth");
        
        // Test other ENS cases that might be problematic
        assert!(matches!(parse_subject("alice.eth").unwrap(), Subject::Identity(_)));
        assert!(matches!(parse_subject("ens:alice.eth").unwrap(), Subject::Identity(_)));
        assert!(matches!(parse_subject("test.box").unwrap(), Subject::Identity(_)));
        
        // Test non-ENS names should still be groups
        assert!(matches!(parse_subject("notanens").unwrap(), Subject::Group(_)));
        assert!(matches!(parse_subject("founders").unwrap(), Subject::Group(_)));
    }
    
    #[test]
    fn test_is_ens_name_comprehensive() {
        // Test various ENS name formats
        assert!(is_ens_name("vitalik.eth"), "vitalik.eth should be valid");
        assert!(is_ens_name("alice.eth"), "alice.eth should be valid");
        assert!(is_ens_name("test.box"), "test.box should be valid");
        assert!(is_ens_name("example.com"), "example.com should be valid");
        assert!(is_ens_name("subdomain.example.eth"), "subdomain.example.eth should be valid");
        
        // Test invalid cases
        assert!(!is_ens_name("localhost"), "localhost should be invalid (no dot)");
        assert!(!is_ens_name("invalid"), "invalid should be invalid (no dot)");
        assert!(!is_ens_name("test.invalid"), "test.invalid should be invalid (bad TLD)");
        assert!(!is_ens_name(".eth"), ".eth should be invalid (empty name)");
        assert!(!is_ens_name("test."), "test. should be invalid (empty TLD)");
        
        // Edge cases
        assert!(!is_ens_name(""), "empty string should be invalid");
        assert!(!is_ens_name("."), "single dot should be invalid");
        assert!(!is_ens_name(".."), "double dot should be invalid");
    }
    
    #[test]
    fn test_ens_with_groups_present() {
        // Test the scenario where groups exist and ENS names are used in rules
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
    - vitalik.eth

permissions:
  default: deny
  rules:
    - "vitalik.eth push >main"
    - "founders edit *"
"#;
        let result = parse(yaml);
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        
        let config = result.unwrap();
        assert_eq!(config.permissions.rules.len(), 2);
        
        // First rule: vitalik.eth should be parsed as Identity, not Group
        let rule1 = &config.permissions.rules[0];
        match &rule1.subject {
            Subject::Identity(id) => {
                assert_eq!(id.kind, IdentityKind::Ens);
                assert_eq!(id.address, "vitalik.eth");
            }
            Subject::Group(name) => panic!("❌ BUG: vitalik.eth in rule incorrectly parsed as Group: {}", name),
            _ => panic!("Expected Identity subject"),
        }
        
        // Second rule: founders should be parsed as Group
        let rule2 = &config.permissions.rules[1];
        match &rule2.subject {
            Subject::Group(name) => {
                assert_eq!(name, "founders");
            }
            _ => panic!("Expected Group subject"),
        }
    }

    // ========== Virtuals Integration Tests ==========

    #[test]
    fn test_virtuals_config_basic() {
        let yaml = r#"
groups:
  agents:
    - evm:0xBBB0000000000000000000000000000000000002

permissions:
  default: allow
  rules: []

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
        let config = parse(yaml).unwrap();
        
        assert!(config.virtuals.is_some());
        let virtuals = config.virtuals.unwrap();
        
        assert!(virtuals.enabled);
        assert_eq!(virtuals.bug_bounties.critical, "50.00");
        assert_eq!(virtuals.bug_bounties.high, "25.00");
        assert_eq!(virtuals.bug_bounties.medium, "10.00");
        assert_eq!(virtuals.bug_bounties.low, "5.00");
        
        assert_eq!(virtuals.agent_requirements.min_reputation, 0.8);
        assert!(virtuals.agent_requirements.required_tests);
        assert!(virtuals.agent_requirements.human_review_required);
        
        assert!(virtuals.payments.is_some());
        let payments = virtuals.payments.unwrap();
        assert_eq!(payments.network, "base");
        assert_eq!(payments.token, "USDC");
        assert_eq!(payments.treasury, "0x1234567890123456789012345678901234567890");
        assert_eq!(payments.gas_sponsor.unwrap(), "0x9876543210987654321098765432109876543210");
    }

    #[test]
    fn test_virtuals_config_minimal() {
        let yaml = r#"
virtuals:
  enabled: false
  bug_bounties:
    critical: "10.00"
    high: "5.00"
    medium: "2.50"
    low: "1.00"
"#;
        let config = parse(yaml).unwrap();
        
        assert!(config.virtuals.is_some());
        let virtuals = config.virtuals.unwrap();
        
        assert!(!virtuals.enabled);
        assert_eq!(virtuals.bug_bounties.critical, "10.00");
        
        // Should use defaults for agent_requirements
        assert_eq!(virtuals.agent_requirements.min_reputation, 0.8);
        assert!(virtuals.agent_requirements.required_tests);
        assert!(virtuals.agent_requirements.human_review_required);
        
        // No payments config
        assert!(virtuals.payments.is_none());
    }

    #[test]
    fn test_virtuals_config_validation_errors() {
        // Invalid bounty amount
        let yaml = r#"
virtuals:
  enabled: true
  bug_bounties:
    critical: "invalid"
    high: "25.00"
    medium: "10.00"
    low: "5.00"
"#;
        let result = parse(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid critical bounty amount"));

        // Negative bounty amount
        let yaml = r#"
virtuals:
  enabled: true
  bug_bounties:
    critical: "-10.00"
    high: "25.00"
    medium: "10.00"
    low: "5.00"
"#;
        let result = parse(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be negative"));

        // Invalid min_reputation
        let yaml = r#"
virtuals:
  enabled: true
  bug_bounties:
    critical: "50.00"
    high: "25.00"
    medium: "10.00"
    low: "5.00"
  agent_requirements:
    min_reputation: 1.5
"#;
        let result = parse(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("min_reputation must be between 0.0 and 1.0"));
    }

    #[test]
    fn test_virtuals_payment_validation() {
        // Invalid network
        let yaml = r#"
virtuals:
  enabled: true
  bug_bounties:
    critical: "50.00"
    high: "25.00"
    medium: "10.00"
    low: "5.00"
  payments:
    network: "solana"
    token: "USDC"
    treasury: "0x1234567890123456789012345678901234567890"
"#;
        let result = parse(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unsupported payment network"));

        // Invalid treasury address
        let yaml = r#"
virtuals:
  enabled: true
  bug_bounties:
    critical: "50.00"
    high: "25.00"
    medium: "10.00"
    low: "5.00"
  payments:
    network: "base"
    token: "USDC"
    treasury: "invalid"
"#;
        let result = parse(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("treasury must be a valid EVM address"));
    }

    #[test]
    fn test_virtuals_config_defaults() {
        let yaml = r#"
virtuals:
  bug_bounties:
    critical: "50.00"
    high: "25.00"
    medium: "10.00"
    low: "5.00"
"#;
        let config = parse(yaml).unwrap();
        
        let virtuals = config.virtuals.unwrap();
        
        // Should default to enabled: true
        assert!(virtuals.enabled);
        
        // Should use all defaults for agent_requirements
        assert_eq!(virtuals.agent_requirements.min_reputation, 0.8);
        assert!(virtuals.agent_requirements.required_tests);
        assert!(virtuals.agent_requirements.human_review_required);
    }

    #[test]
    fn test_config_without_virtuals() {
        let yaml = r#"
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001

permissions:
  default: allow
  rules: []
"#;
        let config = parse(yaml).unwrap();
        assert!(config.virtuals.is_none());
    }
}
