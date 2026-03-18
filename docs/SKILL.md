# repo.box — SKILL.md

> Git, but with permissions. For AI agents.

## What is repo.box?

repo.box is a git permission layer. It shims the `git` command so agents think they're using normal git, but every commit, merge, and push is checked against a `.repobox-config` file. Permissions are tied to EVM wallet identities.

Five agents, five keys, one repo, zero risk.

## Quick Start

### 1. Initialize a repo

```bash
cd my-repo
git repobox init
```

This sets up `gpg.program = repobox` in your git config and creates a `.repobox-config` template.

### 2. Generate your identity

```bash
git repobox keys generate
# → Created: ~/.repobox/keys/0xAAA...123.key
# → Address: evm:0xAAA...123

git config --local user.signingkey evm:0xAAA...123
```

### 3. Configure permissions

Edit `.repobox-config`:

```yaml
groups:
  founders:
    members:
      - evm:0xAAA...123

  agents:
    members: []

permissions:
  style: subject-first
  rules:
    - @founders edit *
    - @founders merge >main
    - @founders edit .repobox-config
    - @agents push >feature/*
    - @agents create >feature/*
    - @agents append .repobox-config
```

### 4. Commit

```bash
git add .repobox-config
git commit -m "initialize repo.box permissions"
```

Your commit is now signed with your EVM key.

## Aliases (Local Address Book)

Raw EVM addresses are unreadable. Use aliases to give them names:

```bash
git repobox alias add claude evm:0xBBB...456
git repobox alias list
git repobox whoami
# → @alice (evm:0xAAA...123)
```

Aliases are stored in `~/.repobox/aliases` — local to your machine, not part of the repo. Each collaborator can have their own names for the same addresses.

The CLI shows aliases everywhere:
```
❌ permission denied: @claude cannot edit .repobox-config
```

You can also set an alias when creating your identity:
```bash
git repobox identity set <private-key> --alias alice
```

## Onboarding an Agent

```bash
# Generate a key for the agent
git repobox keys generate
# → evm:0xBBB...456

# Give it a name
git repobox alias add claude evm:0xBBB...456

# Add to .repobox-config under agents.members:
#   - evm:0xBBB...456

# Commit the change
git add .repobox-config
git commit -m "onboard @claude"

# Spawn the agent with its identity
GIT_CONFIG_COUNT=1 \
GIT_CONFIG_KEY_0=user.signingkey \
GIT_CONFIG_VALUE_0=evm:0xBBB...456 \
your-agent-command --task "fix the auth bug"
```

The agent uses normal git commands. The shim handles permission checks and signing transparently.

## Agent Spawning Sub-Agents

Agents can onboard sub-agents on feature branches. Sub-agents use **plus notation** names: `@parent+childname` with a random adjective-animal pattern.

```bash
# Agent (@claude) is on feature/big-refactor
git repobox keys generate --alias claude+roudy-piglet
# → @claude+roudy-piglet (evm:0xCCC...789)

# Agent appends a direct permission rule to .repobox-config:
#   evm:0xCCC...789 write >feature/big-refactor/*

git add .repobox-config
git commit -m "onboard @claude+roudy-piglet for refactor"

# Spawn the sub-agent with its identity
GIT_CONFIG_COUNT=1 \
GIT_CONFIG_KEY_0=user.signingkey \
GIT_CONFIG_VALUE_0=evm:0xCCC...789 \
sub-agent-command --task "refactor the database layer"
```

The plus notation makes lineage visible in the audit trail:
```
commit f3e2d1c
EVM-signed by @claude+roudy-piglet (evm:0xCCC...789)

    refactor database layer
```

When done, revert `.repobox-config` changes before merging to main:

```bash
git checkout main -- .repobox-config
git commit -m "revert .repobox-config changes"
git checkout main
git merge feature/big-refactor
```

If you forget to revert, the merge gate catches it:
```
❌ Blocked: merge contains .repobox-config changes.
   You don't have permission to edit .repobox-config on >main.
```

## Permission Rules

### Verbs

| Verb | Meaning |
|------|---------|
| `push` | Push commits to a branch |
| `merge` | Merge another branch into this one |
| `create` | Create a new branch matching a pattern |
| `delete` | Delete a branch |
| `edit` | Full file modification (add, change, remove lines) |
| `write` | Add lines to a file (no deletions) |
| `append` | Add lines at end of file only |
| `force-push` | Force-push to a branch |

### Targets

- `>main` — branch named "main"
- `>feature/*` — any branch matching the glob
- `contracts/**` — file path glob
- `contracts/** >dev` — file path + branch combo

### Deny Rules

```yaml
- @agents not merge >main
- evm:0xBob not edit contracts/**
```

`not` explicitly denies. Overrides any allow rule for that identity.

### Priority

Rules are evaluated top-to-bottom. First match wins. Founders write rules at the top (highest priority). Agents can only append at the bottom (lowest priority).

## Which .repobox-config applies?

| Operation | Which .repobox-config? |
|-----------|-------------------|
| `git commit` | Current branch |
| `git push` | Current branch |
| `git merge X into Y` | **Y** (target branch — the one you're on) |

Feature branches are sandboxes. Protected branches are the trust boundary.

## Identity

Identity = `user.signingkey` in git config. Standard git mechanisms for switching:

```bash
# Per-repo
git config --local user.signingkey evm:0xAAA

# Per-command
git -c user.signingkey=evm:0xBBB commit -m "fix"

# Per-process (for agents)
GIT_CONFIG_COUNT=1 \
GIT_CONFIG_KEY_0=user.signingkey \
GIT_CONFIG_VALUE_0=evm:0xCCC \
agent-command
```

## Verification

```bash
# Verify a commit's signature
git verify-commit HEAD

# See who signed each commit
git log --show-signature

# Check if an identity can do something
git repobox check evm:0xAAA push >main
# → ✅ allowed / ❌ denied
```

## Passthrough Commands

These git commands are never intercepted (no permission checks):

`git status`, `git log`, `git diff`, `git add`, `git stash`, `git fetch`, `git clone`, `git remote`, and any unknown subcommand.
