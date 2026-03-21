//! Group resolver — resolves remote group membership via HTTP or on-chain proxy.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::config::{Group, GroupResolver, Identity, IdentityKind};

/// Cache entry for resolved membership.
#[derive(Debug, Clone)]
struct CacheEntry {
    is_member: bool,
    expires_at: Instant,
}

/// Cache entry for ENS resolution.
#[derive(Debug, Clone)]
struct EnsEntry {
    address: String,
    expires_at: Instant,
}

/// Resolver with in-memory TTL cache.
pub struct RemoteResolver {
    cache: Mutex<HashMap<String, CacheEntry>>,
    ens_cache: Mutex<HashMap<String, EnsEntry>>,
    /// Base URL for on-chain proxy (e.g. "https://repo.box/api")
    pub onchain_proxy_url: String,
    /// Default ENS cache TTL in seconds
    pub ens_cache_ttl: u64,
}

impl RemoteResolver {
    pub fn new(onchain_proxy_url: &str) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            ens_cache: Mutex::new(HashMap::new()),
            onchain_proxy_url: onchain_proxy_url.to_string(),
            ens_cache_ttl: 60, // Default 60 seconds
        }
    }

    pub fn with_ens_cache_ttl(onchain_proxy_url: &str, ens_cache_ttl: u64) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            ens_cache: Mutex::new(HashMap::new()),
            onchain_proxy_url: onchain_proxy_url.to_string(),
            ens_cache_ttl,
        }
    }

    /// Check if an identity is a member of a resolved group.
    /// Returns None if the group has no resolver (static-only).
    /// Returns Some(bool) if resolved (from cache or remote).
    pub fn check_membership(
        &self,
        group: &Group,
        identity: &Identity,
    ) -> Option<bool> {
        let resolver = group.resolver.as_ref()?;

        let cache_key = format!("{}:{}", group.name, identity.address);
        let cache_ttl = match resolver {
            GroupResolver::Http { cache_ttl, .. } => *cache_ttl,
            GroupResolver::Onchain { cache_ttl, .. } => *cache_ttl,
        };

        // Check cache first
        if cache_ttl > 0 {
            if let Ok(cache) = self.cache.lock() {
                if let Some(entry) = cache.get(&cache_key) {
                    if entry.expires_at > Instant::now() {
                        return Some(entry.is_member);
                    }
                }
            }
        }

        // Resolve remotely
        let result = match resolver {
            GroupResolver::Http { url, .. } => self.resolve_http(url, identity),
            GroupResolver::Onchain { chain, contract, function, .. } => {
                self.resolve_onchain(*chain, contract, function, identity)
            }
        };

        // Cache the result
        if let Ok(is_member) = result {
            if cache_ttl > 0 {
                if let Ok(mut cache) = self.cache.lock() {
                    cache.insert(cache_key, CacheEntry {
                        is_member,
                        expires_at: Instant::now() + Duration::from_secs(cache_ttl),
                    });
                }
            }
            Some(is_member)
        } else {
            // Fail closed — resolver error = not a member
            Some(false)
        }
    }

    /// HTTP resolver: GET <url>/members/<address> → { "member": true/false }
    fn resolve_http(&self, url: &str, identity: &Identity) -> Result<bool, String> {
        let check_url = format!("{}/members/{}", url.trim_end_matches('/'), identity.address);
        self.fetch_membership(&check_url, 5)
    }

    /// On-chain resolver via server proxy:
    /// GET <proxy>/resolve?chain=<chain>&contract=<contract>&function=<function>&address=<address>
    fn resolve_onchain(
        &self,
        chain: u64,
        contract: &str,
        function: &str,
        identity: &Identity,
    ) -> Result<bool, String> {
        let url = format!(
            "{}/resolve?chain={}&contract={}&function={}&address={}",
            self.onchain_proxy_url.trim_end_matches('/'),
            chain, contract, function, identity.address
        );
        self.fetch_membership(&url, 10)
    }

    /// Fetch membership from a URL that returns { "member": true/false }
    fn fetch_membership(&self, url: &str, timeout_secs: u64) -> Result<bool, String> {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .build();

        let body = agent.get(url)
            .call()
            .map_err(|e| format!("resolver error: {e}"))?
            .into_string()
            .map_err(|e| format!("resolver read error: {e}"))?;

        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("resolver parse error: {e}"))?;

        Ok(json.get("member").and_then(|v| v.as_bool()).unwrap_or(false))
    }

    /// Clear the entire cache.
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
        if let Ok(mut ens_cache) = self.ens_cache.lock() {
            ens_cache.clear();
        }
    }

    /// Resolve an identity to its canonical address form.
    /// For EVM addresses, returns as-is. For ENS names, resolves via API with caching.
    pub fn resolve_identity(&self, identity: &Identity) -> Result<String, String> {
        match identity.kind {
            IdentityKind::Evm => Ok(identity.address.clone()),
            IdentityKind::Ens => self.resolve_ens_name(&identity.address),
        }
    }

    /// Resolve an ENS name to an address with TTL-based caching.
    fn resolve_ens_name(&self, name: &str) -> Result<String, String> {
        let now = Instant::now();
        
        // Check cache first
        if let Ok(cache) = self.ens_cache.lock() {
            if let Some(entry) = cache.get(name) {
                if now < entry.expires_at {
                    return Ok(entry.address.clone());
                }
            }
        }
        
        // Make API request
        let url = format!(
            "{}/resolve?name={}", 
            self.onchain_proxy_url.trim_end_matches('/'), 
            urlencoding::encode(name)
        );
        
        let result = self.fetch_ens_resolution(&url, name, 10)?;
        
        // Cache the result
        if let Ok(mut cache) = self.ens_cache.lock() {
            cache.insert(name.to_string(), EnsEntry {
                address: result.clone(),
                expires_at: now + Duration::from_secs(self.ens_cache_ttl),
            });
        }
        
        Ok(result)
    }

    /// Fetch ENS resolution from the API endpoint
    fn fetch_ens_resolution(&self, url: &str, name: &str, timeout_secs: u64) -> Result<String, String> {
        let response_text = match ureq::get(url)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .call() {
            Ok(response) => match response.into_string() {
                Ok(text) => text,
                Err(_) => return Err("failed to read response body".to_string()),
            },
            Err(ureq::Error::Status(code, _)) => {
                return Err(format!("HTTP {} error resolving ENS name '{}'", code, name));
            }
            Err(_) => {
                return Err(format!("network error resolving ENS name '{}'", name));
            }
        };

        // Parse JSON response
        match serde_json::from_str::<serde_json::Value>(&response_text) {
            Ok(json) => {
                if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
                    return Err(error.to_string());
                }
                
                if let Some(address) = json.get("address").and_then(|a| a.as_str()) {
                    Ok(address.to_string())
                } else {
                    Err("missing address field in response".to_string())
                }
            }
            Err(_) => Err("invalid JSON response".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_group_returns_none() {
        let resolver = RemoteResolver::new("https://repo.box/api");
        let group = Group {
            name: "founders".to_string(),
            members: vec![Identity::parse("evm:0xAAA0000000000000000000000000000000000001").unwrap()],
            includes: vec![],
            resolver: None,
        };
        let id = Identity::parse("evm:0xAAA0000000000000000000000000000000000001").unwrap();
        assert!(resolver.check_membership(&group, &id).is_none());
    }
}
