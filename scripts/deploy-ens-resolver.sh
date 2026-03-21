#!/bin/bash

# Deploy script for repo.box ENS Offchain Resolver
# Usage: ./scripts/deploy-ens-resolver.sh <PRIVATE_KEY> <RPC_URL> <GATEWAY_SIGNER_ADDRESS>

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check arguments
if [ $# -lt 3 ]; then
    echo -e "${RED}Error: Missing required arguments${NC}"
    echo "Usage: $0 <PRIVATE_KEY> <RPC_URL> <GATEWAY_SIGNER_ADDRESS>"
    echo ""
    echo "Example:"
    echo "  $0 0x1234... https://rpc.ankr.com/eth 0xabcd..."
    echo ""
    echo "Where:"
    echo "  PRIVATE_KEY         - Private key for deployment (must have ETH for gas)"
    echo "  RPC_URL            - Ethereum mainnet RPC endpoint"
    echo "  GATEWAY_SIGNER_ADDRESS - Address that will sign CCIP gateway responses"
    exit 1
fi

PRIVATE_KEY="$1"
RPC_URL="$2"
GATEWAY_SIGNER="$3"
GATEWAY_URL="https://ens.repo.box/{sender}/{data}.json"

# Validate inputs
if [[ ! "$PRIVATE_KEY" =~ ^0x[a-fA-F0-9]{64}$ ]]; then
    echo -e "${RED}Error: Invalid private key format${NC}"
    exit 1
fi

if [[ ! "$GATEWAY_SIGNER" =~ ^0x[a-fA-F0-9]{40}$ ]]; then
    echo -e "${RED}Error: Invalid gateway signer address format${NC}"
    exit 1
fi

echo -e "${BLUE}🚀 Deploying ENS Offchain Resolver for repo.box${NC}"
echo -e "${YELLOW}Gateway URL: $GATEWAY_URL${NC}"
echo -e "${YELLOW}Gateway Signer: $GATEWAY_SIGNER${NC}"
echo ""

# Navigate to contracts directory
cd contracts

# Check if solidity compiler is available
if ! command -v solc &> /dev/null; then
    echo -e "${RED}Error: solidity compiler (solc) is not installed${NC}"
    exit 1
fi

echo -e "${BLUE}📝 Compiling contract...${NC}"

# Compile the contract
solc --abi --bin --optimize --optimize-runs 200 OffchainResolver.sol > compilation_output.txt 2>&1

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Compilation failed${NC}"
    cat compilation_output.txt
    exit 1
fi

# Extract bytecode (this is a simplified approach - in production you'd use Foundry)
BYTECODE=$(grep -A 1 "======= OffchainResolver.sol:OffchainResolver =======" compilation_output.txt | tail -1)

if [ -z "$BYTECODE" ]; then
    echo -e "${RED}Error: Could not extract bytecode${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Compilation successful${NC}"

# For this script, we'll provide instructions for manual deployment
# In a real production environment, you'd use Foundry's forge script or cast

echo ""
echo -e "${BLUE}📋 DEPLOYMENT INSTRUCTIONS${NC}"
echo -e "${YELLOW}Due to the complexity of constructor encoding, please use the following steps:${NC}"
echo ""

echo -e "${BLUE}1. Deploy using Foundry (Recommended):${NC}"
echo ""
echo "   # Install Foundry if not already installed:"
echo "   curl -L https://foundry.paradigm.xyz | bash"
echo "   foundryup"
echo ""
echo "   # Deploy the contract:"
echo "   forge script script/DeployOffchainResolver.s.sol:SimpleDeployScript \\"
echo "       --rpc-url $RPC_URL \\"
echo "       --private-key $PRIVATE_KEY \\"
echo "       --broadcast"
echo ""

echo -e "${BLUE}2. Deploy using cast:${NC}"
echo ""
echo "   cast send --rpc-url $RPC_URL \\"
echo "        --private-key $PRIVATE_KEY \\"
echo "        --create \\"
echo "        \$(cast abi-encode \"constructor(string,address)\" \"$GATEWAY_URL\" \"$GATEWAY_SIGNER\")$BYTECODE"
echo ""

echo -e "${BLUE}3. Manual deployment steps:${NC}"
echo ""
echo -e "${YELLOW}Contract bytecode:${NC}"
echo "$BYTECODE"
echo ""
echo -e "${YELLOW}Constructor parameters:${NC}"
echo "Gateway URL: $GATEWAY_URL"
echo "Signer Address: $GATEWAY_SIGNER"
echo ""

echo -e "${BLUE}📤 After deployment:${NC}"
echo ""
echo "1. Note the deployed contract address"
echo ""
echo "2. Set up the ENS resolver by calling setResolver() on the ENS Registry:"
echo "   - Contract: 0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e (ENS Registry)"
echo "   - Function: setResolver(bytes32 node, address resolver)"
echo "   - node: [repo.box ENS node hash]"
echo "   - resolver: [your deployed resolver address]"
echo ""
echo "3. Start the CCIP gateway server:"
echo "   cd ../repobox-ens-gateway"
echo "   pnpm install"
echo "   echo \"GATEWAY_PRIVATE_KEY=[private key for $GATEWAY_SIGNER]\" > .env"
echo "   echo \"DB_PATH=../repobox-server/repobox.db\" >> .env"
echo "   pnpm dev"
echo ""

echo -e "${GREEN}🎉 Deployment script completed!${NC}"

# Cleanup
rm -f compilation_output.txt

echo ""
echo -e "${BLUE}📋 NEXT STEPS SUMMARY:${NC}"
echo "1. Deploy the contract using one of the methods above"
echo "2. Configure ENS to use your resolver for repo.box domain"
echo "3. Start the CCIP gateway server"
echo "4. Test the resolution with an ENS client"