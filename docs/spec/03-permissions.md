# repo.box Spec: Permissions

## Overview

Permissions control what identities and groups can do — which branches they can push to, which files they can modify, and how. Rules are defined in `.repobox-config` (YAML), enforced both **locally** (by the `repobox` CLI) and **server-side** (on push). No bypass possible.

## Core Principles

- **Enforced everywhere.** The git shim enforces permissions locally. An agent cannot merge to main on its own machine. The server also rejects non-compliant pushes as defense in depth.
- **No bypass.** Git hooks can be skipped with `--no-verify`. The shim cannot — it IS the `git` command in the agent's environment.
- **Implicit deny.** If any rule exists for a verb, identities not matched by any rule for that verb are denied.
- **Top-to-bottom priority.** First matching rule wins. Higher position = higher authority.
- **No overrides.** There is no sudo, no admin escape hatch, no emergency bypass.

## Rule Syntax

Every rule is a single line in `Subject Verb Object` order:

```
<subject> <verb> <target>
```

### Subjects

- `@groupname` — group reference (defined in `groups:` section)
- `evm:0x...` — individual identity

### Verbs

| Verb | Scope | Meaning |
|------|-------|---------|
| `push` | branch | Push commits to a branch |
| `merge` | branch | Merge into a branch |
| `create` | branch | Create a new branch matching pattern |
| `delete` | branch | Delete a branch |
| `edit` | path | Full file modification — add, change, or remove lines |
| `write` | path | Add lines only — no deletions allowed |
| `append` | path | Add lines strictly at the end of the file only |
| `force-push` | branch | Rewrite history (force push) |

Prefix with `not` to deny: `@agents not merge >main`

#### `edit` vs `write` vs `append`

Three levels of file modification, each a subset of the previous:

- **`edit`**: Full power. Add, modify, delete any line.
- **`write`**: Can add new lines anywhere, but cannot remove or modify existing lines. Diff may only contain `+` lines, no `-` lines.
- **`append`**: Can only add lines at the end of the file. Preserves the entire existing file and only extends it.

The shim validates at commit time by inspecting the diff:
- `write`: rejects any diff hunk containing `-` lines for that file
- `append`: rejects if any `+` lines appear before the last existing line of the file

### Targets

- `>main` — branch named "main"
- `>feature/*` — branches matching glob (one level)
- `>feature/**` — branches matching glob (recursive)
- `*` — all files / all branches (context-dependent)
- `contracts/**` — file path glob (recursive)
- `package.json` — specific file
- `contracts/** >dev` — combined: file path + branch (both must match)

### Combined Targets (Path + Branch)

A target can combine a file path and branch:

```
@devs write contracts/** >feature/*
```

This means: "@devs can write to files matching `contracts/**` on branches matching `feature/*`." Both the path and branch must match for the rule to apply.

If only a branch is specified (`>main`), the rule applies to all files on that branch.
If only a path is specified (`contracts/**`), the rule applies on all branches.

## Config Format

```yaml
groups:
  founders:
    members:
      - evm:0xAAA...123
  agents:
    members:
      - evm:0xBBB...456

permissions:
  default: allow
  rules:
    - @founders edit *
    - @founders push >*
    - @founders merge >*
    - @founders create >*
    - @agents push >feature/**
    - @agents create >feature/**
    - @agents merge >feature/**
    - @agents append .repobox-config
```

### The `default` field

- **`allow`** (default if omitted) — if no rule covers a verb at all, the action is permitted
- **`deny`** — if no rule covers a verb at all, the action is denied

Note: `default` only matters for verbs with **zero** rules. The moment you write even one rule for a verb, implicit deny handles the rest.

## Evaluation Algorithm

Given an action (subject, verb, target):

1. Collect all rules matching the verb (including `not` variants)
2. If **zero rules** exist for this verb:
   - `default: allow` → **permit**
   - `default: deny` → **deny**
3. If rules exist, walk them **top-to-bottom**:
   - For each rule, check if subject AND target match
   - First match wins:
     - Allow rule → **permit**
     - `not` (deny) rule → **deny**
4. If no rule matched but rules exist for this verb → **deny** (implicit deny)

### Examples

Config:
```yaml
permissions:
  default: allow
  rules:
    - @founders merge >main
```

| Action | Result | Why |
|--------|--------|-----|
| @founders merge >main | ✅ permit | Rule 1 matches |
| @agents merge >main | ❌ deny | Rules exist for `merge`, no match → implicit deny |
| @agents push >feature/x | ✅ permit | No rules for `push` at all → default: allow |

