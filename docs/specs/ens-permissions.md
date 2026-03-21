# ENS Names in Permissions Specification

**Status**: Draft  
**Author**: PM Agent  
**Date**: 2026-03-21  

## Overview

This specification describes the implementation of ENS (Ethereum Name Service) support in the repo.box permissions system. The feature enables using human-readable ENS names (e.g., `vitalik.eth`) in `.repobox/config.yml` configuration files instead of only hexadecimal Ethereum addresses.

### Key Requirements

1. **Configuration syntax**: Allow ENS names in permissions rules, e.g., `vitalik.eth push >main`
2. **Evaluation-time resolution**: Resolve ENS names when checking permissions, not when parsing config
3. **Ownership follows the name**: If an ENS name changes ownership, permissions automatically follow
4. **Short TTL cache**: Cache resolutions to avoid excessive API calls while maintaining freshness

## Background

### Current State

The permissions system currently supports:
- **EVM addresses**: `evm:0x1234...` or bare `0x1234...`
- **Groups**: Named collections of identities with static membership or remote resolvers
- **Mixed formats**: Different identity types can coexist in the same configuration

### ENS Integration Foundation

The codebase already includes substantial ENS infrastructure:

1. **Identity Types**: `IdentityKind::Ens` enum variant exists in `config.rs`
2. **Parsing Logic**: ENS name detection and validation in `Identity::parse()`
3. **Resolution Infrastructure**: `RemoteResolver` class with ENS resolution capabilities
4. **Server API**: `/api/resolve?name=...` endpoint for ENS-to-address resolution
5. **Caching**: TTL-based caching for both group membership and ENS resolutions

## Architecture

### Core Components

```
┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
│   Config Parser │    │    Engine    │    │ Remote Resolver │
│                 │    │              │    │                 │
│ - ENS detection │────│ - Permission │────│ - ENS resolution│
│ - Validation    │    │   evaluation │    │ - TTL cache     │
│ - AST building  │    │ - Identity   │    │ - Alchemy API   │
└─────────────────┘    │   resolution │    └─────────────────┘
                       └──────────────┘
```

### Resolution Flow

1. **Parse Time**: Configuration parser detects ENS names and creates `Identity { kind: Ens, address: "name.eth" }` objects
2. **Evaluation Time**: Permission engine calls `RemoteResolver::resolve_identity()` to convert ENS names to addresses
3. **Comparison**: Resolved addresses are compared for permission matching
4. **Caching**: Successful resolutions cached with configurable TTL (default: 60 seconds)

### Error Handling Strategy

ENS resolution can fail due to:
- **Network issues**: Alchemy API unreachable
- **Invalid names**: Malformed ENS names  
- **Unregistered names**: Valid format but no address set
- **API limits**: Rate limiting or quota exhaustion

**Policy**: Fail closed - resolution errors result in permission denied to prevent security bypasses.

## Implementation Details

### 1. Configuration Syntax

#### Supported Formats

```yaml
# Explicit ENS prefix
groups:
  maintainers:
    - ens:vitalik.eth
    - ens:alice.eth

# Implicit ENS detection (recommended)
groups:
  maintainers: 
    - vitalik.eth
    - alice.eth

# Mixed with EVM addresses  
groups:
  team:
    - evm:0x1234567890123456789012345678901234567890
    - vitalik.eth
    - alice.eth

# Direct use in permission rules
permissions:
  rules:
    - "vitalik.eth push >main"
    - "alice.eth edit contracts/**"
```

#### ENS Name Validation

**Valid TLDs**: `.eth`, `.box`, `.com`, `.xyz`, `.org`, `.io`, `.dev`, `.app`

**Validation Rules**:
- Maximum total length: 253 characters
- Maximum label length: 63 characters  
- Labels cannot start/end with hyphens
- Only alphanumeric characters and hyphens allowed
- Must contain at least one dot
- Must end with a supported TLD

**Examples**:
```yaml
# ✅ Valid
- vitalik.eth
- my-project.eth  
- subdomain.example.eth
- company.box

# ❌ Invalid  
- localhost          # No TLD
- invalid.xyz123     # Unsupported TLD
- -invalid.eth       # Leading hyphen
- toolongname.eth    # Label > 63 chars
```

### 2. Resolution Strategy

#### Resolution Order

For identity comparison in permission checking:

