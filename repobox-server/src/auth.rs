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

    // Support both Bearer and Basic auth schemes.
    // Bearer: Authorization: Bearer <sig_hex>:<timestamp>
    // Basic:  Authorization: Basic base64(evm:<sig_hex>:<timestamp>)  (git's native format)
    let token = if let Some(bearer) = auth.strip_prefix("Bearer ").or_else(|| auth.strip_prefix("bearer ")) {
        bearer.to_string()
    } else if let Some(basic) = auth.strip_prefix("Basic ").or_else(|| auth.strip_prefix("basic ")) {
        // Decode base64 → "evm:<sig_hex>:<timestamp>"
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(basic.trim())
            .map_err(|_| "invalid base64 in Basic auth")?;
        let decoded_str = String::from_utf8(decoded)
            .map_err(|_| "invalid UTF-8 in Basic auth")?;
        // Strip "evm:" username prefix → "<sig_hex>:<timestamp>"
        decoded_str
            .strip_prefix("evm:")
            .ok_or("Basic auth username must be 'evm'")?
            .to_string()
    } else {
        return Err("auth must use Bearer or Basic scheme".into());
    };

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

    // Reconstruct the signed message: "{repo_path}:{timestamp}"
    // (The signing module will keccak256 this before verifying)
    let message = format!("{repo_path}:{timestamp}");
    tracing::debug!(
        message = %message,
        sig_hex_len = sig_hex.len(),
        timestamp = timestamp,
        "verifying auth signature"
    );

    // Recover the signer address from the signature
    let sig_bytes = hex::decode(sig_hex.strip_prefix("0x").unwrap_or(sig_hex))
        .map_err(|_| "invalid signature hex")?;

    if sig_bytes.len() != 65 {
        return Err(format!("signature must be 65 bytes, got {}", sig_bytes.len()));
    }

    // Use repobox::signing::recover_address which handles keccak256 internally
    let address = repobox::signing::recover_address(message.as_bytes(), &sig_bytes)
        .map_err(|e| format!("signature recovery failed: {e}"))?;
    tracing::debug!(recovered_address = %address, "signature verified");
    let identity = Identity::parse(&format!("evm:{address}"))
        .map_err(|e| format!("invalid recovered address: {e}"))?;

    Ok(Some(identity))
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
