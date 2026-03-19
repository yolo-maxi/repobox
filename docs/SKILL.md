# repo.box — Permission System Reference

> Canonical reference for the repo.box permission model. Read this before writing or reviewing `.repobox-config` files.

## Source of Truth

- Full specs: `/home/xiko/repobox/docs/spec/`
- Dashboard: `/home/xiko/repobox-dashboard/`
- Tests: `/home/xiko/repobox-dashboard/data/tests.json`
- Stories: `/home/xiko/repobox-dashboard/data/stories.json`
- SKILL.md (for agents): `/home/xiko/repobox/docs/SKILL.md`
- Project brief: `/home/xiko/clawd/memory/projects/repobox-platform.md`

## What is repo.box?

A git permission layer. Shims the `git` command so every commit, merge, and push is checked against a `.repobox-config` file. Permissions are tied to EVM wallet identities. Five agents, five keys, one repo, zero risk.

## Config Structure

```yaml
groups:
  founders:
    members:
      - evm:0xAAA...123
  agents:
    members:
      - evm:0xBBB...456

permissions:
  default: allow    # or "deny"
  rules:
    # Flat rules (one-liners)
    - %founders push >*
    - %founders merge >*
    - %founders create >*
    - %founders edit .repobox-config

    # Nested rules (grouped by subject)
    - %agents:
        push:
          - >feature/**
          - >fix/**
        create:
          - >feature/**
          - >fix/**
        append:
          - .repobox-config
```

Both flat and nested are equivalent. Mix freely in the same `rules:` list.

## Complete `.repobox-config` Reference

The config file is YAML with exactly two top-level keys: `groups` and `permissions`. Nothing else.

### Full annotated example (every possible feature used)

```yaml
# ═══════════════════════════════════════════════════════
# GROUPS — define named sets of EVM identities
# ═══════════════════════════════════════════════════════
groups:
  # Group names are bare words (no prefix in the config file).
  # In rules, reference them as %founders, %agents, etc.
  founders:
    members:
      - evm:0xAAA0000000000000000000000000000000000001   # Alice
      - evm:0xAAA0000000000000000000000000000000000002   # Bob
  agents:
    members:
      - evm:0xBBB0000000000000000000000000000000000001   # Claude
      - evm:0xBBB0000000000000000000000000000000000002   # Codex
  deploy-bots:
    members:
      - evm:0xCCC0000000000000000000000000000000000001   # CI runner

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
    # Subject: %groupname or evm:0x...
    # Verb: push | merge | create | delete | force-push | edit | write | append
    # Target: >branchpattern (branches) or filepattern (files)
    #         * means "all" for either context
    #         Combine both: filepattern >branchpattern

    # Branch control — who can do branch operations
    - %founders push >*                     # founders push to any branch
    - %founders merge >*                    # founders merge into any branch
    - %founders create >*                   # founders create any branch
    - %founders delete >*                   # founders delete any branch
    - %founders force-push >*               # founders rewrite history anywhere

    # File control — who can modify which files
    - %founders edit .repobox-config        # only founders can fully edit config
    - %founders edit *                      # founders can edit all files

    # Deny rules — prefix verb with "not"
    - %agents not push >main               # agents cannot push to main
    - %agents not merge >main              # agents cannot merge into main

    # Individual identity rule (not a group)
    - evm:0xDDD0000000000000000000000000000000000001 push >hotfix/**

    # ─── NESTED RULES (grouped by subject) ───────────
    # Exactly equivalent to flat rules, just organized differently.
    # Useful when one subject has many verb+target combinations.
    - %agents:
        push:
          - ">feature/**"                   # agents push to feature branches
          - ">fix/**"                       # agents push to fix branches
        create:
          - ">feature/**"                   # agents create feature branches
          - ">fix/**"
        edit:
          - "* >feature/**"                 # agents edit any file ON feature branches
          - "* >fix/**"                     # agents edit any file ON fix branches
        append:
          - .repobox-config                 # agents can append to config (not full edit)

    - %deploy-bots:
        push:
          - ">main"                         # deploy bot pushes to main
        edit:
          - "CHANGELOG.md >main"            # but can only touch these files
          - "k8s/** >main"
```

### Structure rules

- **`groups`** is optional (you can write rules with raw `evm:0x...` addresses only)
- **`permissions`** is required
- **`permissions.default`** is optional (defaults to `allow`)
- **`permissions.rules`** is a list. Each entry is either:
  - A **string** (flat rule): `"%founders push >*"`
  - A **mapping** (nested rule): `{ "%agents": { push: [">feature/**"] } }`
- Both flat and nested can be mixed freely in the same list
- Group names in `groups:` are bare words. In rules, prefix with `%`
- Members are always full `evm:0x...` addresses (42 hex chars with checksum)
- Branch targets always start with `>`. File targets never do
- `*` alone means "all" (all branches or all files depending on context)
- `**` in globs means recursive: `src/**` matches `src/a.rs` and `src/deep/b.rs`
- Combined targets: `contracts/** >dev` means "files matching `contracts/**` on branch `dev`"

