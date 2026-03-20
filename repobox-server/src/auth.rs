//! EVM signature authentication for Git Smart HTTP.
//!
//! Auth scheme: `Authorization: Bearer <hex_signature>:<unix_timestamp>`
//! The client signs: keccak256("{repo_path}:{timestamp}")
//! The server recovers the EVM address and uses it as the identity.
//!
//! Timestamp must be within 5 minutes of server time.

use axum::http::HeaderMap;
use repobox::config::Identity;

const MAX_TIMESTAMP_DRIFT_SECS: u64 = 300; // 5 minutes

/// Extract an authenticated EVM identity from request headers.
///
/// Returns `Ok(Some(identity))` if valid auth is present.
/// Returns `Ok(None)` if no auth header is present (anonymous).
/// Returns `Err(message)` if auth is present but invalid.
pub fn extract_identity(headers: &HeaderMap, repo_path: &str) -> Result<Option<Identity>, String> {
    let auth = match headers.get("authorization") {
        Some(v) => v.to_str().map_err(|_| "invalid auth header encoding")?,
        None => return Ok(None),
    };

    let token = auth
        .strip_prefix("Bearer ")
        .or_else(|| auth.strip_prefix("bearer "))
        .ok_or("auth must use Bearer scheme")?;

    // Parse "hex_signature:timestamp"
    let (sig_hex, timestamp_str) = token
        .rsplit_once(':')
        .ok_or("auth token must be signature:timestamp")?;

    let timestamp: u64 = timestamp_str
        .parse()
        .map_err(|_| "invalid timestamp")?;

    // Check timestamp freshness
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now.abs_diff(timestamp) > MAX_TIMESTAMP_DRIFT_SECS {
        return Err("timestamp too old or too far in the future".into());
    }

    // Reconstruct the signed message: keccak256("{repo_path}:{timestamp}")
    let message = format!("{repo_path}:{timestamp}");
    let message_hash = keccak256(message.as_bytes());

    // Recover the signer address from the signature
    let sig_bytes = hex::decode(sig_hex.strip_prefix("0x").unwrap_or(sig_hex))
        .map_err(|_| "invalid signature hex")?;

    if sig_bytes.len() != 65 {
        return Err(format!("signature must be 65 bytes, got {}", sig_bytes.len()));
    }

    let address = recover_address(&message_hash, &sig_bytes)?;
    let identity = Identity::parse(&format!("evm:{address}"))
        .map_err(|e| format!("invalid recovered address: {e}"))?;

    Ok(Some(identity))
}

/// Keccak-256 hash.
fn keccak256(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Recover an EVM address from a message hash and 65-byte signature.
/// Uses secp256k1 ECDSA recovery (same as Ethereum's ecrecover).
fn recover_address(message_hash: &[u8; 32], sig: &[u8]) -> Result<String, String> {
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

    // Ethereum signature: [r (32 bytes), s (32 bytes), v (1 byte)]
    let v = match sig[64] {
        0 | 27 => 0u8,
        1 | 28 => 1u8,
        v => return Err(format!("invalid recovery id: {v}")),
    };
    let recovery_id = RecoveryId::from_byte(v)
        .ok_or("invalid recovery id")?;

    let signature = Signature::from_slice(&sig[..64])
        .map_err(|e| format!("invalid signature: {e}"))?;

    let recovered_key = VerifyingKey::recover_from_prehash(message_hash, &signature, recovery_id)
        .map_err(|e| format!("signature recovery failed: {e}"))?;

    // Derive address: keccak256(uncompressed_pubkey[1..]) → last 20 bytes
    let pubkey_bytes = recovered_key.to_encoded_point(false);
    let pubkey_hash = keccak256(&pubkey_bytes.as_bytes()[1..]);
    let address = &pubkey_hash[12..32];

    Ok(format!("0x{}", hex::encode(address)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_auth_returns_none() {
        let headers = HeaderMap::new();
        let result = extract_identity(&headers, "alice/my-repo");
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_bad_scheme_returns_error() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Basic abc123".parse().unwrap());
        let result = extract_identity(&headers, "alice/my-repo");
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_timestamp_returns_error() {
        let mut headers = HeaderMap::new();
        // Timestamp from 2020 — way too old
        headers.insert("authorization", "Bearer 0xdeadbeef:1577836800".parse().unwrap());
        let result = extract_identity(&headers, "alice/my-repo");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timestamp"));
    }
}