Config:
```yaml
permissions:
  default: allow
  rules:
    - @founders edit *
    - @founders push >*
    - @founders merge >*
    - @founders create >*
    - @agents push >feature/**
    - @agents create >feature/**
    - @agents merge >feature/**
```

| Action | Result | Why |
|--------|--------|-----|
| @founders merge >main | ✅ permit | Rule 3 matches |
| @agents create >feature/fix | ✅ permit | Rule 6 matches |
| @agents push >main | ❌ deny | Rules exist for `push`, none match @agents on >main → implicit deny |
| @agents delete >feature/fix | ❌ deny | Rules exist for... wait, no rules for `delete` → default: allow |

That last case reveals an important point: if you want to lock down `delete`, you need at least one rule for it. With `default: allow`, unmentioned verbs are open. With `default: deny`, they'd be closed.

### Explicit deny (`not`) vs implicit deny

- **Implicit deny**: "No rule matched you, but rules exist for this verb" → denied
- **Explicit deny**: `@agents not merge >main` → denied by a specific rule

Explicit deny is useful when you want to override a broader allow:

```yaml
rules:
  - @agents push >*
  - @agents not push >main
```

Here `@agents push >*` would allow pushing to main, but `@agents not push >main` is more specific. Wait — rules are top-to-bottom, so `push >*` would match first. To make this work, put the deny **above** the allow:

```yaml
rules:
  - @agents not push >main
  - @agents push >*
```

Now agents can push to any branch except main.

## Priority Model

**Top-to-bottom, first match wins.** Rules higher in the file have higher priority.

### Why this works safely

- **Founders** have full `edit` access to `.repobox-config`. They write the top rules — highest priority.
- **Agents** can only `append` to `.repobox-config`. Their rules land at the bottom — lowest priority. They can never override founder rules.

**Append-only + top-wins = permission escalation is structurally impossible.**

## Branch-Scoped Agent Permissions

Agents can grant sub-agents temporary access on feature branches:

1. Agent on `feature/fix` generates a key for a sub-agent
2. Agent appends a direct permission rule to `.repobox-config`:
   `evm:0xSub write >feature/fix/*`
3. Commits the change → ✅ (has append permission on feature branch)
4. Sub-agent works within its scope
5. Work done. Agent reverts `.repobox-config` changes.
6. Merges clean code to `main` → ✅ (no `.repobox-config` changes in diff)

If the agent forgets to revert:
```
❌ Blocked: merge contains .repobox-config changes.
   @claude cannot edit .repobox-config on >main.
```

Permissions are branch-scoped: sub-agent access exists only while the feature branch exists.

### Permission smuggling prevention

The server evaluates permissions from the `.repobox-config` on the **target branch**, not incoming changes. Modifying `.repobox-config` to grant yourself access doesn't work — the pre-push version is used for evaluation.

### Which .repobox-config applies?

| Operation | Which .repobox-config? |
|-----------|------------------------|
| `git commit` | Current branch |
| `git push` | Current branch |
| `git merge X into Y` | **Y** (target branch) |

## Enforcement Layers

### Layer 1: Git shim (local)

- Intercepts `git commit`, `git merge`, `git push`, `git checkout -b`, `git branch`
- Validates every commit against `.repobox-config` before delegating to real git
- Validates `write`/`append` constraints by inspecting the diff
- **Cannot be bypassed** — it IS the `git` command in the agent's environment
- Read-only commands (`git status`, `git log`, `git diff`, etc.) pass through unchanged

### Layer 2: Server (remote, post-hackathon)

- Validates every push against `.repobox-config` on the target branch
- Defense in depth — catches raw `git` usage that bypasses the shim

## Tooling

### `git repobox check <identity> <verb> <target>`

Answers "can this identity do this?" against the current `.repobox-config`. Shows which rule matched and why.

```bash
git repobox check evm:0xBBB...456 push >main
# ❌ denied — implicit deny (rules exist for 'push', no match for this identity on >main)

git repobox check evm:0xBBB...456 push >feature/fix
# ✅ allowed — rule 5: @agents push >feature/**
```

### `git repobox lint`

- Validates `.repobox-config` syntax
- Detects undefined group references
- Warns about ordering issues (allow below deny for same target)
- Detects unreachable rules (shadowed by earlier broader rules)

### `git repobox diff`

Shows permission changes between two `.repobox-config` versions. Used in code review.

## What Permissions Does NOT Cover

- **Who is in a group** — See 02-groups.md
- **PR/merge workflows** — See 04-workflows.md
- **Read access** — Future consideration
- **Tag permissions** — Future addition
