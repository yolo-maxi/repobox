# repo.box

Git permission layer for AI agents. EVM-signed commits. On-chain group resolvers. Granular file and branch permissions.

## What It Is

repo.box is a git toolchain that replaces SSH keys and GitHub accounts with EVM wallets.

- Every commit is cryptographically signed with an Ethereum private key (via `gpg.program`)
- Permissions (who can push, edit files, merge branches) are defined in `.repobox/config.yml` per repo
- The server enforces permissions on push. The optional client-side shim also enforces them locally before commands execute
- Groups can resolve dynamically — via HTTP endpoints or on-chain contract calls (token holders, NFT owners, DAO members)
- No accounts, no sign-ups. Your EVM address is your identity

**Platform**: https://repo.box  
**Explorer**: https://repo.box/explore  
**Git server**: https://git.repo.box

---

## Installation

```bash
curl -sSf https://repo.box/install.sh | sh
```

This places the `repobox` binary on your system.

### `repobox init` vs `repobox setup`

These are two separate commands with very different scopes:

**`repobox init`** — Run inside a git repo. Does two things:
1. Creates `.repobox/config.yml` (the permission config)
2. Sets `gpg.program = /path/to/repobox` and `commit.gpgsign = true` in the repo's **local** git config

After `repobox init`, commits in that repo are automatically EVM-signed. No other repos are affected. No PATH changes. No global config.

**`repobox setup`** — Installs the global git shim. Does two things:
1. Creates a symlink at `~/.repobox/bin/git` → the repobox binary
2. Prepends `~/.repobox/bin` to your PATH (via `~/.bashrc` / `~/.zshrc`)

After `repobox setup`, every `git` command goes through repobox first. **For repos without `.repobox/config.yml`, repobox is a complete passthrough** — it delegates to real git with zero overhead. For repos where you've run `repobox init`, permissions are enforced locally before the command executes.

With the shim active, repobox subcommands are accessed as git subcommands: `git repobox init`, `git repobox keys generate`, etc.

**We recommend running `repobox setup`.** The passthrough is transparent — your non-repobox repos work exactly as before.

To remove: `repobox setup --remove`

---

## Commands

All commands below are shown as `repobox <cmd>`. If you've run `repobox setup`, use `git repobox <cmd>` instead.

### Identity & Keys

| Command | Description | Requires setup? |
|---------|-------------|-----------------|
| `repobox keys generate` | Generate a new EVM key pair | No |
| `repobox keys import <key>` | Import an existing private key | No |
| `repobox keys list` | List all stored keys | No |
| `repobox whoami` | Show current identity (address + alias) | No |
| `repobox identity set <addr>` | Set active identity | No |
| `repobox use <alias>` | Switch identity by alias (shorthand) | No |
| `repobox alias add <name> <addr>` | Create a named alias for an address | No |
| `repobox alias list` | List all aliases | No |

Keys are stored at `~/.repobox/keys/<address>.key`.

### Repo Management

| Command | Description | Requires setup? |
|---------|-------------|-----------------|
| `repobox init` | Initialize repo.box in current repo | No |
| `repobox init --force` | Overwrite existing config | No |
| `repobox status` | Show identity, groups, permissions summary | No |
| `repobox lint` | Validate `.repobox/config.yml` | No |
| `repobox check <identity> <verb> <target>` | Test if an identity can perform an action | No |

### Shim (requires `repobox setup`)

After setup, the following git commands are intercepted and checked against `.repobox/config.yml`:

- `git commit` — checks file-level verbs (`write`, `append`, `edit`) on staged files
- `git push` — checks `push` verb on the target branch; checks `force-push` if `--force`
- `git merge` — checks `merge` verb on the target branch
- `git branch` — checks `create` / `delete` verbs
- `git checkout -b` — checks `create` verb for new branches

These commands pass through without checks: `git status`, `git log`, `git diff`, `git add`, `git stash`, `git fetch`, `git clone`, `git remote`, `git show`, `git tag`, `git reflog`, `git blame`, `git bisect`, `git archive`.

---

## Groups

Groups are named sets of identities defined in `.repobox/config.yml`.

### Static Groups

```yaml
groups:
  founders:
    - evm:0xAlice0000000000000000000000000000000001
    - evm:0xBob00000000000000000000000000000000002
  agents:
    - evm:0xAgent000000000000000000000000000000003
```

### Group Includes

Groups can include other groups:

```yaml
groups:
  core:
    - evm:0xAlice...
  extended:
    - evm:0xBob...
    includes:
      - core
```

### HTTP Resolver (Off-Chain)

Resolves membership dynamically via an HTTP endpoint:

```yaml
groups:
  company:
    resolver: http
    url: https://api.example.com/groups/company
    cache_ttl: 60  # seconds, 0 = no cache
```

The server calls `GET <url>/members/<address>` and expects `{ "member": true }` or `{ "member": false }`.

### On-Chain Resolver

Resolves membership by calling a smart contract function:

```yaml
groups:
  holders:
    resolver: onchain
    chain: 8453              # Base mainnet
    contract: "0xDDD..."
    function: isMember       # function(address) → bool
    cache_ttl: 300           # seconds
```

