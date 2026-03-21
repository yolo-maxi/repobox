#!/bin/bash

# Deploy script for RepoBox ENS NFT contracts
# Usage: ./scripts/deploy-ens-nft.sh [network] [recipient_address]

set -e

# Default values
NETWORK=${1:-"mainnet"}
RECIPIENT=${2:-""}

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Deploying RepoBox ENS NFT System${NC}"
echo "=================================="
echo "Network: $NETWORK"

# Check if recipient address is provided
if [ -z "$RECIPIENT" ]; then
    echo -e "${RED}❌ Error: Recipient address is required${NC}"
    echo "Usage: ./scripts/deploy-ens-nft.sh [network] [recipient_address]"
    echo "Example: ./scripts/deploy-ens-nft.sh mainnet 0x742d35Cc6335C4532C5a1DE19f31BE0A5A0b9851"
    exit 1
fi

# Validate recipient address
if [[ ! "$RECIPIENT" =~ ^0x[a-fA-F0-9]{40}$ ]]; then
    echo -e "${RED}❌ Error: Invalid recipient address format${NC}"
    exit 1
fi

echo "Recipient: $RECIPIENT"
echo ""

# Set forge options
FORGE_OPTS=""
if [ "$NETWORK" != "anvil" ] && [ "$NETWORK" != "localhost" ]; then
    FORGE_OPTS="--rpc-url $NETWORK --broadcast --verify"
else
    FORGE_OPTS="--rpc-url http://localhost:8545 --broadcast"
fi

# Make sure we're in the correct directory
cd "$(dirname "$0")/.."

# Check if forge is available
if ! command -v forge &> /dev/null; then
    echo -e "${RED}❌ Error: Forge not found. Please install Foundry first.${NC}"
    echo "Visit: https://getfoundry.sh/"
    exit 1
fi

# Build contracts first
echo -e "${YELLOW}🔨 Building contracts...${NC}"
if ! forge build; then
    echo -e "${RED}❌ Error: Failed to build contracts${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Contracts built successfully${NC}"
echo ""

# Create deployment script
cat > script/Deploy.s.sol << 'EOF'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/RepoBoxNames.sol";
import "../src/RepoBoxMint.sol";

contract DeployScript is Script {
    function run() external {
        address recipient = vm.envAddress("RECIPIENT_ADDRESS");

        vm.startBroadcast();

        // Deploy NFT contract
        console.log("Deploying RepoBoxNames...");
        RepoBoxNames nftContract = new RepoBoxNames();
        console.log("RepoBoxNames deployed at:", address(nftContract));

        // Deploy mint contract
        console.log("Deploying RepoBoxMint...");
        RepoBoxMint mintContract = new RepoBoxMint(address(nftContract), recipient);
        console.log("RepoBoxMint deployed at:", address(mintContract));

        // Set mint contract
        console.log("Setting mint contract...");
        nftContract.setMintContract(address(mintContract));
        console.log("Mint contract set successfully");

        vm.stopBroadcast();

        // Output deployment info
        console.log("\n=== DEPLOYMENT COMPLETE ===");
        console.log("NFT Contract:", address(nftContract));
        console.log("Mint Contract:", address(mintContract));
        console.log("Recipient:", recipient);
        console.log("Owner:", nftContract.owner());
        console.log("Base Price:", mintContract.BASE_PRICE() / 1e18, "ETH");
        console.log("Price Increment:", mintContract.PRICE_INCREMENT() / 1e18, "ETH");
    }
}
EOF

# Create script directory if it doesn't exist
mkdir -p script

# Set environment variable
export RECIPIENT_ADDRESS=$RECIPIENT

echo -e "${YELLOW}📦 Deploying contracts to $NETWORK...${NC}"
echo "This may take a few minutes..."
echo ""

# Deploy contracts
if forge script script/Deploy.s.sol --skip test $FORGE_OPTS; then
    echo ""
    echo -e "${GREEN}🎉 Deployment successful!${NC}"
    echo ""

    # Get deployed addresses from the broadcast files
    if [ -d "broadcast/Deploy.s.sol" ]; then
        LATEST_RUN=$(ls -t broadcast/Deploy.s.sol/ | head -1)
        if [ -f "broadcast/Deploy.s.sol/$LATEST_RUN/run-latest.json" ]; then
            NFT_ADDRESS=$(jq -r '.transactions[] | select(.contractName == "RepoBoxNames") | .contractAddress' "broadcast/Deploy.s.sol/$LATEST_RUN/run-latest.json")
            MINT_ADDRESS=$(jq -r '.transactions[] | select(.contractName == "RepoBoxMint") | .contractAddress' "broadcast/Deploy.s.sol/$LATEST_RUN/run-latest.json")

            echo -e "${GREEN}📋 DEPLOYMENT SUMMARY${NC}"
            echo "======================"
            echo "Network: $NETWORK"
            echo "NFT Contract: $NFT_ADDRESS"
            echo "Mint Contract: $MINT_ADDRESS"
            echo "Recipient: $RECIPIENT"
            echo ""

            # Verification commands (if not anvil/localhost)
            if [ "$NETWORK" != "anvil" ] && [ "$NETWORK" != "localhost" ]; then
                echo -e "${YELLOW}🔍 Contract Verification Commands:${NC}"
                echo ""
                echo "# Verify NFT Contract"
                echo "forge verify-contract $NFT_ADDRESS src/RepoBoxNames.sol:RepoBoxNames --rpc-url $NETWORK"
                echo ""
                echo "# Verify Mint Contract"
                echo "forge verify-contract $MINT_ADDRESS src/RepoBoxMint.sol:RepoBoxMint --constructor-args \$(cast abi-encode \"constructor(address,address)\" $NFT_ADDRESS $RECIPIENT) --rpc-url $NETWORK"
                echo ""
            fi

            echo -e "${YELLOW}💡 Next Steps:${NC}"
            echo "1. Test the contracts using the mint function"
            echo "2. Set up your frontend to interact with the contracts"
            echo "3. Configure your ENS resolver to point to the NFT contract"
            echo ""
            echo "Example mint call:"
            echo "cast send $MINT_ADDRESS \"mint(string,address)\" \"yourname\" $RECIPIENT --value 0.0005ether --rpc-url $NETWORK --private-key \$PRIVATE_KEY"
        fi
    fi
else
    echo -e "${RED}❌ Deployment failed!${NC}"
    exit 1
fi

# Clean up temporary script
rm -f script/Deploy.s.sol

echo -e "${GREEN}✨ All done!${NC}"