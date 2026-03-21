# ENS Names in Permissions - Technical Specification

**Document**: ENS-PERM-001  
**Author**: PM Agent  
**Date**: 2026-03-21  
**Status**: Draft  

## 1. Overview

This specification defines how to enable ENS names (e.g., `vitalik.eth`) directly in `.repobox/config.yml` permission rules and group members, allowing syntax like:

```yaml
permissions:
  rules:
    - "vitalik.eth push >main"
```

### Key Requirements
- ENS names must resolve to addresses at permission **evaluation time**, not config parse time
- If an ENS name changes ownership, permissions follow the name (not the old address)
- Short-lived caching (60s TTL) to balance freshness vs performance
- Integration with existing `/api/resolve` endpoint on repobox-server

## 2. Config File Format Changes

### 2.1 YAML Schema Updates

The existing config format supports these identity formats:
- `evm:0x1234...` (EVM address)

We extend this to support:
- `ens:vitalik.eth` (ENS name with explicit prefix)
- `vitalik.eth` (ENS name with implicit detection)

#### Examples

**Group Members**:
```yaml
groups:
  founders:
    members:
      - evm:0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048
      - ens:vitalik.eth
      - vitalik.eth  # implicit ENS detection
```

**Permission Rules**:
```yaml
permissions:
  rules:
    - "vitalik.eth push >main"
    - "ens:vitalik.eth edit contracts/**"
    - "founders not merge >main"
```

**Mixed Usage**:
```yaml
groups:
  team:
    - evm:0x1234...
    - alice.eth
    - bob.eth
    
permissions:
  rules:
    - team own >*
    - charlie.eth read >*
```

### 2.2 ENS Name Validation Rules

Valid ENS names must:
1. Contain at least one dot (`.`)
2. End with a recognized TLD: `.eth`, `.box`, `.com`, `.xyz`, `.org`, `.io`, `.dev`, `.app`
3. Each label must be ≤63 characters
4. Total length ≤253 characters
5. Only contain valid DNS characters: `a-z`, `0-9`, `-` (no leading/trailing hyphens)

Invalid examples:
- `localhost` (no TLD)
- `name.invalid` (unrecognized TLD)
- `-invalid.eth` (leading hyphen)

## 3. Parser Changes (Rust Codebase)

### 3.1 Identity Type Extension

**File**: `repobox-core/src/config.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IdentityKind {
    Evm,
    Ens,  // New variant
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identity {
    pub kind: IdentityKind,
    pub address: String, // For ENS: stores the name, not resolved address
}

impl Identity {
    pub fn parse(s: &str) -> Result<Self, ConfigError> {
        if let Some(addr) = s.strip_prefix("evm:") {
            if !addr.starts_with("0x") || addr.len() != 42 {
                return Err(ConfigError::InvalidIdentity(s.to_string()));
            }
            Ok(Identity {
                kind: IdentityKind::Evm,
                address: addr.to_string(),
            })
        } else if let Some(name) = s.strip_prefix("ens:") {
            validate_ens_name(name)?;
            Ok(Identity {
                kind: IdentityKind::Ens,
                address: name.to_string(),
            })
        } else if is_ens_name(s) {
            // Implicit ENS detection
            Ok(Identity {
                kind: IdentityKind::Ens,
                address: s.to_string(),
            })
        } else if s.starts_with("0x") && s.len() == 42 {
            // Legacy EVM format without prefix
            Ok(Identity {
                kind: IdentityKind::Evm,
                address: s.to_string(),
            })
        } else {
            Err(ConfigError::InvalidIdentity(s.to_string()))
        }
    }
    
    /// Get the canonical string representation
    pub fn canonical(&self) -> String {
        match self.kind {
            IdentityKind::Evm => format!("evm:{}", self.address),
            IdentityKind::Ens => format!("ens:{}", self.address),
        }
    }
}

fn is_ens_name(s: &str) -> bool {
    s.contains('.') && (
        s.ends_with(".eth") || s.ends_with(".box") || 
        s.ends_with(".com") || s.ends_with(".xyz") || 
        s.ends_with(".org") || s.ends_with(".io") || 
        s.ends_with(".dev") || s.ends_with(".app")
    )
}

fn validate_ens_name(name: &str) -> Result<(), ConfigError> {
    if !is_ens_name(name) {
        return Err(ConfigError::InvalidIdentity(
            format!("invalid ENS name format: {}", name)
        ));
    }
    
    if name.len() > 253 {
        return Err(ConfigError::InvalidIdentity(
            "ENS name too long (max 253 chars)".to_string()
        ));
    }
    
    for label in name.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err(ConfigError::InvalidIdentity(
                "invalid ENS label length".to_string()
            ));
        }
        
        if label.starts_with('-') || label.ends_with('-') {
            return Err(ConfigError::InvalidIdentity(
                "ENS labels cannot start/end with hyphen".to_string()
            ));
        }
        
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ConfigError::InvalidIdentity(
                "invalid characters in ENS name".to_string()
            ));
        }
    }
    
    Ok(())
}
```

