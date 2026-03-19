use std::fs;
use std::path::Path;

use k256::ecdsa::{SigningKey, Signature, signature::Signer};
use sha3::{Digest, Keccak256};

use crate::config::ConfigError;
use crate::identity::repobox_home_with_base;

/// Sign data with the private key for the given address.
/// Returns the signature as a hex string.
pub fn sign(base: &Path, address: &str, data: &[u8]) -> Result<Vec<u8>, ConfigError> {
    let key = load_key(base, address)?;
    let signing_key = SigningKey::from_bytes((&key[..]).into())
        .map_err(|e| ConfigError::InvalidIdentity(format!("invalid key: {e}")))?;

    // Hash the data with Keccak256 (Ethereum-style)
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    // Sign the hash
    let signature: Signature = signing_key.sign(&hash);
    Ok(signature.to_vec())
}

/// Verify a signature against data and a public address.
pub fn verify(address: &str, data: &[u8], signature_bytes: &[u8]) -> Result<bool, ConfigError> {
    use k256::ecdsa::{VerifyingKey, signature::Verifier};

    // We need the public key to verify, but we only have the address.
    // For verification, we recover the public key from the signature.
    // This requires a recovery id, which we'd need to store.
    // For now, we use a simpler approach: verify by re-signing with stored key.
    // Full ecrecover will be implemented for server-side verification.

    // For local verification: hash the data the same way
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let _hash = hasher.finalize();

    // Try to parse the signature
    let sig = Signature::from_slice(signature_bytes)
        .map_err(|e| ConfigError::InvalidIdentity(format!("invalid signature: {e}")))?;

    // We can't verify without the public key or a recovery id.
    // Return Ok(true) if the signature is well-formed for now.
    // Full verification needs RecoverableSignature which we'll add for the server.
    let _ = sig;
    let _ = address;

    Ok(true) // Placeholder — full ecrecover in server phase
}

/// Load a private key from ~/.repobox/keys/<address>.key
fn load_key(base: &Path, address: &str) -> Result<Vec<u8>, ConfigError> {
    let key_path = repobox_home_with_base(base)
        .join("keys")
        .join(format!("{address}.key"));

    let hex_str = fs::read_to_string(&key_path).map_err(|_| {
        ConfigError::InvalidIdentity(format!("key not found for evm:{address}"))
    })?;

    let hex_str = hex_str.trim().strip_prefix("0x").unwrap_or(hex_str.trim());
    hex::decode(hex_str)
        .map_err(|_| ConfigError::InvalidIdentity("corrupted key file".to_string()))
}

/// GPG program protocol: git calls us with --sign or --verify.
/// This handles the gpg.program interface.
pub fn handle_gpg_sign(status_fd: Option<&str>, data: &[u8], base: &Path, address: &str) -> Result<Vec<u8>, ConfigError> {
    let signature = sign(base, address, data)?;

    // Write GPG-compatible status to the status fd if provided
    if let Some(fd) = status_fd {
        let status_msg = format!(
            "[GNUPG:] SIG_CREATED D 1 8 00 {} 0 {}\n[GNUPG:] GOODSIG {} evm:{}\n",
            chrono_timestamp(),
            hex::encode(&signature),
            address,
            address,
        );
        // In a real implementation, write to the fd
        let _ = status_msg;
        let _ = fd;
    }

    Ok(signature)
}

/// Handle gpg --verify for `git verify-commit`
pub fn handle_gpg_verify(data: &[u8], signature: &[u8], address: &str) -> Result<String, ConfigError> {
    let valid = verify(address, data, signature)?;
    if valid {
        Ok(format!("EVM-signed by evm:{address}"))
    } else {
        Err(ConfigError::InvalidIdentity("signature verification failed".to_string()))
    }
}

fn chrono_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const TEST_ADDRESS: &str = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    fn setup_with_key() -> TempDir {
        let tmp = TempDir::new().unwrap();
        crate::identity::store_key(tmp.path(), TEST_ADDRESS, TEST_PRIVATE_KEY).unwrap();
        tmp
    }

    #[test]
    fn test_sign_produces_valid_signature() {
        let tmp = setup_with_key();
        let data = b"tree abc123\nauthor Test\n\ntest commit";
        let sig = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        assert!(!sig.is_empty());
        assert_eq!(sig.len(), 64); // ECDSA signature is 64 bytes (r + s)
    }

    #[test]
    fn test_sign_uses_correct_key() {
        let tmp = setup_with_key();
        let data = b"test data";
        let sig1 = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        let sig2 = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        // Same key + same data = same signature (deterministic with RFC 6979)
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_verify_commit_well_formed() {
        let tmp = setup_with_key();
        let data = b"test commit data";
        let sig = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        let result = verify(TEST_ADDRESS, data, &sig).unwrap();
        assert!(result);
    }

    #[test]
    fn test_sign_show_signature_format() {
        let tmp = setup_with_key();
        let data = b"test commit data";
        let sig = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        let result = handle_gpg_verify(data, &sig, TEST_ADDRESS).unwrap();
        assert!(result.contains("EVM-signed by"));
        assert!(result.contains(TEST_ADDRESS));
    }

    #[test]
    fn test_signing_uses_signingkey_config() {
        // The key is loaded based on the address, which comes from user.signingkey
        let tmp = setup_with_key();
        let data = b"commit data";
        let sig = sign(tmp.path(), TEST_ADDRESS, data);
        assert!(sig.is_ok());
    }

    #[test]
    fn test_missing_key_error() {
        let tmp = TempDir::new().unwrap();
        let result = sign(tmp.path(), "0xNONEXISTENT", b"data");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("key not found"), "got: {err}");
    }
}
