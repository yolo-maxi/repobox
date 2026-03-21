use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::config::{Identity, IdentityKind};

/// Remote resolution cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    address: String,
    timestamp: Instant,
}

/// ENS name resolver with TTL-based caching
#[derive(Debug)]
pub struct EnsResolver {
    cache: HashMap<String, CacheEntry>,
    cache_ttl: Duration,
    base_url: String,
}

impl EnsResolver {
    pub fn new(base_url: String, cache_ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(cache_ttl_seconds),
            base_url,
        }
    }
    
    /// Resolve an identity to its canonical address form
    pub async fn resolve_identity(&mut self, identity: &Identity) -> Result<String, String> {
        match identity.kind {
            IdentityKind::Evm => Ok(identity.address.clone()),
            IdentityKind::Ens => self.resolve_ens_name(&identity.address).await,
        }
    }
    
    async fn resolve_ens_name(&mut self, name: &str) -> Result<String, String> {
        // Check cache first
        if let Some(entry) = self.cache.get(name) {
            if entry.timestamp.elapsed() < self.cache_ttl {
                return Ok(entry.address.clone());
            }
            // Expired, remove from cache
            self.cache.remove(name);
        }
        
        // Make API request
        let url = format!("{}/api/resolve?name={}", self.base_url, urlencoding::encode(name));
        let client = reqwest::Client::new();
        
        let response = client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("ENS resolution request failed: {}", e))?;
            
        let result: EnsResolveResponse = response
            .json()
            .await
            .map_err(|e| format!("ENS resolution parse error: {}", e))?;
            
        if let Some(error) = result.error {
            return Err(error);
        }
        
        // Cache the result
        self.cache.insert(name.to_string(), CacheEntry {
            address: result.address.clone(),
            timestamp: Instant::now(),
        });
        
        Ok(result.address)
    }
}

#[derive(serde::Deserialize)]
struct EnsResolveResponse {
    name: String,
    address: String,
    error: Option<String>,
}