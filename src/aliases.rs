use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::identity::repobox_home_with_base;

/// Get the aliases file path.
fn aliases_path(base: &Path) -> std::path::PathBuf {
    repobox_home_with_base(base).join("aliases")
}

/// Read all aliases from ~/.repobox/aliases.
pub fn read_aliases(base: &Path) -> HashMap<String, String> {
    let path = aliases_path(base);
    match fs::read_to_string(&path) {
        Ok(content) => parse_aliases(&content),
        Err(_) => HashMap::new(),
    }
}

/// Parse aliases from file content.
fn parse_aliases(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, addr)) = line.split_once('=') {
            let name = name.trim().to_string();
            let addr = addr.trim().to_string();
            if !name.is_empty() && !addr.is_empty() {
                map.insert(name, addr);
            }
        }
    }
    map
}

/// Write all aliases back to file.
fn write_aliases(base: &Path, aliases: &HashMap<String, String>) -> Result<(), std::io::Error> {
    let path = aliases_path(base);
    fs::create_dir_all(path.parent().unwrap())?;

    let mut lines: Vec<String> = aliases
        .iter()
        .map(|(k, v)| format!("{k} = {v}"))
        .collect();
    lines.sort();

    fs::write(&path, lines.join("\n") + "\n")
}

/// Add or update an alias.
pub fn set_alias(base: &Path, name: &str, address: &str) -> Result<(), std::io::Error> {
    validate_alias_name(name).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let mut aliases = read_aliases(base);
    aliases.insert(name.to_string(), address.to_string());
    write_aliases(base, &aliases)
}

/// Remove an alias.
pub fn remove_alias(base: &Path, name: &str) -> Result<bool, std::io::Error> {
    let mut aliases = read_aliases(base);
    let removed = aliases.remove(name).is_some();
    if removed {
        write_aliases(base, &aliases)?;
    }
    Ok(removed)
}

/// Resolve an alias name to an address.
pub fn resolve_alias(base: &Path, name: &str) -> Result<Option<String>, std::io::Error> {
    let aliases = read_aliases(base);
    Ok(aliases.get(name).cloned())
}

/// Reverse lookup: find the alias for an address.
pub fn get_alias_for_address(base: &Path, address: &str) -> Option<String> {
    let aliases = read_aliases(base);
    aliases
        .iter()
        .find(|(_, v)| v.as_str() == address)
        .map(|(k, _)| k.clone())
}

/// Format an address with its alias for display.
pub fn display_identity(base: &Path, address: &str) -> String {
    match get_alias_for_address(base, address) {
        Some(alias) => format!("{alias} ({address})"),
        None => address.to_string(),
    }
}

/// Validate alias name — cannot start with special characters.
fn validate_alias_name(name: &str) -> Result<(), String> {
    if name.starts_with('%') || name.starts_with('@') {
        return Err("alias name should be a bare word (no @ or % prefix)".to_string());
    }
    if name.is_empty() {
        return Err("alias name cannot be empty".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_set_and_read_alias() {
        let tmp = TempDir::new().unwrap();
        set_alias(tmp.path(), "claude", "evm:0xBBB0000000000000000000000000000000000002").unwrap();

        let aliases = read_aliases(tmp.path());
        assert_eq!(
            aliases.get("claude"),
            Some(&"evm:0xBBB0000000000000000000000000000000000002".to_string())
        );
    }

    #[test]
    fn test_alias_list() {
        let tmp = TempDir::new().unwrap();
        set_alias(tmp.path(), "alice", "evm:0xAAA0000000000000000000000000000000000001").unwrap();
        set_alias(tmp.path(), "claude", "evm:0xBBB0000000000000000000000000000000000002").unwrap();

        let aliases = read_aliases(tmp.path());
        assert_eq!(aliases.len(), 2);
    }

    #[test]
    fn test_remove_alias() {
        let tmp = TempDir::new().unwrap();
        set_alias(tmp.path(), "claude", "evm:0xBBB0000000000000000000000000000000000002").unwrap();
        let removed = remove_alias(tmp.path(), "claude").unwrap();
        assert!(removed);

        let aliases = read_aliases(tmp.path());
        assert!(aliases.is_empty());
    }

    #[test]
    fn test_aliases_per_machine() {
        let tmp = TempDir::new().unwrap();
        set_alias(tmp.path(), "claude", "evm:0xBBB0000000000000000000000000000000000002").unwrap();

        // Aliases file is at ~/.repobox/aliases, not in any repo
        let path = aliases_path(tmp.path());
        assert!(path.exists());
        assert!(path.to_str().unwrap().contains(".repobox"));
    }

    #[test]
    fn test_display_with_alias() {
        let tmp = TempDir::new().unwrap();
        set_alias(tmp.path(), "claude", "evm:0xBBB0000000000000000000000000000000000002").unwrap();

        let display = display_identity(tmp.path(), "evm:0xBBB0000000000000000000000000000000000002");
        assert_eq!(display, "claude (evm:0xBBB0000000000000000000000000000000000002)");
    }

    #[test]
    fn test_display_without_alias() {
        let tmp = TempDir::new().unwrap();
        let display = display_identity(tmp.path(), "evm:0xBBB0000000000000000000000000000000000002");
        assert_eq!(display, "evm:0xBBB0000000000000000000000000000000000002");
    }

    #[test]
    fn test_plus_notation_alias() {
        let tmp = TempDir::new().unwrap();
        set_alias(
            tmp.path(),
            "claude+roudy-piglet",
            "evm:0xCCC0000000000000000000000000000000000003",
        )
        .unwrap();

        let resolved = resolve_alias(tmp.path(), "claude+roudy-piglet").unwrap();
        assert_eq!(
            resolved,
            Some("evm:0xCCC0000000000000000000000000000000000003".to_string())
        );
    }

    #[test]
    fn test_alias_cannot_start_with_percent() {
        let tmp = TempDir::new().unwrap();
        let result = set_alias(tmp.path(), "%claude", "evm:0xBBB0000000000000000000000000000000000002");
        assert!(result.is_err());
    }

    #[test]
    fn test_no_alias_returns_none() {
        let tmp = TempDir::new().unwrap();
        let resolved = resolve_alias(tmp.path(), "nonexistent").unwrap();
        assert_eq!(resolved, None);
    }

    #[test]
    fn test_reverse_lookup() {
        let tmp = TempDir::new().unwrap();
        set_alias(tmp.path(), "claude", "evm:0xBBB0000000000000000000000000000000000002").unwrap();

        let alias = get_alias_for_address(tmp.path(), "evm:0xBBB0000000000000000000000000000000000002");
        assert_eq!(alias, Some("claude".to_string()));
    }
}