### What the file does NOT contain

- No aliases (those are local in `~/.repobox/aliases`)
- No key material (keys live in `~/.repobox/keys/`)
- No identity setting (that's `~/.repobox/identity`)
- No remote/server configuration (repo.box is local-only)

## Rule Syntax: `<subject> <verb> <target>`

### Subjects
- `%groupname` — group from `groups:` section
- `evm:0x...` — individual identity

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

Prefix with `not` to deny: `%agents not merge >main`

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

When you write `%founders edit .repobox-config`:
- Only `.repobox-config` is locked down
- Other files are unaffected (follow `default`)
- %agents editing `src/app.rs` → ✅ allowed (no rule covers it)
- %agents editing `.repobox-config` → ❌ denied (rule exists, agent not matched)

When you write `%founders edit *`:
- ALL files are locked down (`*` matches everything)
- %agents editing anything → ❌ denied (unless another rule grants them access)

### Common Mistake

```yaml
rules:
  - %founders edit *        # ← This locks ALL files for non-founders
  - %agents push >feature/**  # ← Agents can push but can't edit any files!
```

Fix: add file permissions for agents too:
```yaml
rules:
  - %founders edit *
  - %agents edit * >feature/**  # ← Agents can edit files on feature branches
  - %agents push >feature/**
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
- %agents not push >main
- %agents push >*

# ❌ Wrong: push >* matches first, deny never reached
- %agents push >*
- %agents not push >main
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
- CLI shows `%alice`, `%claude` everywhere (errors, logs, whoami)
- Sub-agents use `+` notation: `%claude+roudy-piglet`
- `.repobox-config` always stores raw `evm:0x...` addresses (canonical)

## Priority Model

Top-to-bottom, first match wins. Safe because:
- **Founders** write rules at the top (full `edit` on `.repobox-config`)
- **Agents** can only `append` (bottom of file = lowest priority)
- Append-only + top-wins = permission escalation is structurally impossible

## CLI Commands

### Setup

```bash
# Initialize repo.box in an existing git repo
git repobox init
# → Creates .repobox-config template
# → Sets gpg.program = repobox in git config

# Generate a new EVM key pair
git repobox keys generate
# → Created: ~/.repobox/keys/0xAAA...123.key
# → Address: evm:0xAAA...123

# Generate key with alias in one step
git repobox keys generate --alias alice
# → %alice (evm:0xAAA...123)

# Import an existing private key
git repobox keys import <private-key>

# Set your identity (which key to sign with)
git repobox identity set <private-key>
git repobox identity set <private-key> --alias alice

# Check current identity
git repobox whoami
# → %alice (evm:0xAAA...123)
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
❌ %claude cannot edit .repobox-config on >main
```

If no alias exists, the raw `evm:0x...` address is shown.

### Spawning Agents

```bash
# Generate a key for the agent
git repobox keys generate --alias claude
# → %claude (evm:0xBBB...456)

# Add to .repobox-config groups + commit
# Then spawn with identity via env:
GIT_CONFIG_COUNT=1 \
GIT_CONFIG_KEY_0=user.signingkey \
GIT_CONFIG_VALUE_0=evm:0xBBB...456 \
your-agent-command --task "fix the auth bug"
```

Sub-agents use `+` notation: `%claude+roudy-piglet`

```bash
git repobox keys generate --alias claude+roudy-piglet
```

### Verification & Debugging

```bash
# Check if an identity can do something
git repobox check evm:0xBBB...456 push >main
# ❌ denied — implicit deny

git repobox check evm:0xBBB...456 push >feature/fix
# ✅ allowed — rule: %agents push >feature/**

# Validate .repobox-config syntax
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
    - %founders merge >main
```

- Only founders can merge to main
- Everything else is open (default: allow)
- Agents can push anywhere, edit anything

### Standard: branch control, no file restrictions

```yaml
permissions:
  default: allow
  rules:
    - %founders push >*
    - %founders merge >*
    - %founders create >*
    - %agents:
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
    - %founders push >*
    - %founders merge >*
    - %founders create >*
    - %founders edit .repobox-config
    - %agents:
        push:
          - >feature/**
          - >fix/**
        create:
          - >feature/**
          - >fix/**
        edit:
          - * >feature/**
          - * >fix/**
        append:
          - .repobox-config
```

- Agents can edit files only on their branches
- `.repobox-config` is founder-only for full edits, agents can append
- Agents can't edit anything on main even if they could somehow push

### File-scoped main access (CI bot)

```yaml
permissions:
  default: allow
  rules:
    - %founders push >*
    - %founders merge >*
    - %deploy-bot:
        push:
          - >main
        edit:
          - CHANGELOG.md >main
          - k8s/** >main
```

- %deploy-bot can push to main but only touch CHANGELOG.md and k8s/ files
- A commit touching anything else → blocked
