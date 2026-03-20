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
