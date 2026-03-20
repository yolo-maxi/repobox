---
title: "CLI Reference"
description: "All repobox CLI commands: setup, keys, aliases, verification, and agent spawning."
---

# CLI Reference

All commands run as `git repobox <command>`.

## Setup

```bash
# Initialize repo.box in an existing git repo
git repobox init
# → Creates .repobox/config.yml template
# → Sets gpg.program = repobox in git config

# Check current identity
git repobox whoami
# → alice (evm:0xAAA...123)
```

## Keys

```bash
# Generate a new EVM key pair
git repobox keys generate
# → Created: ~/.repobox/keys/0xAAA...123.key
# → Address: evm:0xAAA...123

# Generate with alias
git repobox keys generate --alias alice

# Import an existing private key
git repobox keys import <private-key>

# Set your identity (which key to sign with)
git repobox identity set <private-key>
git repobox identity set <private-key> --alias alice
```

Key storage:
- Private keys: `~/.repobox/keys/<address>.key`
- Identity: `~/.repobox/identity`

Identity precedence (standard git config resolution):
1. `git -c user.signingkey=evm:0x...` (per-command)
2. `GIT_CONFIG_*` env vars (per-process, used for agents)
3. `.git/config` (per-repo)
4. `~/.gitconfig` (global)

## Aliases

Local address book stored in `~/.repobox/aliases`.

```bash
git repobox alias add claude evm:0xBBB...456
git repobox alias remove claude
git repobox alias list
```

Aliases resolve everywhere in CLI output:
```
❌ claude cannot edit .repobox/config.yml on >main
```

## Spawning Agents

```bash
# Generate a key for the agent
git repobox keys generate --alias claude

# Add to .repobox/config.yml groups, then spawn:
GIT_CONFIG_COUNT=1 \
GIT_CONFIG_KEY_0=user.signingkey \
GIT_CONFIG_VALUE_0=evm:0xBBB...456 \
your-agent-command --task "fix the auth bug"
```

Sub-agents use `+` notation:
```bash
git repobox keys generate --alias claude+roudy-piglet
```

## Verification & Debugging

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

## Passthrough Commands

These git commands have no permission checks:

`git status`, `git log`, `git diff`, `git add`, `git stash`, `git fetch`, `git clone`, `git remote`
