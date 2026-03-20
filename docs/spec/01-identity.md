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

For the git shim, identity is stored in `~/.repobox/identity` and used to sign commits and determine permissions locally.

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

## Git Signing Integration

repo.box extends git's native signing infrastructure rather than running alongside it.

### How It Works

- `git repobox init` sets `gpg.program = repobox` in the repo's git config
- `user.signingkey` holds the EVM address: `evm:0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045`
- When git needs a signature, it calls `box` which signs with the secp256k1 private key
- The signature is stored in the commit object — native git, not a side-channel
- `git verify-commit` works using the EVM address

`user.signingkey` is the **single source of identity**. The shim reads it for permission checks. Git reads it for signing. One config key, two purposes.

### Key Storage

Private keys are stored in `~/.repobox/keys/`, keyed by address:

```
~/.repobox/keys/0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045.key
```

`git repobox keys generate` creates a new key pair and prints the address.
`git repobox keys import <private-key>` imports an existing key.

### Identity Precedence

Standard git config resolution, no custom mechanisms:

1. `git -c user.signingkey=evm:0x...` (per-command)
2. `GIT_CONFIG_*` environment variables (per-process)
3. `.git/config` in the repo (per-repo, via `git config --local`)
4. `~/.gitconfig` conditional includes (per-directory pattern)
5. `~/.gitconfig` global default

### Multi-Agent Setup

**Granting access to a new agent (requires .repobox.yml edit permission):**

```bash
# 1. Generate a key for the agent
git repobox keys generate
# → Created: ~/.repobox/keys/0xCCC...def.key
# → Address: evm:0xCCC...def

# 2. Add the address to .repobox.yml (you need edit permission)
# Under groups:
#   agents:
#     members:
#       - evm:0xCCC...def

# 3. Commit the change (only @founders can edit .repobox.yml)
git add .repobox.yml
git commit -m "onboard agent-1"
```

**Spawning agents with separate identities:**

```bash
# Each agent process gets its own identity via environment
GIT_CONFIG_COUNT=1 \
GIT_CONFIG_KEY_0=user.signingkey \
GIT_CONFIG_VALUE_0=evm:0xCCC...def \
codex --task "fix the auth bug"
```

The agent uses normal git commands. The shim reads `user.signingkey` from the environment, checks permissions, and signs commits — all transparently.

**Audit trail:**

```
commit a1b2c3d
EVM-signed by evm:0xCCC...def
Author: agent <agent@repo.box>

    fix the auth bug

commit e5f6a7b
EVM-signed by evm:0xAAA...789
Author: alice <alice@example.com>

    onboard agent-1
```

Every commit is cryptographically tied to a specific EVM address. You know exactly which agent (or human) made which change.

**Security properties:**
- Agents cannot grant themselves access (editing .repobox.yml requires @founders permission)
- Agents cannot impersonate other identities (signing requires the private key)
- The orchestrator controls onboarding — generates keys, edits .repobox.yml, assigns identities
- Post-hoc audit: any commit's authorship is cryptographically verifiable

## Local Aliases (Address Book)

Raw EVM addresses are unreadable. Aliases give them human-friendly names — stored **locally**, not in the repo.

### Storage

Aliases live in `~/.repobox/aliases` (one per line):

```
alice = evm:0xAAA...123
claude = evm:0xBBB...456
roudy-piglet = evm:0xCCC...789
```

This file is **per-machine, not per-repo**. Different collaborators can have different names for the same addresses. The canonical identity is always the EVM address.

### CLI Commands

```bash
git repobox alias add alice evm:0xAAA...123
git repobox alias remove alice
git repobox alias list
```

When setting identity, you can name yourself at the same time:

```bash
git repobox identity set <private-key> --alias alice
# → Identity set: @alice (evm:0xAAA...123)
```

### Display

The CLI resolves aliases everywhere — error messages, logs, permission checks:

```
❌ permission denied: @claude cannot edit .repobox.yml
   (only @founders can edit .repobox.yml on main)
```

```
commit a1b2c3d
EVM-signed by @claude (evm:0xBBB...456)
```

```bash
git repobox whoami
# → @alice (evm:0xAAA...123)
```

If no alias exists for an address, the raw `evm:0x...` is shown.

### Aliases vs Groups

Both use `@` prefix but they're different things:

- **`@founders`** — a group defined in `.repobox.yml`, resolves to multiple addresses
- **`@alice`** — a local alias in `~/.repobox/aliases`, resolves to one address

The CLI distinguishes by checking the alias file first, then the config groups. Groups always contain multiple members; aliases are always 1:1.

### Sub-Agent Naming Convention

When agents spawn sub-agents, they use **plus notation** for the alias:

```
@claude+roudy-piglet = evm:0xCCC...789
@claude+swift-otter = evm:0xDDD...012
```

The parent alias + `+` + a random adjective-animal name. This makes lineage visible at a glance:

```
commit f3e2d1c
EVM-signed by @claude+roudy-piglet (evm:0xCCC...789)

    refactor database layer
```

The naming is a **convention enforced by tooling** (SKILL.md), not by the core system. The alias file just stores whatever string → address mapping you give it.

## What Identity Does NOT Cover

- **Display names / ENS** — UI layer, not identity
- **Group membership** — See 02-groups.md
- **Permissions** — See 03-permissions.md
- **"Who controls this ENS name?"** — Group resolver pattern