1. **Direct address match**: If both identities are EVM addresses, compare directly
2. **ENS resolution**: If either identity is ENS, resolve to address and compare
3. **Caching**: Use cached result if available and not expired
4. **Fallback**: On resolution error, deny permission (fail closed)

#### Cache Configuration

```rust
// Default ENS cache TTL
pub const DEFAULT_ENS_CACHE_TTL: u64 = 60; // seconds

// Cache key format
let cache_key = format!("ens:{}", name);
```

#### API Endpoint

**Existing Implementation**: `/api/resolve?name={ens_name}`

**Response Format**:
```json
{
  "name": "vitalik.eth",
  "address": "0xd8da6bf26964af9d7eed9e03e53415d37aa96045",
  "error": null
}
```

**Error Response**:
```json
{
  "name": "invalid.eth", 
  "address": "",
  "error": "ENS name does not resolve to an address"
}
```

### 3. Permission Engine Integration

#### Identity Resolution in Engine

```rust
// In engine.rs - subject_matches_with_resolver()
match subject {
    Subject::Identity(subject_identity) => {
        // Handle ENS resolution for identity subjects
        if let Some(resolver) = resolver {
            if let Ok(resolved_subject_addr) = resolver.resolve_identity(subject_identity) {
                if let Ok(resolved_target_addr) = resolver.resolve_identity(identity) {
                    return resolved_subject_addr == resolved_target_addr;
                }
            }
        }
        // Fall back to static check
        subject_identity == identity
    }
    // ... other cases
}
```

#### Group Membership with ENS

ENS identities can be members of groups, and group membership checks will resolve ENS names:

```yaml
groups:
  maintainers:
    - vitalik.eth  # Will be resolved at evaluation time
    - alice.eth

permissions:
  rules:
    - "maintainers push >main"
```

When checking if `bob.eth` can push to main:
1. Resolve `bob.eth` → `0xBobAddr`
2. Resolve each group member: `vitalik.eth` → `0xVitalikAddr`, `alice.eth` → `0xAliceAddr`  
3. Check if `0xBobAddr` matches any resolved group member

### 4. Error Handling

#### Resolution Errors

**Types of Errors**:
- `NetworkError`: Cannot reach ENS API
- `InvalidName`: Malformed ENS name format
- `NotFound`: ENS name not registered or no address set
- `RateLimit`: API quota exceeded

**Error Responses**:
```rust
impl RemoteResolver {
    pub fn resolve_identity(&self, identity: &Identity) -> Result<String, String> {
        match identity.kind {
            IdentityKind::Evm => Ok(identity.address.clone()),
            IdentityKind::Ens => self.resolve_ens_name(&identity.address),
        }
    }
}
```

#### Failure Modes

**Parsing Errors** (Configuration loading):
- Invalid ENS name format → Configuration parse error
- Malformed YAML → Serde error

**Runtime Errors** (Permission evaluation): 
- ENS resolution failure → Permission denied
- Network timeout → Permission denied
- API quota exceeded → Permission denied

#### Error Messages

Clear, actionable error messages for different scenarios:

```
❌ Invalid ENS name format: 'localhost' (must end with .eth, .box, etc.)
❌ ENS resolution failed for 'vitalik.eth': name does not resolve to an address  
❌ Network error resolving 'alice.eth': API timeout
❌ ENS API quota exceeded - try again later
```

### 5. Testing Strategy

#### Unit Tests

**Configuration Parsing**:
- Valid ENS name formats accepted
- Invalid ENS names rejected with clear errors
- Mixed EVM/ENS groups parsed correctly
- ENS names in permission rules parsed correctly

**Identity Resolution**:  
- EVM addresses passed through unchanged
- ENS names resolved via API call
- Resolution results cached with TTL
- Cache expiration handled correctly

**Permission Engine**:
- ENS identities matched against permission rules
- Group membership works with ENS identities  
- Resolution errors result in permission denial
- Mixed EVM/ENS configurations work correctly

#### Integration Tests

**End-to-End Permission Checks**:
- Configure permissions with ENS names
- Test actual resolution via Alchemy API
- Verify caching behavior
- Test failure scenarios (network errors, invalid names)

**Configuration Validation**:
- Real `.repobox/config.yml` files with ENS names
- Validation of edge cases and error handling
- Performance testing with large numbers of ENS identities

#### Test Configuration Examples

