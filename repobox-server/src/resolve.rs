//! On-chain group resolver proxy.
//!
//! GET /api/resolve?chain=8453&contract=0x...&function=isMember&address=0x...
//! → { "member": true/false }
//!
//! Proxies eth_call to Alchemy RPC for the specified chain.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

use crate::AppState;

// Global cache for ENS resolutions (name -> (address, timestamp))
lazy_static::lazy_static! {
    static ref ENS_CACHE: Mutex<HashMap<String, (String, Instant)>> = Mutex::new(HashMap::new());
}

const ENS_CACHE_DURATION_SECS: u64 = 300; // 5 minutes
const ENS_UNIVERSAL_RESOLVER: &str = "0xce01f8eee7E479C928F8919abD53E553a36CeF67";

pub(crate) fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/resolve", get(resolve_membership))
}

#[derive(Debug, Deserialize)]
struct ResolveQuery {
    chain: u64,
    contract: String,
    function: String,
    address: String,
}

#[derive(Debug, Serialize)]
struct ResolveResponse {
    member: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Alchemy chain ID → subdomain mapping.
/// Covers all EVM chains Alchemy supports (mainnets + testnets).
fn alchemy_rpc_url(chain: u64, api_key: &str) -> Option<String> {
    let network = match chain {
        // Ethereum
        1 => "eth-mainnet",
        11155111 => "eth-sepolia",
        17000 => "eth-holesky",
        // Optimism
        10 => "opt-mainnet",
        11155420 => "opt-sepolia",
        // Arbitrum
        42161 => "arb-mainnet",
        421614 => "arb-sepolia",
        // Base
        8453 => "base-mainnet",
        84532 => "base-sepolia",
        // Polygon
        137 => "polygon-mainnet",
        80002 => "polygon-amoy",
        // zkSync
        324 => "zksync-mainnet",
        300 => "zksync-sepolia",
        // Blast
        81457 => "blast-mainnet",
        168587773 => "blast-sepolia",
        // Linea
        59144 => "linea-mainnet",
        59141 => "linea-sepolia",
        // Scroll
        534352 => "scroll-mainnet",
        534351 => "scroll-sepolia",
        // Mantle
        5000 => "mantle-mainnet",
        5003 => "mantle-sepolia",
        // Gnosis
        100 => "gnosis-mainnet",
        // Celo
        42220 => "celo-mainnet",
        // Avalanche
        43114 => "avax-mainnet",
        43113 => "avax-fuji",
        // BNB Chain
        56 => "bnb-mainnet",
        97 => "bnb-testnet",
        // Metis
        1088 => "metis-mainnet",
        // Zora
        7777777 => "zora-mainnet",
        999999999 => "zora-sepolia",
        // World Chain
        480 => "worldchain-mainnet",
        // Shape
        360 => "shape-mainnet",
        // Soneium
        1868 => "soneium-mainnet",
        // ZetaChain
        7000 => "zetachain-mainnet",
        // Berachain
        80094 => "berachain-mainnet",
        // Unichain
        130 => "unichain-mainnet",
        1301 => "unichain-sepolia",
        // Ink
        57073 => "ink-mainnet",
        763373 => "ink-sepolia",
        // Abstract
        2741 => "abstract-mainnet",
        // Lens
        37111 => "lens-sepolia",
        // Anime
        69000 => "anime-mainnet",
        // Sonic
        146 => "sonic-mainnet",
        57054 => "sonic-blaze",
        // Degen
        666666666 => "degen-mainnet",
        // Frax
        252 => "frax-mainnet",
        // Polynomial
        8008 => "polynomial-mainnet",
        // Apechain
        33139 => "apechain-mainnet",
        // Boba
        288 => "boba-mainnet",
        // opBNB
        204 => "opbnb-mainnet",
        // Rootstock
        30 => "rootstock-mainnet",
        // Morph
        2818 => "morph-mainnet",
        _ => return None,
    };
    Some(format!("https://{network}.g.alchemy.com/v2/{api_key}"))
}

/// Encode a simple `function(address) → bool` call as ABI calldata.
/// Uses the function selector (first 4 bytes of keccak256(sig)) + abi-encoded address.
fn encode_call(function_name: &str, address: &str) -> String {
    // Compute selector: keccak256("functionName(address)")[0..4]
    use sha3::{Digest, Keccak256};
    let sig = format!("{function_name}(address)");
    let hash = Keccak256::digest(sig.as_bytes());
    let selector = &hash[..4];

    // ABI-encode address: pad to 32 bytes
    let addr = address.strip_prefix("0x").unwrap_or(address);
    let padded = format!("{:0>64}", addr.to_lowercase());

    format!("0x{}{}", hex::encode(selector), padded)
}

async fn resolve_membership(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ResolveQuery>,
) -> Response {
    let api_key = match &state.alchemy_key {
        Some(key) => key,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                axum::Json(ResolveResponse {
                    member: false,
                    error: Some("no Alchemy API key configured".into()),
                }),
            )
                .into_response();
        }
    };

    let rpc_url = match alchemy_rpc_url(params.chain, api_key) {
        Some(url) => url,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(ResolveResponse {
                    member: false,
                    error: Some(format!("unsupported chain: {}", params.chain)),
                }),
            )
                .into_response();
        }
    };

    let calldata = encode_call(&params.function, &params.address);

    // Build eth_call JSON-RPC request
    let rpc_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_call",
        "params": [{
            "to": params.contract,
            "data": calldata,
        }, "latest"]
    });

    let client = reqwest::Client::new();
    let rpc_response = match client
        .post(&rpc_url)
        .json(&rpc_body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                axum::Json(ResolveResponse {
                    member: false,
                    error: Some(format!("RPC request failed: {e}")),
                }),
            )
                .into_response();
        }
    };

    let rpc_json: serde_json::Value = match rpc_response.json().await {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                axum::Json(ResolveResponse {
                    member: false,
                    error: Some(format!("RPC response parse error: {e}")),
                }),
            )
                .into_response();
        }
    };

    // Parse result as truthy: any non-zero return = member
    // This means balanceOf > 0, isMember == true, hasRole == true all work natively
    let is_member = rpc_json
        .get("result")
        .and_then(|v| v.as_str())
        .map(|hex_str| {
            let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
            // Truthy = any non-zero value (works for bool, uint256, etc.)
            !clean.is_empty() && clean.chars().any(|c| c != '0')
        })
        .unwrap_or(false);

    // Check for RPC error
    if let Some(error) = rpc_json.get("error") {
        return (
            StatusCode::BAD_GATEWAY,
            axum::Json(ResolveResponse {
                member: false,
                error: Some(format!("RPC error: {}", error)),
            }),
        )
            .into_response();
    }

    axum::Json(ResolveResponse {
        member: is_member,
        error: None,
    })
    .into_response()
}

