# repo.box Spec: Identity

## Overview

Identity is the foundation layer. It answers one question: **who signed this request?**

## Core Principles

- An identity is an EVM address. Period.
- The system never stores ENS names, usernames, or display names in authoritative config.
- ENS resolution is a UI convenience, resolved on-the-fly, never persisted.
- "Access follows a name" (e.g. `treasury.dao.eth`) is a **groups** concern, not an identity concern.

## Identity Format

Identities are represented as typed strings with a prefix:

```
evm:<checksummed-address>
```

Example: `evm:0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045`

The `evm:` prefix exists to future-proof for other identity types (SSH keys, GitHub accounts, etc.), but EVM is the only supported type at launch.

## Authentication: ERC-8128 (Hard Requirement)

All HTTP authentication uses **ERC-8128 Signed HTTP Requests**. No JWTs. No API keys. No session tokens. No SIWE-then-session pattern.

Every request is individually signed per RFC 9421 with an Ethereum account.

### Spec References
- ERC-8128: https://erc8128.org
- Library: `@slicekit/erc8128`
- RFC 9421: HTTP Message Signatures
- ERC-191: Signed Messages
- ERC-1271: Smart Contract Account verification

### How It Works

1. Client signs each HTTP request — covering method, path, body digest, nonce
2. Signature + metadata sent via `Signature` and `Signature-Input` headers
3. Server reconstructs signature base, verifies via `ecrecover` (EOA) or ERC-1271 (SCA)
4. Server extracts signer address from `keyid` parameter: `eip8128:<chain-id>:<address>`
5. No handshake required. Stateless verification.

### Security Posture for Git Operations

| Operation | Binding | Replay Protection |
|-----------|---------|-------------------|
| `git push` | Request-Bound | Non-Replayable |
| `git fetch` (private) | Request-Bound | Non-Replayable |
| `git fetch` (public) | No auth required | N/A |

**Request-Bound + Non-Replayable** is the mandatory baseline per ERC-8128. Any compliant verifier must accept it.

### Git Smart HTTP Protocol Mapping

Git over HTTP uses these endpoints:

- `GET /<repo>/info/refs?service=git-upload-pack` — discovery (fetch)
- `GET /<repo>/info/refs?service=git-receive-pack` — discovery (push)  
- `POST /<repo>/git-upload-pack` — fetch packfile
- `POST /<repo>/git-receive-pack` — push packfile

Each request carries its own ERC-8128 signature. The server verifies independently — no session state between requests.

### Smart Contract Wallets

Fully supported via ERC-1271. A multisig, a Safe, or any smart contract wallet can push to a repo. The `keyid` chain parameter determines which chain to verify against.

### Client-Side Key Management

**TBD** — Fran consulting with ERC-8128 authors (Slice team) on recommended patterns. Options include OS keychain, credential helpers, browser wallet delegation.

For the git shim, identity is stored in `~/.box/identity` and used to sign commits and determine permissions locally.

### Server-Side Requirements

- Nonce dedup store (in-memory with TTL or Redis) scoped per `keyid`
- RPC connection for ERC-1271 verification (smart contract wallets)
- No session store needed

## Identity Provider Interface (Internal)

```typescript
interface IdentityProvider {
  type: string;                    // "evm"
  extractIdentity(request): string; // "evm:0x1234..."
  verify(request): boolean;         // ERC-8128 signature valid?
}
```

EVM is the only provider at launch. The interface exists so SSH or other providers can be added later without changing the core.

## What Identity Does NOT Cover

- **Display names / ENS** — UI layer, not identity
- **Group membership** — See 02-groups.md
- **Permissions** — See 03-permissions.md
- **"Who controls this ENS name?"** — Group resolver pattern