```yaml
# test-config-ens.yml
groups:
  maintainers:
    - vitalik.eth
    - alice.eth  
    - evm:0x1234567890123456789012345678901234567890

permissions:
  default: deny
  rules:
    - "maintainers own >*"
    - "vitalik.eth force-push >main" 
    - "alice.eth not edit .repobox/config.yml"
```

### 6. Security Considerations

#### Attack Vectors

**ENS Name Takeover**:
- **Risk**: If an ENS name expires and is re-registered by malicious actor
- **Mitigation**: Short TTL cache limits exposure window to ~60 seconds
- **Recommendation**: Monitor critical ENS names for ownership changes

**DNS Poisoning**: 
- **Risk**: Compromised DNS could redirect ENS API calls
- **Mitigation**: Use HTTPS for all API calls, verify SSL certificates
- **Current**: Alchemy API uses HTTPS with certificate pinning

**API Manipulation**:
- **Risk**: Compromised ENS resolution API returning wrong addresses
- **Mitigation**: Use multiple resolution sources, implement response validation
- **Future**: Support backup resolution providers

**DoS via Resolution**:
- **Risk**: Malicious configs with many ENS names causing excessive API calls
- **Mitigation**: Cache with reasonable TTL, rate limiting, fail fast on errors

#### Access Control

**API Key Management**:
- Alchemy API key stored in environment variables
- No hardcoded credentials in configuration files
- Key rotation procedures documented

**Cache Security**:
- In-memory cache only (no persistent storage)
- Cache entries tied to specific ENS names
- TTL prevents stale data attacks

#### Audit Trail

**Resolution Logging**:
- Log all ENS resolution attempts (success/failure)
- Include timestamp, name, resolved address, error details
- Monitor for unusual resolution patterns

### 7. Performance Considerations

#### Caching Strategy

**Cache Hit Optimization**:
- Default 60-second TTL balances freshness vs performance
- In-memory HashMap with Instant-based expiration
- LRU eviction prevents unbounded memory growth

**API Call Reduction**:
- Batch resolution of multiple ENS names in same permission check
- Parallel resolution for independent names
- Circuit breaker pattern for failing resolution endpoints

#### Scalability

**Large Group Support**:
- Groups with many ENS names resolved efficiently
- Parallel resolution using async/await patterns
- Early termination on first match for authorization

**High-Traffic Scenarios**:
- Cache shared across all permission checks
- Connection pooling for API requests  
- Configurable cache size limits

### 8. Monitoring and Observability

#### Metrics

**Resolution Metrics**:
- ENS resolution request rate
- Success/failure rates by error type  
- Average resolution latency
- Cache hit/miss ratio

**Error Tracking**:
- Failed resolution attempts by name
- Network timeout frequency
- API quota exhaustion events
- Invalid name format attempts

#### Logging

**Debug Information**:
```
INFO: Resolving ENS name 'vitalik.eth'
DEBUG: Cache miss for 'vitalik.eth', calling API
DEBUG: API response: {"address": "0xd8da...", "error": null}  
INFO: Cached 'vitalik.eth' -> '0xd8da...' for 60s
```

**Error Logging**:
```
WARN: ENS resolution failed for 'alice.eth': name does not resolve
ERROR: ENS API timeout for 'vitalik.eth' after 10s
ERROR: Invalid ENS name format: 'localhost'
```

## Implementation Steps

### Phase 1: Core Infrastructure ✅ (Already Complete)

- [x] ENS identity types in `config.rs`
- [x] ENS name parsing and validation  
- [x] RemoteResolver with ENS resolution
- [x] Server API endpoint for ENS resolution
- [x] TTL-based caching system

### Phase 2: Permission Engine Integration ✅ (Already Complete)

- [x] Identity resolution in permission engine
- [x] ENS support in group membership checks
- [x] Error handling for resolution failures
- [x] Test coverage for core functionality

### Phase 3: Validation and Testing

- [ ] **Comprehensive test suite** for edge cases
- [ ] **Performance testing** with large ENS configurations
- [ ] **Error handling validation** for all failure modes
- [ ] **Integration testing** with real ENS names

### Phase 4: Documentation and Deployment

- [ ] **User documentation** for ENS configuration
- [ ] **Migration guide** for existing configurations
- [ ] **Monitoring setup** for ENS resolution metrics
- [ ] **Security audit** of ENS-related code paths