/// Resolve an ENS name to an Ethereum address.
/// If the input looks like an address (0x..., 42 chars), return it as-is.
/// If it looks like an ENS name, resolve it using the ENS Universal Resolver.
/// Cache results for 5 minutes.
pub async fn resolve_ens_name(name: &str) -> Result<String, String> {
    // Check if it's already an address (0x followed by 40 hex chars)
    if name.starts_with("0x") && name.len() == 42 && name[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(name.to_lowercase());
    }

    // Check if it looks like an ENS name (contains a dot and ends with a known TLD)
    if !name.contains('.') || !(name.ends_with(".eth") || name.ends_with(".box") ||
                                name.ends_with(".com") || name.ends_with(".xyz") ||
                                name.ends_with(".org") || name.ends_with(".io") ||
                                name.ends_with(".dev") || name.ends_with(".app")) {
        return Err(format!("Invalid name format: {}", name));
    }

    // Check cache first
    {
        let mut cache = ENS_CACHE.lock().unwrap();
        if let Some((address, timestamp)) = cache.get(name) {
            if timestamp.elapsed().as_secs() < ENS_CACHE_DURATION_SECS {
                return Ok(address.clone());
            } else {
                cache.remove(name);
            }
        }
    }

    // Resolve via ENS Universal Resolver
    let resolved_address = resolve_ens_on_chain(name).await?;

    // Cache the result
    {
        let mut cache = ENS_CACHE.lock().unwrap();
        cache.insert(name.to_string(), (resolved_address.clone(), Instant::now()));
    }

    Ok(resolved_address)
}