### 3.2 Display Implementation Update

```rust
impl std::fmt::Display for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            IdentityKind::Evm => write!(f, "evm:{}", self.address),
            IdentityKind::Ens => write!(f, "ens:{}", self.address),
        }
    }
}
```

### 3.3 Subject Parsing Updates

**File**: `repobox-core/src/parser.rs`

The `parse_subject()` function already handles bare strings as group names and `evm:` prefixes. It needs minor updates to handle ENS names:

```rust
fn parse_subject(s: &str) -> Result<Subject, ConfigError> {
    if s == "*" {
        Ok(Subject::All)
    } else if s.starts_with("evm:") || s.starts_with("ens:") || is_ens_name(s) {
        Ok(Subject::Identity(Identity::parse(s)?))
    } else {
        // Bare word = group name. Strip legacy % prefix if present.
        let name = s.strip_prefix('%').unwrap_or(s);
        Ok(Subject::Group(name.to_string()))
    }
}
```

## 4. ENS Resolution Integration

### 4.1 Server-Side `/api/resolve` Endpoint Extension

**File**: `repobox-server/src/resolve.rs`

The existing endpoint handles on-chain contract calls. We extend it to support ENS resolution:

```rust
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ResolveQuery {
    ContractMembership {
        chain: u64,
        contract: String,
        function: String,
        address: String,
    },
    EnsResolution {
        name: String,
    },
}

async fn resolve_membership(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ResolveQuery>,
) -> Response {
    match params {
        ResolveQuery::ContractMembership { chain, contract, function, address } => {
            // Existing contract membership logic
            resolve_contract_membership(&state, chain, &contract, &function, &address).await
        }
        ResolveQuery::EnsResolution { name } => {
            // New ENS resolution logic
            resolve_ens_to_address(&state, &name).await
        }
    }
}

async fn resolve_ens_to_address(state: &Arc<AppState>, name: &str) -> Response {
    match resolve_ens_name(name).await {
        Ok(address) => {
            axum::Json(EnsResolveResponse {
                name: name.to_string(),
                address,
                error: None,
            }).into_response()
        }
        Err(error) => {
            (StatusCode::BAD_REQUEST, axum::Json(EnsResolveResponse {
                name: name.to_string(),
                address: String::new(),
                error: Some(error),
            })).into_response()
        }
    }
}

#[derive(Debug, Serialize)]
struct EnsResolveResponse {
    name: String,
    address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}
```

### 4.2 Core Library Resolution Interface

**File**: `repobox-core/src/resolver.rs` (new file)

```rust
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
```

## 5. Permission Evaluation Timing and Caching Strategy

### 5.1 Resolution Timing

ENS names are resolved **during permission evaluation**, not at config parse time. This ensures:
1. Current ownership is always checked
2. No stale address data from config parsing
3. Dynamic ownership changes are respected

### 5.2 Cache Strategy

**TTL**: 60 seconds (configurable)
**Scope**: Per-resolver instance (process-local)
**Eviction**: TTL-based expiration
**Fallback**: On resolution failure, deny access (fail-safe)

### 5.3 Engine Integration

**File**: `repobox-core/src/engine.rs`

