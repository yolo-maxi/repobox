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

**`repobox setup`** — Installs the global git shim. Does three things:
1. Creates a symlink at `~/.repobox/bin/git` → the repobox binary
2. Prepends `~/.repobox/bin` to your PATH (via `~/.bashrc` / `~/.zshrc`)
3. Sets up the credential helper: `git config --global credential.helper "!repobox credential-helper"`

After `repobox setup`, every `git` command goes through repobox first, and pushes/clones to `git.repo.box` are automatically authenticated using your EVM key (no passwords, no SSH keys). **For repos without `.repobox/config.yml`, repobox is a complete passthrough** — it delegates to real git with zero overhead. For repos where you've run `repobox init`, permissions are enforced locally before the command executes.

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

Groups can include other groups. In the simple list form, any entry that doesn't start with `evm:` is treated as a group include:

```yaml
groups:
  core:
    - evm:0xAlice...
  extended:
    - evm:0xBob...
    - core              # includes all members of 'core'
```

You can also use `group:` prefix to be explicit: `- group:core`

### HTTP Resolver (Off-Chain)

Resolver groups are defined as a mapping (not a list). Membership is checked dynamically via an HTTP endpoint:

```yaml
groups:
  company:
    resolver: http
    url: https://api.example.com/groups/company
    cache_ttl: 60  # seconds (default: 300)
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
    function: isMember       # function(address) → bool (default: isMember)
    cache_ttl: 300           # seconds
```

The repo.box server proxies the call: `GET /resolve?chain=8453&contract=0x...&function=isMember&address=0x...`

**Note**: Resolver groups are resolver-only — they cannot also have static members or includes. If you need both static members and dynamic resolution, use two separate groups and reference both in your rules.

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
| `branch` | Branches | `git branch`, `git checkout -b` | Can create new branches |
| `create` | Files only | `git commit` (new file) | Can add new files |
| `delete` | Branches | `git branch -d` | Can delete the branch |
| `force-push` | Branches | `git push --force` | Can force-push to the branch |
| `write` | Files | `git commit` | General file write (currently not auto-classified; use `create` for new files) |
| `append` | Files | `git commit` | Can add lines to files (no deletions) |
| `edit` | Files | `git commit` | Can modify or delete content (superset of `write` and `append`) |

**Verb hierarchy**: `edit` is the general file-modification verb. `create`, `write`, and `append` are specific sub-verbs. If you grant `edit`, it implicitly covers all sub-verbs. **But not the reverse** — granting `append` does NOT give `create`, `write`, or `edit` permission. The sub-verbs are independent of each other. Also: if `edit` is explicitly denied, it blocks all sub-verbs regardless of their individual permissions.

**Verb classification** (at commit time, client-side shim):
- New file (staged as `A`) → `create`
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
    - evm:0xCoder000000000000000000000000000000003
    - evm:0xTester00000000000000000000000000000004

  community:
    - founders                           # include founders group
    - agents                             # include agents group

  token-holders:
    resolver: onchain
    chain: 8453
    contract: "0xTokenContract000000000000000000000000005"
    function: balanceOf
    cache_ttl: 300

permissions:
  default: deny
  rules:
    # Anyone can clone
    - "* read >*"

    # Founders: full control, but no force-push to main
    - founders not force-push >main
    - founders own >main

    # Agents: work on feature branches, edit source + tests
    - agents push >feature/**
    - agents create >feature/**
    - agents delete >feature/**
    - agents edit ./src/** >feature/**
    - agents edit ./tests/** >feature/**
    - agents append ./CHANGELOG.md

    # Token holders: submit proposals on dedicated branches
    - token-holders push >proposal/**
    - token-holders create >proposal/**
    - token-holders edit ./proposals/** >proposal/**

    # Anyone can append to the guestbook
    - "* append ./guestbook.jsonl"
    - "* push >main"

# Optional: x402 paid access
x402:
  read_price: "1.00"                    # USDC
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

Without a `.repobox/config.yml`, the repo is public (anyone can clone) and only the owner (the address that made the first push) can push. Add a config to define granular permissions for other contributors.

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
