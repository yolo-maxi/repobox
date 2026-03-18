use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{ConfigError, Identity};

/// Get the repobox home directory (~/.repobox/).
pub fn repobox_home() -> PathBuf {
    dirs_or_default().join(".repobox")
}

fn dirs_or_default() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

/// Override for testing — use a temp dir as home.
pub fn repobox_home_with_base(base: &Path) -> PathBuf {
    base.join(".repobox")
}

/// Derive an EVM address from a private key (hex string).
/// Returns the checksummed address string (0x-prefixed).
pub fn derive_address(private_key_hex: &str) -> Result<String, ConfigError> {
    let hex = private_key_hex.strip_prefix("0x").unwrap_or(private_key_hex);

    if hex.len() != 64 {
        return Err(ConfigError::InvalidIdentity(format!(
            "private key must be 32 bytes (64 hex chars), got {}",
            hex.len()
        )));
    }

    // Validate hex
    let key_bytes = hex::decode(hex).map_err(|_| {
        ConfigError::InvalidIdentity("private key is not valid hex".to_string())
    })?;

    // Use k256 to derive the public key and address
    use k256::ecdsa::SigningKey;
    use sha3::{Digest, Keccak256};

    let signing_key = SigningKey::from_bytes((&key_bytes[..]).into())
        .map_err(|e| ConfigError::InvalidIdentity(format!("invalid private key: {e}")))?;

    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.to_encoded_point(false);
    // Skip the 0x04 prefix byte
    let pub_bytes = &public_key_bytes.as_bytes()[1..];

    let mut hasher = Keccak256::new();
    hasher.update(pub_bytes);
    let hash = hasher.finalize();

    // Last 20 bytes of the Keccak hash = address
    let address_bytes = &hash[12..];
    let address = format!("0x{}", hex::encode(address_bytes));

    // EIP-55 checksum
    Ok(eip55_checksum(&address))
}

/// EIP-55 mixed-case checksum encoding.
fn eip55_checksum(address: &str) -> String {
    let addr = address.strip_prefix("0x").unwrap_or(address).to_lowercase();

    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(addr.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);

    let mut result = String::from("0x");
    for (i, c) in addr.chars().enumerate() {
        if c.is_ascii_alphabetic() {
            let hash_nibble = u8::from_str_radix(&hash_hex[i..i + 1], 16).unwrap_or(0);
            if hash_nibble >= 8 {
                result.push(c.to_ascii_uppercase());
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Store a private key in ~/.repobox/keys/<address>.key
pub fn store_key(base: &Path, address: &str, private_key_hex: &str) -> Result<PathBuf, std::io::Error> {
    let keys_dir = repobox_home_with_base(base).join("keys");
    fs::create_dir_all(&keys_dir)?;

    let key_path = keys_dir.join(format!("{address}.key"));
    fs::write(&key_path, private_key_hex)?;

    // Restrictive permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(key_path)
}

/// Store the current identity in ~/.repobox/identity
pub fn set_identity(base: &Path, address: &str) -> Result<(), std::io::Error> {
    let home = repobox_home_with_base(base);
    fs::create_dir_all(&home)?;
    fs::write(home.join("identity"), format!("evm:{address}"))
}

/// Read the current identity from ~/.repobox/identity
pub fn get_identity(base: &Path) -> Result<Option<Identity>, ConfigError> {
    let path = repobox_home_with_base(base).join("identity");
    match fs::read_to_string(&path) {
        Ok(s) => Ok(Some(Identity::parse(s.trim())?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ConfigError::InvalidIdentity(format!(
            "failed to read identity: {e}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Known test vector: private key → address
    const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    // This is the well-known Hardhat account #0
    const TEST_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    #[test]
    fn test_derive_address() {
        let addr = derive_address(TEST_PRIVATE_KEY).unwrap();
        assert_eq!(addr, TEST_ADDRESS);
    }

    #[test]
    fn test_identity_set_and_get() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        set_identity(base, TEST_ADDRESS).unwrap();
        let id = get_identity(base).unwrap().unwrap();
        assert_eq!(id.address, TEST_ADDRESS);
    }

    #[test]
    fn test_identity_set_with_alias() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        // Set identity
        set_identity(base, TEST_ADDRESS).unwrap();

        // Set alias
        crate::aliases::set_alias(base, "alice", &format!("evm:{TEST_ADDRESS}")).unwrap();

        // Read back
        let id = get_identity(base).unwrap().unwrap();
        assert_eq!(id.address, TEST_ADDRESS);

        let resolved = crate::aliases::resolve_alias(base, "alice").unwrap();
        assert_eq!(resolved, Some(format!("evm:{TEST_ADDRESS}")));
    }

    #[test]
    fn test_whoami_with_alias() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        set_identity(base, TEST_ADDRESS).unwrap();
        crate::aliases::set_alias(base, "alice", &format!("evm:{TEST_ADDRESS}")).unwrap();

        let id = get_identity(base).unwrap().unwrap();
        let alias = crate::aliases::get_alias_for_address(base, &id.to_string());
        assert_eq!(alias, Some("alice".to_string()));
    }

    #[test]
    fn test_whoami_without_alias() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        set_identity(base, TEST_ADDRESS).unwrap();

        let id = get_identity(base).unwrap().unwrap();
        let alias = crate::aliases::get_alias_for_address(base, &id.to_string());
        assert_eq!(alias, None);
    }

    #[test]
    fn test_identity_stored_globally() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        set_identity(base, TEST_ADDRESS).unwrap();

        // Check it's at ~/.repobox/identity, not per-repo
        let path = repobox_home_with_base(base).join("identity");
        assert!(path.exists());
    }

    #[test]
    fn test_no_identity_returns_none() {
        let tmp = TempDir::new().unwrap();
        let id = get_identity(tmp.path()).unwrap();
        assert!(id.is_none());
    }

    #[test]
    fn test_store_key() {
        let tmp = TempDir::new().unwrap();
        let path = store_key(tmp.path(), TEST_ADDRESS, TEST_PRIVATE_KEY).unwrap();
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, TEST_PRIVATE_KEY);
    }
}
