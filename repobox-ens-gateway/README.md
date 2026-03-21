# repo.box ENS Gateway

CCIP-Read gateway server for resolving repo.box ENS subdomains to Ethereum addresses.

## Overview

This gateway enables ENS resolution of repo.box aliases like `clever-green-canyon.repo.box` to their associated Ethereum addresses using the CCIP-Read standard (ERC-3668).

### Architecture

1. **OffchainResolver Contract** - Deployed on Ethereum mainnet as the ENS resolver for repo.box
2. **CCIP Gateway Server** - This TypeScript server that responds to resolution requests
3. **Database** - Uses the same SQLite database as repobox-server for alias storage

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

# Path to repobox SQLite database
DB_PATH=../repobox-server/repobox.db

# Private key for signing responses (must match resolver contract signer)
GATEWAY_PRIVATE_KEY=0x...

# Optional: Custom RPC URL
RPC_URL=https://rpc.ankr.com/eth
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

1. **ENS Query**: User queries `clever-green-canyon.repo.box` in their wallet
2. **Resolver Lookup**: ENS finds the OffchainResolver contract for repo.box
3. **CCIP-Read**: Resolver contract reverts with `OffchainLookup` pointing to this gateway
4. **Gateway Resolution**: This server looks up the alias in the database
5. **Signed Response**: Gateway signs and returns the Ethereum address
6. **Verification**: Resolver contract verifies the signature and returns the address

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