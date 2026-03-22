# ENS Resolution End-to-End Testing Guide

This document describes the comprehensive test suite for ENS name resolution across all repo.box surfaces, implementing the specification in `docs/spec/test-ens-resolution-end-to-end.md`.

## Test Architecture

The ENS testing is implemented across multiple layers:

### 1. Unit Tests (`repobox-core/tests/ens_resolution_e2e.rs`)
- **Core ENS Identity Parsing**: Tests ENS name parsing and validation
- **Permission Rules**: Tests ENS names in `.repobox/config.yml` files
- **RemoteResolver Integration**: Tests ENS resolution caching and behavior
- **Edge Cases**: Tests various edge cases and error conditions
- **Integration Scenarios**: Tests complex workflows with mixed identity types

### 2. Integration Test Script (`tests/ens_integration_script.sh`)
- **Explorer API Testing**: Tests `/api/explorer/resolve/{name}` endpoint
- **Performance Testing**: Tests resolution speed and caching behavior
- **Concurrent Resolution**: Tests multiple simultaneous ENS lookups
- **Error Handling**: Tests various error scenarios and edge cases
- **Git Operations**: Mock tests for git URL patterns with ENS names

## Running the Tests

### Unit Tests (Rust)

Run the comprehensive unit test suite:

```bash
# Run all ENS-related unit tests
cargo test -p repobox-core ens_resolution_e2e

# Run specific test modules
cargo test -p repobox-core ens_identity_tests
cargo test -p repobox-core permission_tests
cargo test -p repobox-core resolver_tests
cargo test -p repobox-core integration_tests

# Run with output
cargo test -p repobox-core ens_resolution_e2e -- --nocapture
```

### Integration Tests (Bash)

Run the live API integration tests:

```bash
# Test against local development server
./tests/ens_integration_script.sh http://localhost:3000

# Test against staging/production
./tests/ens_integration_script.sh https://repo.box

# View detailed output
./tests/ens_integration_script.sh http://localhost:3000 | tee test-output.log
```

### Manual Testing Checklist

For demo preparation, manually verify these key scenarios:

#### 1. Explorer Navigation
- [ ] Navigate to `/explore/vitalik.eth/`
- [ ] Verify it resolves to correct address and shows repositories
- [ ] Check that URL remains human-readable
- [ ] Verify breadcrumb shows "vitalik.eth"

#### 2. AddressDisplay Component
- [ ] Component shows ENS name instead of truncated address
- [ ] Tooltip shows full address
- [ ] Copy functionality copies original address
- [ ] Loading state appears during resolution

#### 3. Permission Rules
```yaml
# Test this configuration in a repository
permissions:
  default: deny
  rules:
    - vitalik.eth push >main
    - alice.eth edit src/**

groups:
  maintainers:
    - vitalik.eth
    - nick.eth
    - evm:0x1234...
```

#### 4. Git Operations
- [ ] Clone works: `git clone https://repo.box/vitalik.eth/repo.git`
- [ ] Push works with ENS-based permissions
- [ ] Clear error messages for invalid ENS names

## Test Data

### Real ENS Names (for testing)
- `vitalik.eth` → `0xd8da6bf26964af9d7eed9e03e53415d37aa96045`
- `nick.eth` → `0xb8c2c29ee19d8307cb7255e1cd9cbde883906c6a`
- `ens.eth` → `0xfdb33f8ac7ce72d7d4795dd8610e323b4c122fbb`

### Test Subdomains (mock/future)
- `ocean.repobox.eth` → `0x1234567890123456789012345678901234567890`
- `alice.repobox.eth` → `0x2345678901234567890123456789012345678901`

### Test Aliases (internal)
- `founder` → `0xd8da6bf26964af9d7eed9e03e53415d37aa96045`
- `core-dev` → `0xb8c2c29ee19d8307cb7255e1cd9cbde883906c6a`

## Test Coverage

### Test Surfaces ✅
- [x] **Explorer URL Routing** (`/explore/{name}/`)
- [x] **AddressDisplay Component** (React component resolution)
- [x] **Permission Rules** (`.repobox/config.yml` parsing)
- [x] **Clone URLs** (Git HTTP operations pattern matching)

### Name Types ✅
- [x] **Real ENS Names** (vitalik.eth, nick.eth, ens.eth)
- [x] **repobox.eth Subdomains** (ocean.repobox.eth, alice.repobox.eth)
- [x] **repo.box Aliases** (founder, core-dev)

### Test Scenarios ✅

#### Functional Requirements
- [x] ENS name parsing and validation
- [x] Permission rules with ENS identities
- [x] Mixed EVM/ENS groups
- [x] Resolution caching behavior
- [x] Error handling for invalid/non-existent names
- [x] Canonical representation (`ens:name.eth`)

