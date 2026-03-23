# repo.box ENS Gateway

CCIP-Read gateway server for resolving repo.box ENS subdomains to Ethereum addresses.

## Overview

This gateway enables ENS resolution of repo.box aliases like `clever-green-canyon.repo.box` to their associated Ethereum addresses using the CCIP-Read standard (ERC-3668).

### Architecture

1. **OffchainResolver Contract** - Deployed on Ethereum mainnet as the ENS resolver for repo.box
2. **CCIP Gateway Server** - This TypeScript server that responds to resolution requests
3. **SQLite Name Index** - Indexed on-chain RepoBoxNames data (token name, owner, resolved address)
4. **JSON Alias Fallback** - Static aliases for local/dev and free aliases

## Quick Start

### 1. Install Dependencies

```bash
pnpm install
```

### 2. Configure Environment

Copy `.env.example` to `.env` and set:

```bash
# Gateway server port
PORT=3001

# Path to JSON aliases fallback file
DB_PATH=./data/aliases.json

# Path to SQLite index DB (shared repobox db is recommended)
NAMES_INDEX_DB_PATH=/var/lib/repobox/repos/repobox.db

# RepoBoxNames contract address to index
NAMES_CONTRACT_ADDRESS=0x891eaE87e40dF6B26B7ae4877f2bdd781313fAe8

# Private key for signing responses (must match resolver contract signer)
GATEWAY_PRIVATE_KEY=0x...

# Ethereum mainnet RPC URL
RPC_URL=https://eth.drpc.org

# Optional periodic sync cadence (ms)
NAMES_SYNC_INTERVAL_MS=300000
```

### 3. Start the Gateway

```bash
# Development
pnpm dev

# Production
pnpm build
pnpm start
```

## API Endpoints

### Health Check
```
GET /health
```

### CCIP Resolution
```
GET /{sender}/{data}.json
```

This endpoint is called automatically by ENS clients when resolving repo.box domains.

## How It Works

1. **ENS Query**: User queries `name.repobox.eth` in their wallet
2. **Resolver Lookup**: ENS finds the OffchainResolver contract for repo.box
3. **CCIP-Read**: Resolver contract reverts with `OffchainLookup` pointing to this gateway
4. **Gateway Resolution**: This server resolves alias/address using indexed on-chain names first
5. **Signed Response**: Gateway signs and returns the Ethereum address
6. **Verification**: Resolver contract verifies the signature and returns the address

### Reverse Resolution Priority (address → name)

1. Match by `resolvedAddress` in indexed RepoBoxNames
2. Fallback to `ownerOf(tokenId)` in indexed RepoBoxNames
3. Fallback to static JSON aliases

## Development

### Project Structure

```
src/
├── index.ts      # Express server setup
├── gateway.ts    # CCIP gateway logic
└── database.ts   # SQLite database wrapper
```

### Testing

The gateway can be tested by simulating CCIP requests or by deploying the full ENS resolver stack.

## Deployment

1. **Deploy the OffchainResolver contract** (see `../contracts/`)
2. **Configure ENS** to use your resolver for repo.box
3. **Deploy this gateway server** to a public URL
4. **Start the server** with the correct signing key

See `../scripts/deploy-ens-resolver.sh` for detailed deployment instructions.

## Security

- The gateway signs responses using a private key
- The resolver contract verifies signatures before returning addresses
- Only the configured signer can provide valid responses
- All queries are logged for monitoring

## Dependencies

- **viem**: Ethereum utilities and ABI encoding/decoding
- **express**: HTTP server framework
- **sqlite3**: Database access
- **cors**: Cross-origin request handling

## License

MIT