/// Resolve ENS name using the Universal Resolver contract on Ethereum mainnet
async fn resolve_ens_on_chain(name: &str) -> Result<String, String> {
    // Get the API key from environment or return an error
    let api_key = std::env::var("ALCHEMY_API_KEY")
        .map_err(|_| "ALCHEMY_API_KEY environment variable not set".to_string())?;

    let rpc_url = format!("https://eth-mainnet.g.alchemy.com/v2/{}", api_key);

    // Encode the ENS name
    let encoded_name = encode_dns_name(name)?;

    // Create the calldata for resolve(bytes name, bytes data)
    // where data is the encoded call to addr(bytes32)
    let addr_selector = &Keccak256::digest(b"addr(bytes32)")[..4];
    let name_hash = namehash(name);

    // ABI encode the addr(bytes32) call
    let addr_calldata = format!("0x{}{:0>64}", hex::encode(addr_selector), hex::encode(name_hash));

    // Encode the resolve function call
    let resolve_selector = &Keccak256::digest(b"resolve(bytes,bytes)")[..4];
    let resolve_calldata = encode_resolve_call(&encoded_name, &addr_calldata)?;
    let full_calldata = format!("0x{}{}", hex::encode(resolve_selector), resolve_calldata);

    // Build eth_call JSON-RPC request
    let rpc_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_call",
        "params": [{
            "to": ENS_UNIVERSAL_RESOLVER,
            "data": full_calldata,
        }, "latest"]
    });

    let client = reqwest::Client::new();
    let rpc_response = client
        .post(&rpc_url)
        .json(&rpc_body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("RPC request failed: {}", e))?;

    let rpc_json: serde_json::Value = rpc_response
        .json()
        .await
        .map_err(|e| format!("RPC response parse error: {}", e))?;

    // Check for RPC error
    if let Some(error) = rpc_json.get("error") {
        return Err(format!("RPC error: {}", error));
    }

    // Parse the result
    let result_hex = rpc_json
        .get("result")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "No result in RPC response".to_string())?;

    // Decode the Universal Resolver response
    // The response is ABI-encoded (bytes), and inside that is the address
    decode_address_from_resolver_response(result_hex)
}

/// Encode a domain name into DNS wire format
fn encode_dns_name(name: &str) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();

    for label in name.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err("Invalid DNS label".to_string());
        }
        result.push(label.len() as u8);
        result.extend_from_slice(label.as_bytes());
    }
    result.push(0); // Root label

    Ok(result)
}

/// Calculate the ENS namehash for a domain name
fn namehash(name: &str) -> [u8; 32] {
    let mut node = [0u8; 32];

    if name.is_empty() {
        return node;
    }

    let labels: Vec<&str> = name.split('.').collect();
    for label in labels.iter().rev() {
        let label_hash = Keccak256::digest(label.as_bytes());
        let mut hasher = Keccak256::new();
        hasher.update(node);
        hasher.update(label_hash);
        node = hasher.finalize().into();
    }

    node
}