The repo.box server proxies the call: `GET /resolve?chain=8453&contract=0x...&function=isMember&address=0x...`

### Mixed Groups

Groups can combine static members with a resolver:

```yaml
groups:
  team:
    - evm:0xAlice...         # always a member
    resolver: http           # also check dynamically
    url: https://api.example.com/team
    cache_ttl: 120
```

---

## Permission Rules

Rules are defined in `.repobox/config.yml` under `permissions`. Each rule is a **Subject-Verb-Target** triple.

### Syntax

```
<subject> <verb> <target>
```

- **Subject**: `*` (everyone), a group name, or `evm:0x...` (specific identity)
- **Verb**: see table below
- **Target**: a file glob (`./contracts/**`), a branch (`>main`), or combined (`./src/** >main`)

Deny rules use `not`:
```
agents not push >main
```

### Verbs

| Verb | Applies to | Checked when | Description |
|------|-----------|-------------|-------------|
| `read` | Files | Clone / fetch (server) | Can read / clone the repo |
| `push` | Branches | `git push` | Can push to the branch |
| `merge` | Branches | `git merge` | Can merge into the branch |
| `create` | Branches | `git branch`, `git checkout -b` | Can create the branch |
| `delete` | Branches | `git branch -d` | Can delete the branch |
| `force-push` | Branches | `git push --force` | Can force-push to the branch |
| `write` | Files | `git commit` | Can create new files (not modify existing) |
| `append` | Files | `git commit` | Can add lines to files (no deletions) |
| `edit` | Files | `git commit` | Can modify or delete content (superset of `write` and `append`) |

**Verb hierarchy**: `edit` > `append` > `write`. If you have `edit`, you implicitly have `append` and `write`.

**Verb classification** (at commit time, client-side shim):
- New file (staged as `A`) → `write`
- Modified file, only additions in diff, zero deletions → `append`  
- Modified file with any deletions → `edit`

### Rule Evaluation

1. Find all rules matching the verb AND target
2. If no rules match → use `default` policy (`allow` or `deny`)
3. If rules exist, walk top-to-bottom. First rule where the subject matches wins
4. If rules exist but none match the identity → **implicit deny** (even with `default: allow`)

This means: once you write a rule for a verb+target, it becomes an allowlist for that combination.

### Target Syntax

- `>main` — branch `main`
- `>feature/**` — branches matching `feature/*`  
- `>*` — all branches
- `./contracts/**` — files under `contracts/`
- `./**` — all files
- `./src/** >main` — files under `src/` on branch `main`

File paths must start with `./`.

---

## Full Config Example

```yaml
groups:
  founders:
    - evm:0xAlice0000000000000000000000000000000001
    - evm:0xBob00000000000000000000000000000000002

  agents:
    - evm:0xAgent1000000000000000000000000000000003
    - evm:0xAgent2000000000000000000000000000000004

  community:
    includes:
      - founders
      - agents
    resolver: http
    url: https://api.example.com/community
    cache_ttl: 120

  token-holders:
    resolver: onchain
    chain: 8453
    contract: "0xTokenContract..."
    function: balanceOf
    cache_ttl: 300

permissions:
  default: deny
  rules:
    # Access
    - "* read ./**"                    # anyone can clone

    # Branch permissions
    - "community push >main"           # community can push to main
    - "agents push >agent/**"          # agents can push to agent/ branches
    - "* create >feature/**"           # anyone can create feature branches
    - "founders merge >main"           # only founders merge to main
    - "founders force-push >*"         # only founders can force-push
    - "founders delete >*"             # only founders can delete branches

    # File permissions
    - "* append ./wall.jsonl"          # anyone can append to wall.jsonl
    - "founders edit ./**"             # founders can edit anything
    - "agents edit ./src/**"           # agents can edit source code
    - "agents not edit ./.repobox/**"  # but agents cannot edit config
    - "token-holders write ./docs/**"  # token holders can add docs

# Optional: x402 paid access
x402:
  read_price: "1.00"                   # USDC
  recipient: "0xRecipient..."
  network: base
```

---

## The Platform

### repo.box Git Server

https://git.repo.box hosts git repositories. No account needed.

**To create a repo**: just push to it. Your first signed commit establishes you as the owner.

```bash
repobox keys generate
repobox init
git remote add origin https://git.repo.box/<your-address>/<repo-name>.git
git add . && git commit -m "first commit"
git push -u origin main
```

Your repo is immediately live at `https://git.repo.box/<address>/<repo>.git`. Anyone can clone it (if read permissions allow). The server enforces your `.repobox/config.yml` on every push.

**Important**: If you don't add a `.repobox/config.yml`, the repo has no permission enforcement — anyone who knows the URL can push. Run `repobox init` and configure your permissions before sharing.

### Explorer

Browse all repos at https://repo.box/explore. See recent activity, repo contents, and permission configs.

### x402 Paid Access

Repos can require USDC payment for read access using the x402 protocol:

```yaml
x402:
  read_price: "1.00"
  recipient: "0xYourAddress..."
  network: base
```

Clone attempts return `402 Payment Required` with payment headers. Compatible x402 clients pay automatically.
