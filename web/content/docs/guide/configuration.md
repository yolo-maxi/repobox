---
title: "Configuration"
description: "How to write .repobox/config.yml files: groups, permissions, and rule formats."
---

# Configuration

All repo.box rules live in a single file: `.repobox/config.yml` at the repo root. It has two top-level keys: `groups` and `permissions`.

## Groups

Groups are named sets of EVM identities.

```yaml
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001   # Alice
    - evm:0xAAA0000000000000000000000000000000000002   # Bob
  agents:
    - evm:0xBBB0000000000000000000000000000000000001   # Claude
    - evm:0xBBB0000000000000000000000000000000000002   # Codex
  all-humans:
    - evm:0xDDD0000000000000000000000000000000000001   # External reviewer
    - founders                                          # includes all founders
```

- **Bare word** = group name
- **`evm:0x...`** = individual EVM identity (42 hex chars, checksummed)
- Groups can include other groups by bare name

## Rule Formats

Rules can be written in three equivalent formats. Mix freely.

### Format A: Flat list (one-liners)

```yaml
permissions:
  default: allow
  rules:
    - founders push >main
    - founders merge >main
    - agents not edit ./.repobox/config.yml
```

### Format B: Subject-grouped

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

### Format C: Verb-mapping

```yaml
permissions:
  default: allow
  rules:
    agents:
      push:
        - ">feature/**"
        - ">fix/**"
      append:
        - "./.repobox/config.yml"
```

## Rule Syntax

Each rule follows: `<subject> [not] <verb> <target>`

### Subjects
- `groupname` — references a group from `groups:`
- `evm:0x...` — individual identity

### Branch Verbs

- `push` — push commits to a branch
- `merge` — merge into a branch
- `create` — create a new branch
- `delete` — delete a branch
- `force-push` — rewrite history

### File Verbs

- `edit` — full modification (add, change, remove lines)
- `write` — add lines only, no deletions
- `append` — add lines at end of file only

Prefix with `not` to deny: `agents not merge >main`

### Targets

- `>main` — specific branch
- `>feature/**` — branch glob (recursive)
- `*` — all files or all branches
- `contracts/**` — file path glob
- `contracts/** >dev` — combined: file + branch (both must match)

## The Default Policy

```yaml
permissions:
  default: allow   # or "deny"
```

- **`allow`** (default if omitted) — anything without a matching rule is permitted
- **`deny`** — anything without a matching rule is blocked

## Implicit Deny

When rules exist for a target, identities not mentioned are denied. This is per-target, not global.

```yaml
rules:
  - founders edit .repobox/config.yml
```

- Only `.repobox/config.yml` is locked down
- Other files follow `default`
- Agents editing `src/app.rs` → ✅ (no rule covers it)
- Agents editing `.repobox/config.yml` → ❌ (rule exists, agent not matched)

### Common Mistake

```yaml
rules:
  - founders edit *        # Locks ALL files for non-founders
  - agents push >feature/**  # Agents can push but can't edit any files!
```

Fix: add file permissions for agents too:
```yaml
rules:
  - founders edit *
  - agents edit * >feature/**  # Agents can edit on feature branches
  - agents push >feature/**
```

## Evaluation

Given action (subject, verb, target):

1. Collect all rules for this verb whose target pattern matches
2. Zero rules match → use `default`
3. Walk rules top-to-bottom: first subject match wins
4. No subject match → **deny**

**Order matters.** Put deny rules before allow rules:

```yaml
# ✅ Correct
- agents not push >main
- agents push >*

# ❌ Wrong: push >* matches first
- agents push >*
- agents not push >main
```

## What the File Does NOT Contain

- No aliases (local in `~/.repobox/aliases`)
- No key material (lives in `~/.repobox/keys/`)
- No identity setting (`~/.repobox/identity`)
- No remote/server config
