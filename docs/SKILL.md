# repo.box — Permission System Reference

> Canonical reference for the repo.box permission model. Read this before writing or reviewing `.repobox/config.yml` files.


## What is repo.box?

repo.box makes git repositories safe for AI agents. It shims the `git` command so agents use normal git workflows — but every commit, merge, and push is silently checked against a `.repobox/config.yml` file before it lands. If the action violates a rule, it's blocked before anything touches your repo.

Each agent gets its own EVM keypair as identity. Each commit is signed. Permissions live in the repo, not on a server. One YAML file controls who can push where, which files they can touch, and which branches they can create — with first-match-wins evaluation and implicit deny on any target that has rules.

Five agents, five keys, one repo, zero risk.

## Config Structure

Rules can be written in **three formats**. All are equivalent — mix freely.

### Format A: Flat list (one-liners)

```yaml
permissions:
  default: allow
  rules:
    - founders push >main
    - founders merge >main
    - agents not edit ./.repobox/config.yml
```

### Format B: Subject-grouped (subject → list of "verb target" strings)

```yaml
permissions:
  default: allow
  rules:
    founders:
      - push >main
      - merge >main
    agents:
      - not edit ./.repobox/config.yml
```

### Format C: Verb-mapping (subject → verb → targets)

```yaml
permissions:
  default: allow
  rules:
    founders:
      push:
        - ">main"
      merge:
        - ">main"
    agents:
      push:
        - ">feature/**"
        - ">fix/**"
      create:
        - ">feature/**"
        - ">fix/**"
      append:
        - "./.repobox/config.yml"
```

### Mixing formats

In Format A (list), each entry can be a flat string OR a nested mapping (C-style):

```yaml
rules:
  - founders push >*
  - founders merge >*
  - agents:
      push:
        - ">feature/**"
      append:
        - "./.repobox/config.yml"
```

Formats B and C use a top-level mapping for `rules:` (subjects as keys). Write it however feels natural.

## Complete `.repobox/config.yml` Reference

The config file is YAML with exactly two top-level keys: `groups` and `permissions`. Nothing else.

### Full annotated example (every possible feature used)

```yaml
# ═══════════════════════════════════════════════════════
# GROUPS — define named sets of EVM identities
# ═══════════════════════════════════════════════════════
groups:
  # Group names are bare words. Just list the addresses.
  founders:
    - evm:0xAAA0000000000000000000000000000000000001   # Alice
    - evm:0xAAA0000000000000000000000000000000000002   # Bob
  agents:
    - evm:0xBBB0000000000000000000000000000000000001   # Claude
    - evm:0xBBB0000000000000000000000000000000000002   # Codex
  deploy-bots:
    - evm:0xCCC0000000000000000000000000000000000001   # CI runner
  # Include another group's members by bare name:
  all-humans:
    - evm:0xDDD0000000000000000000000000000000000001   # External reviewer
    - founders                                          # includes all founders too

# ═══════════════════════════════════════════════════════
# PERMISSIONS — who can do what, where
# ═══════════════════════════════════════════════════════
permissions:
  # DEFAULT POLICY: what happens when NO rules match a verb+target.
  # "allow" (default if omitted) = open by default, rules restrict.
  # "deny" = closed by default, rules grant access.
  default: allow

  # RULES: evaluated top-to-bottom, first match wins.
  rules:
    # ─── FLAT RULES (one-liners) ─────────────────────
    # Format: <subject> [not] <verb> <target>
    #
    # Subject: group name (bare word) or evm:0x... (address)
    # Verb: push | merge | create | delete | force-push | edit | write | append
    # Target: >branch (branches), ./path (files), or * (all)
    #         Combine both: ./path >branch

    # Branch control — who can do branch operations
    - founders push >*                     # founders push to any branch
    - founders merge >*                    # founders merge into any branch
    - founders create >*                   # founders create any branch
    - founders delete >*                   # founders delete any branch
    - founders force-push >*               # founders rewrite history anywhere

    # File control — who can modify which files
    - founders edit ./.repobox/config.yml      # only founders can fully edit config
    - founders edit *                      # founders can edit all files

    # Deny rules — prefix verb with "not"
    - agents not push >main                # agents cannot push to main
    - agents not merge >main               # agents cannot merge into main

    # Individual identity rule (not a group)
    - evm:0xDDD0000000000000000000000000000000000001 push >hotfix/**

    # ─── NESTED RULES (grouped by subject) ───────────
    # Exactly equivalent to flat rules, just organized differently.
    # Useful when one subject has many verb+target combinations.
    - agents:
        push:
          - ">feature/**"                   # agents push to feature branches
          - ">fix/**"                       # agents push to fix branches
        create:
          - ">feature/**"                   # agents create feature branches
          - ">fix/**"
        edit:
          - "./* >feature/**"               # agents edit any file ON feature branches
          - "./* >fix/**"                   # agents edit any file ON fix branches
        append:
          - "./.repobox/config.yml"             # agents can append to config (not full edit)

    - deploy-bots:
        push:
          - ">main"                         # deploy bot pushes to main
        edit:
          - "./CHANGELOG.md >main"          # but can only touch these files
          - "./k8s/** >main"
```