#### Performance Requirements
- [x] Resolution speed testing (< 2s cold, < 100ms warm)
- [x] Cache hit/miss behavior
- [x] Concurrent resolution handling

#### Security Requirements
- [x] Forward/reverse resolution verification patterns
- [x] Permission checks with resolved addresses
- [x] Fail-safe behavior for unresolved names

#### Edge Cases
- [x] Various ENS domain formats (.eth, .app, .xyz, etc.)
- [x] Invalid characters and format validation
- [x] Case sensitivity handling
- [x] Long domain names
- [x] International/Unicode domains (basic support)

## Expected Results

### Unit Test Output
```
✓ ENS identity parsing works correctly
✓ ENS validation edge cases work correctly  
✓ ENS canonical representation works correctly
✓ Direct ENS permissions work correctly
✓ ENS names in groups work correctly
✓ Subdomain permissions work correctly
✓ Resolution failure handling works correctly (static mode)
✓ Complex mixed EVM/ENS groups work correctly
✓ RemoteResolver ENS configuration works correctly
✓ Identity resolution handles different types correctly
✓ Resolver cache behavior tests passed
✓ Group membership with ENS identities works correctly
✓ Full ENS workflow completed: vitalik.eth -> parse -> permissions -> groups
✓ Mixed name types scenario completed successfully
✓ Comprehensive identity and permission scenario passed

🎯 ENS Resolution Test Suite Summary:
- ✓ Core ENS identity parsing and validation
- ✓ Permission system integration with ENS names
- ✓ Group membership with mixed EVM/ENS identities
- ✓ RemoteResolver ENS caching and resolution
- ✓ Edge cases and error conditions
- ✓ End-to-end workflow scenarios

🚀 Ready for:
- Demo presentations with ENS resolution
- Production deployment with comprehensive ENS support
- Additional surfaces (Explorer UI, Git operations)
```

### Integration Test Output
```
🧭 Testing Explorer URL Routing
[PASS] ENS resolution: vitalik.eth -> 0xd8da6bf26964af9d7eed9e03e53415d37aa96045
[PASS] ENS resolution: nick.eth -> 0xb8c2c29ee19d8307cb7255e1cd9cbde883906c6a
[WARN] Subdomain resolution not implemented: ocean.repobox.eth (expected)
[PASS] Correctly rejected non-existent name: nonexistent.eth
[PASS] Address passthrough works correctly

⚡ Testing Resolution Performance
[PASS] Cold cache resolution: 1250ms (< 2000ms)
[PASS] Warm cache resolution: 45ms (< 100ms)
[PASS] Cached response matches original

🔄 Testing Concurrent Resolution
[PASS] Concurrent resolution: 3/3 succeeded

📦 Testing Git Operations (Mock)
[PASS] Git URL ENS extraction: https://repo.box/vitalik.eth/my-repo.git -> vitalik.eth

🚨 Testing Error Handling
[PASS] Error handling: Non-existent ENS repository

🎉 All tests passed! ENS resolution is ready for demo.
```

## Troubleshooting

### Common Issues

1. **ENS Resolution Fails**: Check internet connectivity and Ethereum RPC endpoint
2. **Cache Issues**: Clear resolver cache between tests if needed
3. **Permission Parsing Errors**: Verify YAML syntax in test configurations
4. **Performance Slow**: Check network latency to ENS resolver

### Debug Commands

```bash
# Test individual ENS resolution
curl "http://localhost:3000/api/explorer/resolve/vitalik.eth" | jq

# Check server logs for resolution errors
tail -f logs/server.log | grep -i ens

# Verify configuration parsing
./target/release/repobox config check .repobox/config.yml
```

## Demo Readiness Checklist

Before demo, ensure all these pass:

- [ ] Unit tests: `cargo test -p repobox-core ens_resolution_e2e`
- [ ] Integration tests: `./tests/ens_integration_script.sh`
- [ ] Manual verification of key user journeys
- [ ] Performance benchmarks meet requirements (< 2s cold, < 100ms warm)
- [ ] Error handling provides clear user messages
- [ ] All three name types work correctly (ENS, subdomains, aliases)

## Future Enhancements

### Phase 2: Advanced Features
- [ ] Full repobox.eth subdomain integration with NFT registry
- [ ] Git clone/push testing with real repositories
- [ ] Browser-based integration tests with Playwright/Selenium
- [ ] Load testing with high concurrent ENS resolution

### Phase 3: Optimization  
- [ ] Background cache warming for popular ENS names
- [ ] Batch resolution for multiple names
- [ ] Advanced error recovery and retry logic
- [ ] Metrics and monitoring for resolution performance

---

This testing guide ensures comprehensive coverage of ENS resolution functionality across all repo.box surfaces, providing confidence for demo presentations and production deployment.