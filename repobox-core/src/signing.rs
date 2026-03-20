use std::fs;
use std::path::Path;

use k256::ecdsa::{RecoveryId, SigningKey, Signature, VerifyingKey, signature::hazmat::PrehashSigner};
use sha3::{Digest, Keccak256};

use crate::config::ConfigError;
use crate::identity::repobox_home_with_base;

/// Sign data with the private key for the given address.
/// Returns 65 bytes: 64-byte ECDSA signature (r || s) + 1-byte recovery ID.
/// This enables ecrecover on the verifier side without needing the public key.
pub fn sign(base: &Path, address: &str, data: &[u8]) -> Result<Vec<u8>, ConfigError> {
    let key = load_key(base, address)?;
    let signing_key = SigningKey::from_bytes((&key[..]).into())
        .map_err(|e| ConfigError::InvalidIdentity(format!("invalid key: {e}")))?;

    // Hash the data with Keccak256 (Ethereum-style)
    let hash = keccak256(data);

    // Sign the hash, producing a recoverable signature
    let (signature, recovery_id): (Signature, RecoveryId) = signing_key
        .sign_prehash(&hash)
        .map_err(|e| ConfigError::InvalidIdentity(format!("signing failed: {e}")))?;

    // 64 bytes (r || s) + 1 byte recovery id
    let mut result = signature.to_vec();
    result.push(recovery_id.to_byte());
    Ok(result)
}

/// Verify a signature against data and a claimed EVM address.
/// Uses ecrecover: recovers the public key from the signature,
/// derives the EVM address, and checks it matches.
pub fn verify(address: &str, data: &[u8], signature_bytes: &[u8]) -> Result<bool, ConfigError> {
    let recovered = recover_address(data, signature_bytes)?;
    Ok(recovered.eq_ignore_ascii_case(address))
}

/// Recover the EVM address that signed the given data.
/// signature_bytes must be 65 bytes: 64-byte sig + 1-byte recovery ID.
pub fn recover_address(data: &[u8], signature_bytes: &[u8]) -> Result<String, ConfigError> {
    if signature_bytes.len() != 65 {
        return Err(ConfigError::InvalidIdentity(format!(
            "signature must be 65 bytes (64 + recovery id), got {}",
            signature_bytes.len()
        )));
    }

    let sig = Signature::from_slice(&signature_bytes[..64])
        .map_err(|e| ConfigError::InvalidIdentity(format!("invalid signature: {e}")))?;
    let recovery_id = RecoveryId::from_byte(signature_bytes[64])
        .ok_or_else(|| ConfigError::InvalidIdentity("invalid recovery id".to_string()))?;

    let hash = keccak256(data);

    let verifying_key = VerifyingKey::recover_from_prehash(&hash, &sig, recovery_id)
        .map_err(|e| ConfigError::InvalidIdentity(format!("ecrecover failed: {e}")))?;

    // Derive EVM address from recovered public key
    let public_key_bytes = verifying_key.to_encoded_point(false);
    let pub_bytes = &public_key_bytes.as_bytes()[1..]; // skip 0x04 prefix

    let addr_hash = keccak256(pub_bytes);
    let address_bytes = &addr_hash[12..];
    Ok(crate::identity::eip55_checksum_bytes(address_bytes))
}

/// GPG program protocol: git calls us with --sign or --verify.
/// This handles the gpg.program interface.
pub fn handle_gpg_sign(_status_fd: Option<&str>, data: &[u8], base: &Path, address: &str) -> Result<Vec<u8>, ConfigError> {
    sign(base, address, data)
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

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
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
    fn test_sign_produces_recoverable_signature() {
        let tmp = setup_with_key();
        let data = b"tree abc123\nauthor Test\n\ntest commit";
        let sig = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        assert_eq!(sig.len(), 65, "signature must be 65 bytes (64 + recovery id)");
    }

    #[test]
    fn test_sign_is_deterministic() {
        let tmp = setup_with_key();
        let data = b"test data";
        let sig1 = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        let sig2 = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_verify_roundtrip() {
        let tmp = setup_with_key();
        let data = b"test commit data";
        let sig = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        let valid = verify(TEST_ADDRESS, data, &sig).unwrap();
        assert!(valid, "signature should verify against the signer's address");
    }

    #[test]
    fn test_verify_wrong_address_fails() {
        let tmp = setup_with_key();
        let data = b"test commit data";
        let sig = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        let valid = verify("0x0000000000000000000000000000000000000001", data, &sig).unwrap();
        assert!(!valid, "signature should NOT verify against a different address");
    }

    #[test]
    fn test_recover_address() {
        let tmp = setup_with_key();
        let data = b"some commit content";
        let sig = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        let recovered = recover_address(data, &sig).unwrap();
        assert_eq!(
            recovered.to_lowercase(),
            TEST_ADDRESS.to_lowercase(),
            "recovered address should match signer"
        );
    }

    #[test]
    fn test_verify_wrong_data_fails() {
        let tmp = setup_with_key();
        let data = b"original data";
        let sig = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        let valid = verify(TEST_ADDRESS, b"tampered data", &sig).unwrap();
        assert!(!valid, "signature should NOT verify against different data");
    }

    #[test]
    fn test_invalid_signature_length() {
        let result = verify(TEST_ADDRESS, b"data", &[0u8; 32]);
        assert!(result.is_err());
    }

    #[test]
    fn test_gpg_verify_interface() {
        let tmp = setup_with_key();
        let data = b"test commit data";
        let sig = sign(tmp.path(), TEST_ADDRESS, data).unwrap();
        let result = handle_gpg_verify(data, &sig, TEST_ADDRESS).unwrap();
        assert!(result.contains("EVM-signed by"));
        assert!(result.contains(TEST_ADDRESS));
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