### Conventions

- **Bare word** = group name (self-defined in `groups:`)
- **`evm:0x...`** = EVM address (external identity)
- **`vitalik.eth`** = ENS name (auto-detected by `.eth`/`.box`/`.xyz`/etc. suffix)
- **`ens:vitalik.eth`** = explicit ENS prefix (equivalent to bare form above)
- **`./path`** = file path (`./` is optional but recommended for clarity)
- **`>branch`** = branch target
- **`*`** = wildcard (matches all files or all branches)

### Identity formats

Three ways to specify an identity — all equivalent:

| Format | Example | Notes |
|--------|---------|-------|
| `evm:0x...` | `evm:0xAAA...123` | EVM address (42-char hex with checksum) |
| `name.eth` | `vitalik.eth` | ENS name — auto-detected from TLD suffix |
| `ens:name.eth` | `ens:vitalik.eth` | Explicit ENS prefix (same as bare form) |

The parser auto-detects ENS names by their suffix (`.eth`, `.box`, `.com`, `.xyz`, `.org`, `.io`, `.dev`, `.app`). The `ens:` prefix is accepted but redundant — `vitalik.eth` and `ens:vitalik.eth` are identical internally.

ENS names are resolved to EVM addresses at evaluation time via on-chain resolution. This means:
- Groups can mix addresses and ENS names freely
- ENS names in rules work the same as addresses
- Resolution is cached with TTL (fail-closed: unresolvable name = denied)

```yaml
groups:
  founders:
    - vitalik.eth              # ENS name (auto-detected)
    - ens:nick.eth             # explicit prefix (equivalent)  
    - evm:0xAAA...123          # raw address

permissions:
  rules:
    - vitalik.eth push >main   # ENS name in flat rules works too
```

### Structure rules

- **`groups`** is optional (you can write rules with raw `evm:0x...` addresses only)
- **`permissions`** is required
- **`permissions.default`** is optional (defaults to `allow`)
- **`permissions.rules`** is a list. Each entry is either:
  - A **string** (flat rule): `"founders push >*"`
  - A **mapping** (nested rule): `{ "agents": { push: [">feature/**"] } }`
- Both flat and nested can be mixed freely in the same list
- Groups can contain `evm:0x...` addresses, ENS names, and/or bare group names (for includes)
- `./` prefix on file paths is stripped by the parser (purely visual)
- `**` in globs means recursive: `./src/**` matches `src/a.rs` and `src/deep/b.rs`
- Combined targets: `./contracts/** >dev` means "files matching `contracts/**` on branch `dev`"

### What the file does NOT contain

