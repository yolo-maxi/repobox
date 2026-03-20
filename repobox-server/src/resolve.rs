//! On-chain group resolver proxy.
//!
//! GET /api/resolve?chain=8453&contract=0x...&function=isMember&address=0x...
//! → { "member": true/false }
//!
//! Proxies eth_call to Alchemy RPC for the specified chain.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};

use crate::AppState;

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

/// Alchemy chain ID → subdomain mapping
fn alchemy_rpc_url(chain: u64, api_key: &str) -> Option<String> {
    let network = match chain {
        1 => "eth-mainnet",
        5 => "eth-goerli",
        11155111 => "eth-sepolia",
        10 => "opt-mainnet",
        137 => "polygon-mainnet",
        42161 => "arb-mainnet",
        8453 => "base-mainnet",
        84532 => "base-sepolia",
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

    // Parse result: "0x...0001" = true, "0x...0000" = false
    let is_member = rpc_json
        .get("result")
        .and_then(|v| v.as_str())
        .map(|hex_str| {
            // The result is a 32-byte ABI-encoded bool
            // True = ...0001, False = ...0000
            let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
            clean.ends_with('1') && !clean.ends_with("00")
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
}
