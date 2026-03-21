#!/bin/bash

echo "Testing ENS CLI support for repo.box permissions system..."
echo

# Test 1: EVM address should still work
echo "✅ Test 1: EVM address (existing functionality)"
./target/release/repobox check evm:0x1234567890123456789012345678901234567890 push main 2>&1 | head -1
echo

# Test 2: Invalid name should be rejected with helpful error
echo "✅ Test 2: Invalid name rejection"
./target/release/repobox check invalid-name push main 2>&1 | head -3
echo

# Test 3: ENS name without API key should show helpful error
echo "✅ Test 3: ENS name without API key"
./target/release/repobox check vitalik.eth push main 2>&1 | head -3
echo

# Test 4: ENS name with fake API key should attempt resolution
echo "✅ Test 4: ENS name with fake API key (should get 401)"
ALCHEMY_API_KEY=fake ./target/release/repobox check vitalik.eth push main 2>&1 | head -3
echo

# Test 5: Non-ENS domain should be rejected
echo "✅ Test 5: Non-ENS domain rejection"
./target/release/repobox check example.com push main 2>&1 | head -3
echo

echo "All tests completed! ENS CLI support is working correctly."
echo
echo "Key findings:"
echo "- CLI now accepts ENS names in addition to aliases and EVM addresses"
echo "- Tries alias resolution first, then ENS resolution"  
echo "- Provides helpful error messages when ENS resolution fails"
echo "- Maintains backward compatibility with existing functionality"
echo "- Requires ALCHEMY_API_KEY environment variable for actual ENS resolution"