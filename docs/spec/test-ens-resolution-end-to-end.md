# Technical Specification: ENS Resolution End-to-End Testing

**Version:** 1.0  
**Author:** PM Agent  
**Date:** 2026-03-22  
**Purpose:** Ensure flawless ENS resolution across all repo.box surfaces for demo readiness

## Overview

This specification defines comprehensive end-to-end testing for ENS name resolution within the repo.box ecosystem. The testing covers three types of names across four key surfaces to ensure seamless user experience and demo-ready quality.

## Scope

### Name Types to Test

1. **Real ENS Names** (e.g., `vitalik.eth`)
   - Public ENS names on Ethereum mainnet
   - Verifiable through standard ENS resolution
   - Must resolve to actual Ethereum addresses

2. **repobox.eth Subdomains** (e.g., `ocean.repobox.eth`)
   - Paid subdomains through repo.box NFT system
   - Custom resolution through subdomain registry
   - Must resolve via repo.box subdomain infrastructure

3. **repo.box Aliases** (e.g., custom aliases)
   - Internal repo.box aliases/shortcuts
   - Managed by repo.box alias system
   - Custom resolution logic

### Test Surfaces

1. **Explorer URL Routing** (`/explore/{name}/`)
2. **AddressDisplay Component Resolution**
3. **Permission Rules** (`.repobox/config.yml`)
4. **Clone URLs** (Git HTTP operations)

## Architecture Analysis

### Current ENS Integration Points

Based on codebase analysis, ENS resolution is implemented at these levels:

#### Core Layer (`repobox-core`)
- **Identity Parsing**: `IdentityKind::Ens` variant in `config.rs`
- **RemoteResolver**: ENS resolution with TTL caching in `resolver.rs`
- **Permission Engine**: ENS identity support in permission rules

#### Server Layer (`repobox-server`) 
- **API Endpoint**: `/api/resolve?name={ens_name}` in `resolve.rs`
- **Universal Resolver**: On-chain ENS resolution via Alchemy
- **Caching**: 5-minute TTL for ENS lookups

#### Web Layer (`web/`)
- **AddressDisplay Component**: ENS name display and resolution
- **AddressResolver**: Forward/reverse ENS resolution with caching
- **Explorer Routes**: Dynamic routing for ENS names

#### Git Layer (Smart HTTP)
- **Clone URLs**: Address resolution for git operations
- **Permission Checks**: ENS-based access control

## Test Plan

### 1. Explorer URL Routing Tests

**Objective**: Verify `/explore/{name}/` routes resolve correctly for all name types.

#### Test Cases

**TC-1.1: Real ENS Name Resolution**
```
Input: /explore/vitalik.eth/
Expected: 
- Resolves to Vitalik's Ethereum address
- Displays repositories owned by that address
- URL remains human-readable (/explore/vitalik.eth/)
- Breadcrumb shows "vitalik.eth"
```

**TC-1.2: repobox.eth Subdomain Resolution**
```
Input: /explore/ocean.repobox.eth/
Expected:
- Resolves via subdomain NFT registry
- Maps to owner's Ethereum address  
- Displays repositories for resolved address
- Shows subdomain in UI, not raw address
```

**TC-1.3: repo.box Alias Resolution**
```
Input: /explore/{alias}/
Expected:
- Resolves via internal alias system
- Maps to target Ethereum address
- Functions identically to direct address access
```

**TC-1.4: Non-existent Name Handling**
```
Input: /explore/nonexistent.eth/
Expected:
- Returns 404 with clear error message
- Does not crash or show raw errors
- Provides link back to explorer
```

**TC-1.5: Invalid Name Format**
```
Input: /explore/invalid-name/
Expected:
- Returns 400 with validation error
- Clear message about expected format
- Does not attempt resolution
```

#### Implementation Requirements

1. **Async Resolution**: Names resolved in `useEffect` without blocking UI
2. **Loading States**: Show spinner while resolving
3. **Error Handling**: Graceful fallback for resolution failures
4. **URL Preservation**: Keep human-readable URLs in browser
5. **Caching**: Respect resolution cache TTL (5 minutes)

### 2. AddressDisplay Component Tests

**Objective**: Verify `<AddressDisplay>` component correctly resolves and displays names.

#### Test Cases

**TC-2.1: Forward Resolution Display**
```
Input: <AddressDisplay address="0x..." />
Expected:
- Shows ENS name if available (e.g., "vitalik.eth")
- Falls back to truncated address if no ENS
- Tooltip shows full address
- Copy functionality works for original address
```

**TC-2.2: Reverse Resolution Verification**
```
Input: <AddressDisplay address="0xd8da6bf26964af9d7eed9e03e53415d37aa96045" />
Expected:
- Resolves to "vitalik.eth" via reverse lookup
- Verifies forward resolution matches (prevents spoofing)
- Caches result for 5 minutes
- Shows loading state during resolution
```

**TC-2.3: repobox.eth Subdomain Display**
```
Input: <AddressDisplay address="0x..." displayName="ocean.repobox.eth" />
Expected:
- Prioritizes provided displayName
- Shows subdomain instead of ENS (if different)
- Indicates verification status
- Links to explorer if linkable=true
```

