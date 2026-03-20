//! Group resolver — resolves remote group membership via HTTP or on-chain proxy.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::config::{Group, GroupResolver, Identity};

/// Cache entry for resolved membership.
#[derive(Debug, Clone)]
struct CacheEntry {
    is_member: bool,
    expires_at: Instant,
}

/// Resolver with in-memory TTL cache.
pub struct RemoteResolver {
    cache: Mutex<HashMap<String, CacheEntry>>,
    /// Base URL for on-chain proxy (e.g. "https://repo.box/api")
    pub onchain_proxy_url: String,
}

impl RemoteResolver {
    pub fn new(onchain_proxy_url: &str) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            onchain_proxy_url: onchain_proxy_url.to_string(),
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