/// ABI encode the resolve function call parameters
fn encode_resolve_call(name_bytes: &[u8], data: &str) -> Result<String, String> {
    // ABI encoding for resolve(bytes name, bytes data)
    // Parameters are: offset to name, offset to data, name length + data, data length + data

    let data_bytes = hex::decode(data.strip_prefix("0x").unwrap_or(data))
        .map_err(|_| "Invalid hex data".to_string())?;

    let mut encoded = String::new();

    // Offset to first parameter (name) - always 0x40 (64 bytes) for two parameters
    encoded.push_str(&format!("{:0>64}", "40"));

    // Offset to second parameter (data) - depends on length of name
    let name_padded_len = ((name_bytes.len() + 31) / 32) * 32;
    let data_offset = 64 + 32 + name_padded_len;
    encoded.push_str(&format!("{:0>64x}", data_offset));

    // Encode name: length + padded data
    encoded.push_str(&format!("{:0>64x}", name_bytes.len()));
    let mut name_hex = hex::encode(name_bytes);
    while name_hex.len() % 64 != 0 {
        name_hex.push_str("00");
    }
    encoded.push_str(&name_hex);

    // Encode data: length + padded data
    encoded.push_str(&format!("{:0>64x}", data_bytes.len()));
    let mut data_hex = hex::encode(data_bytes);
    while data_hex.len() % 64 != 0 {
        data_hex.push_str("00");
    }
    encoded.push_str(&data_hex);

    Ok(encoded)
}

/// Decode the address from the Universal Resolver response
fn decode_address_from_resolver_response(result_hex: &str) -> Result<String, String> {
    let data = hex::decode(result_hex.strip_prefix("0x").unwrap_or(result_hex))
        .map_err(|_| "Invalid hex result".to_string())?;

    if data.len() < 64 {
        return Err("Response too short".to_string());
    }

    // Skip the first 32 bytes (offset to data) and next 32 bytes (length)
    // The actual address should be in the next 32 bytes, right-padded
    if data.len() < 96 {
        return Err("Response too short for address".to_string());
    }

    // Extract the address from bytes 76-96 (20 bytes, right-aligned in 32-byte slot)
    let addr_bytes = &data[76..96];

    // Check if all bytes are zero (no address set)
    if addr_bytes.iter().all(|&b| b == 0) {
        return Err("ENS name does not resolve to an address".to_string());
    }

    Ok(format!("0x{}", hex::encode(addr_bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_call_is_member() {
        let calldata = encode_call("isMember", "0xAAA0000000000000000000000000000000000001");
        // Selector for "isMember(address)" = keccak256("isMember(address)")[0..4]
        assert!(calldata.starts_with("0x"));
        assert_eq!(calldata.len(), 2 + 8 + 64); // 0x + 4 bytes selector + 32 bytes address
    }

    #[test]
    fn test_alchemy_url_base() {
        let url = alchemy_rpc_url(8453, "test-key").unwrap();
        assert_eq!(url, "https://base-mainnet.g.alchemy.com/v2/test-key");
    }

    #[test]
    fn test_alchemy_url_unknown_chain() {
        assert!(alchemy_rpc_url(99999, "key").is_none());
    }

    #[test]
    fn test_encode_dns_name() {
        let encoded = encode_dns_name("vitalik.eth").unwrap();
        // Should be: 7 "vitalik" 3 "eth" 0
        assert_eq!(encoded, vec![7, b'v', b'i', b't', b'a', b'l', b'i', b'k', 3, b'e', b't', b'h', 0]);
    }

    #[test]
    fn test_namehash() {
        // Empty string should hash to zero
        let empty_hash = namehash("");
        assert_eq!(empty_hash, [0u8; 32]);

        // Test known namehash for "eth"
        let eth_hash = namehash("eth");
        // This is a known value for "eth" - keccak256(keccak256("eth"))
        assert_eq!(hex::encode(eth_hash), "93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae");
    }

    #[tokio::test]
    async fn test_resolve_ens_name_address_passthrough() {
        // Valid address should pass through
        let result = resolve_ens_name("0x1234567890123456789012345678901234567890").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0x1234567890123456789012345678901234567890");
    }

    #[tokio::test]
    async fn test_resolve_ens_name_invalid_format() {
        // Invalid format should return error
        let result = resolve_ens_name("invalid-name").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid name format"));
    }
}