- No aliases (those are local in `~/.repobox/aliases`)
- No key material (keys live in `~/.repobox/keys/`)
- No identity setting (that's `~/.repobox/identity`)
- No remote/server configuration (repo.box is local-only)

## Rule Syntax: `<subject> <verb> <target>`

### Subjects
- `groupname` — bare word, references a group from `groups:` section
- `evm:0x...` — individual EVM identity

### Branch Verbs
| Verb | Meaning |
|------|---------|
| `push` | Push commits to a branch |
| `merge` | Merge into a branch |
| `create` | Create a new branch |
| `delete` | Delete a branch |
| `force-push` | Rewrite history |

### File Verbs
| Verb | Meaning |
|------|---------|
| `edit` | Full modification (add, change, remove lines) |
| `write` | Add lines only, no deletions |
| `append` | Add lines at end of file only |

Prefix with `not` to deny: `agents not merge >main`

### Targets
- `>main` — specific branch
- `>feature/**` — branch glob (recursive)
- `*` — all files or all branches (context-dependent)
- `contracts/**` — file path glob
- `contracts/** >dev` — combined: file + branch (both must match)

## Two Independent Checks

Every git operation triggers up to two checks:

1. **Branch check** — can you do this branch operation? (push/merge/create/delete)
2. **File check** — can you modify these files? (edit/write/append)

**Both must pass.** But if no file rules exist at all, files are unrestricted (with `default: allow`).

## The `default` Field

- **`allow`** (default if omitted) — verb+target combinations with zero rules are permitted
- **`deny`** — verb+target combinations with zero rules are denied

## ⚠️ Implicit Deny (Critical Concept)

**Implicit deny is per-target, NOT per-verb globally.**

When you write `founders edit .repobox/config.yml`:
- Only `.repobox/config.yml` is locked down
- Other files are unaffected (follow `default`)
- agents editing `src/app.rs` → ✅ allowed (no rule covers it)
- agents editing `.repobox/config.yml` → ❌ denied (rule exists, agent not matched)

When you write `founders edit *`:
- ALL files are locked down (`*` matches everything)
- agents editing anything → ❌ denied (unless another rule grants them access)

### Common Mistake

```yaml
rules:
  - founders edit *        # ← This locks ALL files for non-founders
  - agents push >feature/**  # ← Agents can push but can't edit any files!
```

Fix: add file permissions for agents too:
```yaml
rules:
  - founders edit *
  - agents edit * >feature/**  # ← Agents can edit files on feature branches
  - agents push >feature/**
```

## Evaluation Algorithm

Given action (subject, verb, target):

1. Collect all rules for this verb whose **target pattern** matches the actual target
2. If **zero rules** match → use `default` (allow or deny)
3. If rules exist, walk **top-to-bottom**:
   - First rule where subject matches wins → allow or deny
4. If no rule matched the subject → **deny** (implicit deny)

### Order Matters

```yaml
# ✅ Correct: deny first
- agents not push >main
- agents push >*

# ❌ Wrong: push >* matches first, deny never reached
- agents push >*
- agents not push >main
```

## Aliases (Local Address Book)

Raw addresses are unreadable. Local aliases in `~/.repobox/aliases`:

```
alice = evm:0xAAA...123
claude = evm:0xBBB...456
```

- CLI commands: `git repobox alias add/remove/list`
- `git repobox keys generate --alias claude`
- `git repobox identity set <key> --alias alice`
- CLI shows `alice`, `claude` everywhere (errors, logs, whoami)
- Sub-agents use `+` notation: `claude+roudy-piglet`
- `.repobox/config.yml` always stores raw `evm:0x...` addresses (canonical)

## Priority Model

Top-to-bottom, first match wins. Safe because:
- **Founders** write rules at the top (full `edit` on `.repobox/config.yml`)
- **Agents** can only `append` (bottom of file = lowest priority)
- Append-only + top-wins = permission escalation is structurally impossible

## CLI Commands

### Setup

```bash
# Initialize repo.box in an existing git repo
git repobox init
# → Creates .repobox/config.yml template
# → Sets gpg.program = repobox in git config

# Generate a new EVM key pair
git repobox keys generate
# → Created: ~/.repobox/keys/0xAAA...123.key
# → Address: evm:0xAAA...123

# Generate key with alias in one step
git repobox keys generate --alias alice
# → alice (evm:0xAAA...123)

# Import an existing private key
git repobox keys import <private-key>

# Set your identity (which key to sign with)
git repobox identity set <private-key>
git repobox identity set <private-key> --alias alice

# Check current identity
git repobox whoami
# → alice (evm:0xAAA...123)
```

### Aliases

Stored in `~/.repobox/aliases` (local to your machine, not part of the repo).

```bash
git repobox alias add claude evm:0xBBB...456
git repobox alias remove claude
git repobox alias list
```

The CLI resolves aliases everywhere — errors, logs, signature display:
```
❌ claude cannot edit .repobox/config.yml on >main
```

If no alias exists, the raw `evm:0x...` address is shown.

### Spawning Agents

```bash
# Generate a key for the agent
git repobox keys generate --alias claude
# → claude (evm:0xBBB...456)

# Add to .repobox/config.yml groups + commit
# Then spawn with identity via env:
GIT_CONFIG_COUNT=1 \
GIT_CONFIG_KEY_0=user.signingkey \
GIT_CONFIG_VALUE_0=evm:0xBBB...456 \
your-agent-command --task "fix the auth bug"
```

Sub-agents use `+` notation: `claude+roudy-piglet`

```bash
git repobox keys generate --alias claude+roudy-piglet
```

### Verification & Debugging

```bash
# Check if an identity can do something
git repobox check evm:0xBBB...456 push >main
# ❌ denied — implicit deny

git repobox check evm:0xBBB...456 push >feature/fix
# ✅ allowed — rule: agents push >feature/**

# Validate .repobox/config.yml syntax
git repobox lint

# Show permission changes between versions
git repobox diff

# Verify commit signatures
git verify-commit HEAD
git log --show-signature
```

### Passthrough (never intercepted)

These git commands have no permission checks:
`git status`, `git log`, `git diff`, `git add`, `git stash`, `git fetch`, `git clone`, `git remote`

## Key Storage

- Private keys: `~/.repobox/keys/<address>.key`
- Identity: `~/.repobox/identity` (global, not per-repo)
- Aliases: `~/.repobox/aliases` (global, not per-repo)

Identity precedence (standard git config resolution):
1. `git -c user.signingkey=evm:0x...` (per-command)
2. `GIT_CONFIG_*` environment variables (per-process, used for agents)
3. `.git/config` (per-repo)
4. `~/.gitconfig` (global)

## Worked Examples

### Minimal: just protect main

```yaml
permissions:
  default: allow
  rules:
    - founders merge >main
```

- Only founders can merge to main
- Everything else is open (default: allow)
- Agents can push anywhere, edit anything

### Standard: branch control, no file restrictions

```yaml
permissions:
  default: allow
  rules:
    - founders push >*
    - founders merge >*
    - founders create >*
    - agents:
        push:
          - >feature/**
          - >fix/**
        create:
          - >feature/**
          - >fix/**
```

- Founders control all branches
- Agents work on feature/fix branches only
- No file restrictions (no edit rules)

### Strict: branch + file control

```yaml
permissions:
  default: allow
  rules:
    - founders push >*
    - founders merge >*
    - founders create >*
    - founders edit ./.repobox/config.yml
    - agents:
        push:
          - ">feature/**"
          - ">fix/**"
        create:
          - ">feature/**"
          - ">fix/**"
        edit:
          - "./* >feature/**"
          - "./* >fix/**"
        append:
          - "./.repobox/config.yml"
```

- Agents can edit files only on their branches
- `.repobox/config.yml` is founder-only for full edits, agents can append
- Agents can't edit anything on main even if they could somehow push

### File-scoped main access (CI bot)

```yaml
permissions:
  default: allow
  rules:
    - founders push >*
    - founders merge >*
    - deploy-bot:
        push:
          - ">main"
        edit:
          - "./CHANGELOG.md >main"
          - "./k8s/** >main"
```

- deploy-bot can push to main but only touch CHANGELOG.md and k8s/ files
- A commit touching anything else → blocked