**TC-2.4: Mixed Display Scenarios**
```
Test all combinations:
- Address with ENS + subdomain
- Address with ENS, no subdomain  
- Address with subdomain, no ENS
- Address with neither (raw display)
```

#### Implementation Requirements

1. **Resolution Priority**: displayName prop > subdomain > ENS > truncated address
2. **Async Resolution**: Non-blocking with loading states
3. **Cache Integration**: Use existing `addressResolver.ts` cache
4. **Error Boundaries**: Graceful fallback on resolution failure
5. **Accessibility**: Proper ARIA labels and screen reader support

### 3. Permission Rules Tests

**Objective**: Verify ENS names work correctly in `.repobox/config.yml` permission rules.

#### Test Cases

**TC-3.1: Direct ENS Permission**
```yaml
# .repobox/config.yml
permissions:
  default: deny
  rules:
    - vitalik.eth push >main
    - alice.eth edit src/**
```
**Expected**: 
- vitalik.eth resolves to address and gets push access
- alice.eth gets edit permissions on src/ files
- Permission engine caches resolved addresses

**TC-3.2: ENS in Groups**
```yaml
groups:
  maintainers:
    - vitalik.eth
    - alice.eth
    - evm:0x1234...

permissions:
  default: deny
  rules:
    - maintainers push >*
```
**Expected**:
- All group members get permissions
- ENS names resolve to addresses for checking
- Mixed EVM/ENS groups work correctly

**TC-3.3: repobox.eth Subdomain Permissions**
```yaml
permissions:
  rules:
    - ocean.repobox.eth admin >*
```
**Expected**:
- Subdomain resolves via NFT registry
- Permissions granted to resolved address
- Works with subdomain ownership changes

**TC-3.4: Resolution Failure Handling**
```yaml
permissions:
  rules:
    - nonexistent.eth push >main
```
**Expected**:
- Permission check fails gracefully
- Does not grant access for unresolved names
- Logs warning but continues operation
```

#### Implementation Requirements

1. **Resolution Caching**: Use `RemoteResolver` 60-second ENS cache
2. **Fail-Safe**: Unresolved names = no permissions granted
3. **Mixed Groups**: Support EVM addresses and ENS names in same group
4. **Async Resolution**: Non-blocking permission checks where possible
5. **Audit Logging**: Log resolution failures for debugging

### 4. Clone URL Tests

**Objective**: Verify git clone operations work with ENS names in URLs.

#### Test Cases

**TC-4.1: Clone with ENS Name**
```bash
git clone https://repo.box/vitalik.eth/my-repo.git
```
**Expected**:
- ENS name resolves to address during clone
- Repository served from resolved address path
- Clone completes successfully
- Remote URL preserved as human-readable

**TC-4.2: Clone with repobox.eth Subdomain**
```bash  
git clone https://repo.box/ocean.repobox.eth/project.git
```
**Expected**:
- Subdomain resolves via NFT registry
- Maps to owner's repository
- Works with subdomain transfers

**TC-4.3: Push to ENS-addressed Repository**
```bash
git remote add origin https://repo.box/vitalik.eth/repo.git
git push origin main
```
**Expected**:
- Push resolves ENS name for permission check
- Works with existing ownership verification
- Subsequent pushes use cached resolution

**TC-4.4: Clone Failure Scenarios**
```bash
# Non-existent ENS name
git clone https://repo.box/missing.eth/repo.git