```rust
pub fn check_with_ens_resolver(
    config: &Config,
    identity: &Identity,
    verb: Verb,
    branch: Option<&str>,
    path: Option<&str>,
    ens_resolver: Option<&mut EnsResolver>,
) -> CheckResult {
    // Build resolved group membership map
    let resolved_groups = if let Some(resolver) = ens_resolver {
        resolve_all_groups_with_ens(&config.groups, resolver).await
    } else {
        resolve_all_groups(&config.groups)  // Static only
    };

    // Rest of permission checking logic unchanged
    check_with_groups(config, identity, verb, branch, path, &resolved_groups)
}

async fn resolve_all_groups_with_ens(
    groups: &HashMap<String, Group>,
    ens_resolver: &mut EnsResolver,
) -> HashMap<String, Vec<Identity>> {
    let mut result = HashMap::new();

    for (group_name, group) in groups {
        let mut resolved_members = Vec::new();
        
        for member in &group.members {
            match member.kind {
                IdentityKind::Evm => resolved_members.push(member.clone()),
                IdentityKind::Ens => {
                    // Resolve ENS name to address
                    match ens_resolver.resolve_identity(member).await {
                        Ok(address) => {
                            resolved_members.push(Identity {
                                kind: IdentityKind::Evm,
                                address,
                            });
                        }
                        Err(e) => {
                            // Log error but don't add to group
                            tracing::warn!(
                                group = group_name,
                                ens_name = &member.address,
                                error = %e,
                                "failed to resolve ENS name in group"
                            );
                        }
                    }
                }
            }
        }
        
        // Handle includes recursively...
        result.insert(group_name.clone(), resolved_members);
    }

    result
}
```

## 6. Error Handling

### 6.1 ENS Resolution Failures

**Scenarios**:
- Network timeout
- ENS name doesn't exist
- ENS name has no address set
- Alchemy API rate limiting

**Behavior**: 
- Log warning with context
- Exclude from group membership
- Fail-safe: deny access for unresolved names

### 6.2 Config Validation Errors

**Invalid ENS name format**:
```
ConfigError::InvalidIdentity("invalid ENS name format: localhost")
```

**ENS resolution timeout during validation**:
```
ConfigError::EnsResolutionFailed("timeout resolving vitalik.eth")
```

### 6.3 Permission Check Failures

When ENS resolution fails during permission evaluation:
1. Log warning with full context
2. Treat as if identity is not in the group
3. Continue with remaining rules
4. Return appropriate deny reason

## 7. Test Cases

### 7.1 Parser Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ens_identity_parsing() {
        let id = Identity::parse("ens:vitalik.eth").unwrap();
        assert_eq!(id.kind, IdentityKind::Ens);
        assert_eq!(id.address, "vitalik.eth");
        
        let id = Identity::parse("vitalik.eth").unwrap();
        assert_eq!(id.kind, IdentityKind::Ens);
    }
    
    #[test]
    fn test_invalid_ens_names() {
        assert!(Identity::parse("localhost").is_err());
        assert!(Identity::parse("invalid.xyz123").is_err());
        assert!(Identity::parse("-invalid.eth").is_err());
    }
    
    #[test]
    fn test_mixed_group_members() {
        let yaml = r#"
groups:
  mixed:
    - evm:0x1234567890123456789012345678901234567890
    - ens:vitalik.eth
    - alice.eth
"#;
        let config = parse(yaml).unwrap();
        assert_eq!(config.groups["mixed"].members.len(), 3);
    }
}
```

### 7.2 Resolution Tests

```rust
#[test]
async fn test_ens_resolution_caching() {
    let mut resolver = EnsResolver::new("https://api.test.com".to_string(), 60);
    
    // Mock successful resolution
    let identity = Identity::parse("ens:test.eth").unwrap();
    let result1 = resolver.resolve_identity(&identity).await;
    let result2 = resolver.resolve_identity(&identity).await; // Should hit cache
    
    assert_eq!(result1, result2);
}

#[test] 
async fn test_ens_resolution_failure() {
    let mut resolver = EnsResolver::new("https://api.test.com".to_string(), 60);
    let identity = Identity::parse("ens:nonexistent.eth").unwrap();
    let result = resolver.resolve_identity(&identity).await;
    assert!(result.is_err());
}
```

### 7.3 Permission Tests

```rust
#[test]
async fn test_ens_permission_evaluation() {
    let yaml = r#"
groups:
  maintainers:
    - ens:alice.eth
permissions:
  rules:
    - maintainers push >main
"#;
    let config = parse(yaml).unwrap();
    
    // Mock resolution: alice.eth → 0xABC...
    let resolved_identity = Identity::parse("evm:0xABC...").unwrap();
    let result = check(&config, &resolved_identity, Verb::Push, Some("main"), None);
    assert!(result.is_allowed());
}
```

### 7.4 Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_ens_permission() {
    // Set up test server with ENS resolution
    // Create config with ENS names
    // Simulate Git operation
    // Verify permission check with resolution
}
```

