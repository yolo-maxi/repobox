#!/bin/bash
# wall-merge.sh — Merge a wall contribution and generate claim attestation
# Usage: ./scripts/wall-merge.sh <contributor-address> <units> [commit-hash]
#
# This script:
# 1. Finds the contributor's note in the wall repo
# 2. Merges it (if on a branch) or accepts it (if on main)  
# 3. Signs an attestation with the Mergeooor key
# 4. Writes claims/<address>.json with the claim data
# 5. Commits and pushes

set -euo pipefail

WALL_DIR="/home/xiko/wall"  # local clone of the wall repo
MERGER_KEY_FILE="$HOME/.repobox/keys/0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048.key"  # Mergeooor's private key
CONTRACT="0x57D779c322245E0b4eC33aFAb9B8EFA7e8abB371"
CHAIN_ID=8453

# Parse args
CONTRIBUTOR="${1:?Usage: wall-merge.sh <contributor-address> <units> [commit-hash]}"
UNITS="${2:?Usage: wall-merge.sh <contributor-address> <units> [commit-hash]}"
COMMIT_HASH="${3:-}"

# Validate units
if [ "$UNITS" -lt 1 ] || [ "$UNITS" -gt 10 ]; then
    echo "❌ Units must be 1-10, got $UNITS"
    exit 1
fi

# Ensure wall repo is cloned
if [ ! -d "$WALL_DIR" ]; then
    echo "📦 Cloning wall repo..."
    git clone https://git.repo.box/0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048/wall.git "$WALL_DIR"
fi

cd "$WALL_DIR"
git pull origin main

# Find the commit hash if not provided
if [ -z "$COMMIT_HASH" ]; then
    # Look for the latest commit touching notes/<address>.md
    COMMIT_HASH=$(git log --oneline --all -- "notes/${CONTRIBUTOR}.md" | head -1 | cut -d' ' -f1)
    if [ -z "$COMMIT_HASH" ]; then
        echo "❌ No commit found for contributor $CONTRIBUTOR"
        exit 1
    fi
fi

echo "📝 Contributor: $CONTRIBUTOR"
echo "⭐ Units: $UNITS"  
echo "🔗 Commit: $COMMIT_HASH"

# Get full commit hash (pad to 32 bytes for the contract)
FULL_HASH=$(git rev-parse "$COMMIT_HASH")
# Convert git hash (20 bytes) to bytes32 (32 bytes, zero-padded)
COMMIT_BYTES32="0x${FULL_HASH}000000000000000000000000"

# Determine nonce (count existing claims for this contributor + 1)
NONCE=1
if [ -f "claims/${CONTRIBUTOR}.json" ]; then
    NONCE=$(python3 -c "import json; claims=json.load(open('claims/${CONTRIBUTOR}.json')); print(len(claims) if isinstance(claims, list) else 2)")
fi

echo "🔢 Nonce: $NONCE"

# Generate the digest: keccak256(abi.encodePacked(contributor, commitHash, units, nonce))
# Using cast for this
export PATH="$HOME/.foundry/bin:$PATH"

# abi.encodePacked(address, bytes32, uint128, uint256)
# address = 20 bytes, bytes32 = 32 bytes, uint128 = 16 bytes, uint256 = 32 bytes
ADDR_HEX=$(echo "$CONTRIBUTOR" | sed 's/0x//' | tr '[:upper:]' '[:lower:]')
COMMIT_HEX=$(echo "$COMMIT_BYTES32" | sed 's/0x//')
UNITS_HEX=$(printf '%032x' "$UNITS")  # uint128 = 16 bytes = 32 hex chars
NONCE_HEX=$(printf '%064x' "$NONCE")  # uint256 = 32 bytes = 64 hex chars
PACKED="0x${ADDR_HEX}${COMMIT_HEX}${UNITS_HEX}${NONCE_HEX}"
DIGEST=$(cast keccak "$PACKED")

echo "📋 Digest: $DIGEST"

# Sign the digest with Mergeooor key (eth_sign format: \x19Ethereum Signed Message:\n32 + digest)
MERGER_PK=$(cat "$MERGER_KEY_FILE")

SIGNATURE=$(cast wallet sign --private-key "$MERGER_PK" "$DIGEST" 2>&1)

echo "✍️ Signature: ${SIGNATURE:0:20}..."

# Write claim file
mkdir -p claims
cat > "claims/${CONTRIBUTOR}.json" << EOF
{
  "contributor": "$CONTRIBUTOR",
  "commitHash": "$COMMIT_BYTES32",
  "units": $UNITS,
  "nonce": $NONCE,
  "signature": "$SIGNATURE",
  "contract": "$CONTRACT",
  "chainId": $CHAIN_ID,
  "pool": "0xeFFDE09639FA692af78Ce37133324B03aB62f3a9",
  "gitCommit": "$FULL_HASH",
  "scoredAt": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo "📄 Claim file written: claims/${CONTRIBUTOR}.json"

# Commit and push with Mergeooor identity
git config user.name "Ocean"
git config user.email "oceanvael@gmail.com"
git config user.signingkey "0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048"
git config gpg.program "/home/xiko/repobox/target/release/repobox"
git config commit.gpgsign true

git add "claims/${CONTRIBUTOR}.json"
git commit -S -m "score: ${CONTRIBUTOR:0:10}... — ${UNITS} units"
git push origin main

echo ""
echo "✅ Done! Claim file pushed to wall repo."
echo ""
echo "🔗 The contributor can now call:"
echo "   GraffitiPool.claim("
echo "     $COMMIT_BYTES32,"
echo "     $UNITS,"
echo "     $NONCE,"
echo "     $SIGNATURE"
echo "   )"
echo ""
echo "   Contract: $CONTRACT (Base)"