# ENS resolves but no repository
git clone https://repo.box/vitalik.eth/nonexistent.git
```
**Expected**:
- Clear error messages for each failure type
- HTTP 404 for missing repositories
- HTTP 400 for invalid names

#### Implementation Requirements

1. **Smart HTTP Integration**: ENS resolution in git protocol handlers
2. **Server-side Resolution**: Use existing `/api/resolve` endpoint
3. **Caching**: Respect resolution cache for performance
4. **Error Responses**: Standard HTTP status codes with clear messages
5. **Permission Integration**: Resolved addresses work with access control

## Test Data Requirements

### Real ENS Names for Testing
- `vitalik.eth` (well-known, stable)
- `nick.eth` (ENS founder)
- `ens.eth` (ENS protocol)

### Test repobox.eth Subdomains
- Create test subdomains in development environment
- `test1.repobox.eth`, `test2.repobox.eth` 
- Mock NFT ownership for testing

### Test Repositories
- Create test repositories for each address/name combination
- Include repositories with various permission configurations
- Test both public and private repositories

## Edge Cases & Error Conditions

### Resolution Edge Cases
1. **ENS with no reverse record**: Forward resolves, reverse doesn't
2. **Multiple ENS names**: Same address with multiple names
3. **Expired ENS names**: Names that previously resolved but expired
4. **DNSSEC names**: .com/.org ENS names with DNSSEC
5. **International domains**: Unicode/IDN ENS names

### Network Edge Cases  
1. **ENS resolver timeout**: Network issues during resolution
2. **Alchemy API limits**: Rate limiting or downtime
3. **Cache invalidation**: Stale cache during ownership changes
4. **Concurrent requests**: Multiple resolution requests for same name

### Security Edge Cases
1. **ENS spoofing**: Verify forward/reverse resolution matches
2. **Subdomain hijacking**: Handle subdomain ownership changes
3. **Cache poisoning**: Prevent malicious cache entries
4. **Permission escalation**: Ensure resolved addresses don't bypass security

## Performance Requirements

### Resolution Performance
- **Initial resolution**: <2 seconds for cold cache
- **Cached resolution**: <100ms for warm cache  
- **Batch resolution**: Handle multiple names efficiently
- **Background refresh**: Update cache before expiry

### UI Performance
- **Loading states**: Show within 100ms
- **Progressive enhancement**: Show address first, enhance with name
- **Error recovery**: Graceful degradation on resolution failure
- **Mobile performance**: Optimized for mobile devices

## Implementation Priority

### Phase 1: Core Resolution (P0)
1. Explorer URL routing with ENS names
2. AddressDisplay component enhancement 
3. Basic permission rule support

### Phase 2: Advanced Features (P1)  
1. repobox.eth subdomain integration
2. Git clone URL support
3. Comprehensive error handling

### Phase 3: Optimization (P2)
1. Performance optimization
2. Advanced caching strategies  
3. Edge case handling

## Test Automation

### Unit Tests
- **IdentityKind::Ens**: Parsing and validation
- **RemoteResolver**: ENS resolution and caching
- **AddressDisplay**: Component rendering and resolution
- **Permission Engine**: ENS name permission checks

### Integration Tests
- **API Endpoints**: `/api/resolve` with various inputs
- **Smart HTTP**: Git operations with ENS URLs  
- **Database**: Resolution result persistence
- **Cache**: TTL and invalidation behavior

### End-to-End Tests  
- **User Workflows**: Complete explorer navigation
- **Git Operations**: Clone, push, pull with ENS names
- **Permission Scenarios**: Access control with ENS identities
- **Error Scenarios**: Resolution failures and edge cases

## Acceptance Criteria

### Functional Requirements
✅ **Explorer navigation works with all three name types**  
✅ **AddressDisplay shows resolved names with proper fallbacks**  
✅ **Permission rules support ENS names in all configurations**  
✅ **Git clone/push operations work with ENS URLs**  
✅ **Error handling provides clear, actionable messages**

### Performance Requirements  
✅ **Resolution completes within 2 seconds (cold cache)**
✅ **Cached lookups complete within 100ms**
✅ **UI remains responsive during resolution**
✅ **No blocking operations in critical paths**

### Security Requirements
✅ **Forward/reverse ENS resolution verification**
✅ **Permission checks use resolved addresses correctly**
✅ **Cache entries cannot be poisoned** 
✅ **Unresolved names are denied access safely**

### User Experience Requirements
✅ **Human-readable URLs preserved in browser**
✅ **Loading states for all async operations**  
✅ **Graceful fallback for resolution failures**
✅ **Clear error messages for invalid inputs**

## Monitoring & Debugging

### Logging Requirements
- **Resolution requests**: Track all ENS lookups
- **Cache hits/misses**: Monitor cache effectiveness  
- **Resolution failures**: Log for debugging
- **Performance metrics**: Resolution timing data

### Debugging Tools
- **Resolution test page**: Manual testing interface
- **Cache inspection**: View current cache state
- **Resolution logs**: Detailed failure analysis
- **Performance profiler**: Identify bottlenecks

## Dependencies

### External Services
- **Ethereum mainnet**: ENS resolution via Alchemy
- **Alchemy API**: Rate limits and availability
- **ENS Registry**: Domain ownership and records
- **repobox.eth subdomain NFTs**: Custom registry

### Internal Systems  
- **AddressResolver cache**: 5-minute TTL
- **RemoteResolver cache**: 60-second TTL
- **Database**: Ownership and permission storage
- **Smart HTTP server**: Git protocol handling

## Risk Mitigation

### Service Dependencies
- **Alchemy backup**: Multiple RPC providers for resilience
- **Cache warming**: Pre-populate common names  
- **Graceful degradation**: Fall back to addresses on failure
- **Circuit breaker**: Prevent cascade failures

### Performance Risks
- **Cache sizing**: Monitor memory usage
- **Batch optimization**: Group resolution requests
- **Background refresh**: Update cache proactively  
- **Rate limiting**: Respect external API limits

## Success Metrics

### Technical Metrics
- **Resolution success rate**: >99% for valid names
- **Cache hit rate**: >80% for repeated lookups
- **Response time P95**: <1 second for resolution
- **Error rate**: <1% for well-formed requests

### User Experience Metrics  
- **Navigation success**: Users can find repositories by ENS name
- **Permission accuracy**: ENS-based access control works correctly
- **Git operation success**: Clone/push operations complete successfully
- **Error clarity**: Users understand and can fix resolution errors

---

This specification ensures comprehensive testing coverage for ENS resolution across all repo.box surfaces, providing the foundation for a flawless demo experience and robust production deployment.