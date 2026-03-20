# repo.box

**Git permissions for AI agents.** A permission layer that sits between your agents and git, enforcing who can push, merge, edit, and create — using EVM wallet identities.

```
curl -sSf https://repo.box/install.sh | sh
```

## Why

AI agents are writing code. They're committing, pushing, opening PRs. But git has no concept of *agent permissions* — if an agent has SSH access, it has full access.

repo.box fixes this. One YAML file in your repo defines exactly what each agent (or group of agents) can do:

```yaml
groups:
  founders:
    - evm:0xAAA...
  agents:
    - evm:0xBBB...
  # Token-gate a group — anyone holding WETH on Base
  weth-holders:
    resolver: onchain
    chain: 8453
    contract: "0x4200000000000000000000000000000000000006"
    function: balanceOf

permissions:
  default: deny
  rules:
    - founders own >*                    # full control everywhere
    - agents push >feature/**            # agents push to feature branches only
    - agents edit * >feature/**          # agents edit files on feature branches
    - agents append ./CHANGELOG.md       # agents can append to changelog anywhere
    - agents not edit .repobox.yml       # agents can never touch the config
    - weth-holders push >*               # token holders can push anywhere
```

## How It Works

repo.box installs as a transparent **git shim**. It intercepts git commands (`commit`, `push`, `merge`, `checkout -b`, `branch -d`) and checks permissions before passing through to real git.

```
you → git commit → [repobox shim] → permission check → real git
```

Identities are **EVM wallet addresses**. Each agent gets a keypair, and permissions are tied to those addresses — not SSH keys, not usernames.

## Install

```bash
curl -sSf https://repo.box/install.sh | sh
```

This downloads the binary, sets up the git shim, and adds it to your PATH.

### From Source

```bash
git clone https://github.com/repobox/repobox
cd repobox
cargo build --release
sudo cp target/release/repobox /usr/local/bin/
```

## Quick Start

```bash
# Initialize repo.box in your project
git repobox init

# Generate a keypair for yourself
git repobox keys generate --alias alice

# Generate one for your agent
git repobox keys generate --alias claude

# Check your identity
git repobox whoami
# → alice (evm:0x7D5b...)

# Edit .repobox.yml to define permissions, then:
git add .repobox.yml
git commit -m "configure permissions"
```

## Concepts

### Identities

Every user and agent is identified by an EVM address:
```
evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661
```

Generate keys with `git repobox keys generate`. Keys are stored locally in `~/.repobox/keys/`.

### Groups

Group members statically or dynamically:

```yaml
groups:
  # Static list
  founders:
    - evm:0xAAA...
    - evm:0xBBB...

  # Include other groups
  core-team:
    members:
      - evm:0xCCC...
    includes:
      - founders

  # HTTP resolver — members fetched from an API
  company:
    resolver: http
    url: https://api.example.com/groups/company
    cache_ttl: 60

  # On-chain resolver — membership from any EVM contract
  token-holders:
    resolver: onchain
    chain: 8453           # Base
    contract: "0xDDD..."
    function: balanceOf   # Any non-zero return = member
    cache_ttl: 300
```

On-chain resolvers use **truthy/falsy** semantics: any non-zero return value means the address is a member. This means `balanceOf`, `isMember`, `hasRole`, and similar functions all work natively — no custom contracts needed.

Supported chains: Ethereum, Base, Optimism, Arbitrum, Polygon, and [40+ more](https://repo.box).

### Verbs

| Verb | What it controls |
|------|-----------------|
| `push` | Push to a branch |
| `merge` | Merge into a branch |
| `create` | Create a new branch |
| `delete` | Delete a branch |
| `force-push` | Force-push to a branch |
| `edit` | Modify any file (superset of write/append) |
| `write` | Create new files |
| `append` | Append to existing files (no deletions) |
| `own` | All of the above (expands to 8 rules) |

The shim inspects git diffs at commit time to classify changes: new files need `write`, append-only changes need `append`, and modifications need `edit`. Having `edit` permission covers both `write` and `append`.

### Targets

```yaml
rules:
  - agents push >main              # specific branch
  - agents push >feature/**        # glob pattern
  - agents push >*                 # all branches
  - agents edit .repobox.yml       # specific file
  - agents edit src/**             # file glob
  - agents edit *                  # all files
  - agents edit * >feature/**     # files + branch scope
```

### Rule Evaluation

- **First match wins** (firewall model — top to bottom)
- **`default: deny`** recommended (whitelist approach)
- **Explicit deny**: prefix with `not` → `agents not push >main`
- **Implicit deny**: if rules exist for a verb+target but none match the identity

### The `own` Shorthand

```yaml
- founders own >*
```

Expands at parse time to: `push`, `merge`, `create`, `delete`, `force-push`, `edit`, `write`, `append` — all on the same target.

## Commands

```bash
git repobox init              # Initialize repo.box
git repobox keys generate     # Generate a new keypair
git repobox keys list         # List all keys
git repobox use <alias>       # Switch active identity
git repobox whoami            # Show current identity
git repobox alias set <n> <a> # Set an alias
git repobox check <id> <v> <t> # Check a permission
git repobox lint              # Validate config
git rb status                 # Show identity, groups, permissions
```

`git rb` is a shorthand for `git repobox`.

## Lint

repo.box includes a built-in linter with 8 checks that auto-runs when `.repobox.yml` is staged for commit:

- Unreachable rules (shadowed by earlier rules)
- Unknown group references
- Empty groups
- Duplicate rules
- Wildcard subject ordering issues
- And more

Lint warnings are printed to stderr but don't block commits — parse errors do.

## On-Chain Resolver Proxy

For on-chain group resolution, the CLI calls a resolver proxy server that handles RPC calls:

```bash
# Start the resolver server
ALCHEMY_API_KEY=your-key repobox-server --bind 0.0.0.0:3456

# Point the CLI at it
export REPOBOX_SERVER=https://your-server.com/api
```

The server proxies `eth_call` requests to Alchemy, supporting 40+ EVM chains.

## Environment Variables

| Variable | Description |
|----------|------------|
| `REPOBOX_IDENTITY` | Override active identity (e.g. `evm:0xAAA...`) |
| `REPOBOX_SERVER` | Resolver proxy URL for on-chain groups |

## Architecture

```
repobox-core/    — Parser, engine, resolver, lint, shim, signing
repobox-cli/     — The `git repobox` / `git rb` binary
repobox-server/  — Axum server: Git Smart HTTP + /api/resolve proxy
```

## License

MIT