## 8. Migration Strategy

### 8.1 Backwards Compatibility

**Existing configs remain unchanged**:
- `evm:0x1234...` continues to work
- Legacy bare addresses in tests continue to work
- No breaking changes to existing YAML

**Gradual adoption**:
- Teams can mix EVM addresses and ENS names
- No forced migration required

### 8.2 Configuration Migration Helper

Optional script for teams wanting to convert addresses to ENS names:

```bash
#!/bin/bash
# migrate-to-ens.sh
# Converts known addresses to ENS names in config files

# Example: 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 → vitalik.eth
sed -i 's/evm:0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045/vitalik.eth/g' .repobox/config.yml
```

### 8.3 Rollout Plan

**Phase 1**: Core implementation
- Identity parsing
- ENS resolution API
- Basic permission evaluation

**Phase 2**: Production testing
- Deploy to staging environment
- Test with known ENS names
- Performance monitoring

**Phase 3**: Documentation and rollout
- Update configuration documentation
- Team adoption guidelines
- Real-world testing

## 9. Performance Considerations

### 9.1 Resolution Latency

**Cold resolution**: ~200-500ms (network call to Alchemy)
**Cached resolution**: ~1-5ms (in-memory lookup)
**Cache hit ratio**: Expected >90% for active repositories

### 9.2 Memory Usage

**Per cache entry**: ~100 bytes (name + address + timestamp)
**1000 ENS names**: ~100KB memory footprint
**Cache size limits**: Optional LRU eviction for large deployments

### 9.3 Rate Limiting

**Alchemy limits**: 300 requests/second on paid tier
**Mitigation**: 60s cache TTL reduces API calls by ~99%
**Fallback**: Increase cache TTL during rate limit periods

## 10. Security Considerations

### 10.1 ENS Name Takeover

**Risk**: If an ENS name expires and someone else registers it, they inherit permissions
**Mitigation**: 
- Teams should monitor ENS name expirations
- Consider using longer-lived .eth names
- Regular permission audits

### 10.2 Resolution Spoofing

**Risk**: Compromised resolution endpoint returns wrong addresses
**Mitigation**:
- Use HTTPS for all API calls
- Verify against multiple sources in critical deployments
- Monitor resolution logs for anomalies

### 10.3 Cache Poisoning

**Risk**: Cached wrong address persists for TTL duration
**Mitigation**:
- Short 60s TTL limits exposure window
- Manual cache invalidation capability
- Failed resolution doesn't cache negative results

## 11. Implementation Checklist

### Core Changes
- [ ] Extend `IdentityKind` enum with `Ens` variant
- [ ] Update `Identity::parse()` with ENS support
- [ ] Add ENS name validation functions
- [ ] Update `Display` implementation for ENS identities
- [ ] Modify subject parsing for ENS support

### Resolution Infrastructure  
- [ ] Extend `/api/resolve` endpoint for ENS
- [ ] Implement `EnsResolver` with caching
- [ ] Update permission engine with ENS resolution
- [ ] Add async group resolution support

### Testing
- [ ] Unit tests for ENS parsing
- [ ] Integration tests for resolution
- [ ] Performance tests for caching
- [ ] End-to-end permission tests

### Documentation
- [ ] Update configuration documentation
- [ ] Add ENS usage examples
- [ ] Migration guide for existing teams
- [ ] Troubleshooting guide

## 12. Future Enhancements

### 12.1 Other Name Services

Support for additional decentralized naming systems:
- Unstoppable Domains (.crypto, .nft, .blockchain)
- Handshake (.hns)
- SNS (Solana Name Service)

### 12.2 Advanced Resolution

- Reverse ENS resolution for display
- Avatar/metadata fetching for UI
- Multi-chain ENS support (L2 deployments)

### 12.3 Performance Optimizations

- Batch resolution for multiple names
- Pre-warming cache for known names
- GraphQL API for efficient queries

---

**End of Specification**