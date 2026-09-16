//! Opaque tokens. The raw value is generated once, handed to exactly one
//! channel (a file the operator reads, a page shown once, a redirect) and never
//! persisted: the store only ever sees `hash(raw)`.

use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// 256 bits of OS randomness, base64url without padding (43 characters).
pub fn generate() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Hex SHA-256 of the raw token. This is the only form stored in the database.
pub fn hash(raw: &str) -> String {
    let digest = Sha256::digest(raw.as_bytes());
    let mut out = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Reject anything that cannot be one of our tokens before touching the
/// database, so junk from scanners never reaches a query.
pub fn looks_like_token(raw: &str) -> bool {
    raw.len() == 43
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape() {
        let t = generate();
        assert!(looks_like_token(&t), "{t}");
        assert_ne!(generate(), t);
        assert_eq!(hash(&t).len(), 64);
        assert_ne!(hash(&t), hash("other"));
        assert!(!looks_like_token("short"));
        assert!(!looks_like_token(&format!("{t}=")));
    }
}