### Phase 5: Advanced Features (Future)

- [ ] **Multiple resolution providers** for redundancy
- [ ] **ENS name validation** at configuration time
- [ ] **ENS ownership monitoring** and alerts
- [ ] **Bulk resolution optimization** for large groups

## Usage Examples

### Basic ENS Configuration

```yaml
# .repobox/config.yml
groups:
  founders:
    - vitalik.eth
    - alice.eth
  
  maintainers:
    - bob.eth
    - charlie.eth
    - founders  # Include other groups

permissions:
  default: deny
  rules:
    # Direct ENS usage in rules
    - "vitalik.eth own >*"
    
    # Group-based permissions
    - "maintainers push >develop"
    - "maintainers branch >feature/**"
    
    # Mixed EVM and ENS
    - "alice.eth edit docs/**"
    - "evm:0x1234567890123456789012345678901234567890 read >*"
```

### Advanced Configuration

```yaml
groups:
  # Static group with ENS names
  core-team:
    - vitalik.eth
    - alice.eth
    
  # Remote resolver group (existing feature)
  token-holders:
    resolver: onchain
    chain: 1
    contract: "0x1234567890123456789012345678901234567890"
    function: "balanceOf"
    cache_ttl: 300
    
  # Hybrid: static + remote + includes
  all-contributors:
    - bob.eth  # Static ENS member
    - core-team  # Include static group
    - token-holders  # Include remote group

permissions:
  default: deny
  rules:
    # Founders have complete control
    - "vitalik.eth own >*"
    
    # Core team can manage development branches
    - "core-team push >develop"
    - "core-team merge >develop"
    - "core-team branch >feature/**"
    
    # Token holders can contribute to features
    - "token-holders push >feature/**"
    - "token-holders not edit .repobox/config.yml"
    
    # All contributors can read
    - "all-contributors read >*"
```

### Migration Example

**Before** (EVM addresses only):
```yaml
groups:
  founders:
    - evm:0xd8da6bf26964af9d7eed9e03e53415d37aa96045
    - evm:0x123456789abcdef123456789abcdef1234567890

permissions:
  rules:
    - "founders own >*"
```

**After** (ENS names):
```yaml
groups:
  founders:
    - vitalik.eth    # Resolves to 0xd8da6bf...
    - alice.eth      # Resolves to 0x12345678...

permissions:
  rules:
    - "founders own >*"
```

## Security Checklist

### Configuration Security

- [ ] ENS name validation prevents malformed inputs
- [ ] Invalid TLDs rejected at parse time
- [ ] Configuration parse errors clearly identify ENS issues
- [ ] No hardcoded ENS names in sensitive configurations

### Runtime Security  

- [ ] Resolution failures result in permission denial (fail closed)
- [ ] Network timeouts handled gracefully  
- [ ] API errors logged but don't crash the system
- [ ] Cache poisoning prevented by TTL expiration

### API Security

- [ ] Alchemy API key stored securely (environment variables)
- [ ] HTTPS used for all ENS resolution requests
- [ ] Request timeout prevents indefinite blocking
- [ ] Rate limiting respected to avoid quota exhaustion

### Access Control

- [ ] ENS resolution does not bypass existing permission checks
- [ ] Group membership correctly includes ENS identities
- [ ] Mixed EVM/ENS configurations work as expected
- [ ] Permission denial on ENS resolution errors

## Conclusion

The ENS names in permissions feature significantly enhances the usability of the repo.box permission system by allowing human-readable names instead of hex addresses. The implementation leverages existing infrastructure while maintaining security and performance through intelligent caching and error handling.

The feature is largely implemented in the current codebase, with the core parsing, resolution, and caching mechanisms already in place. The remaining work focuses on comprehensive testing, documentation, and validation of edge cases to ensure robust production deployment.

**Key Benefits**:
- **Usability**: Human-readable names instead of hex addresses
- **Flexibility**: Mixed EVM/ENS configurations supported
- **Security**: Fail-closed error handling prevents security bypasses  
- **Performance**: TTL caching minimizes API calls
- **Future-proof**: Foundation for advanced ENS features

**Next Steps**:
1. Comprehensive testing of edge cases and error conditions
2. Performance validation with large ENS configurations  
3. Documentation and migration guides for users
4. Production monitoring and alerting setup