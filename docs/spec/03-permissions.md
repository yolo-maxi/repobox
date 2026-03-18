# repo.box Spec: Permissions

## Overview

Permissions control what identities and groups can do — which branches they can push to, which files they can modify, and how. Rules are defined in `.repobox-config` (YAML), enforced both **locally** (by the `repobox` CLI) and **server-side** (on push). No bypass possible.

## Core Principles

- **Enforced everywhere.** The git shim enforces permissions locally. An agent cannot merge to main on its own machine. The server also rejects non-compliant pushes as defense in depth.
- **No bypass.** Git hooks can be skipped with `--no-verify`. The shim cannot — it IS the `git` command in the agent's environment.
- **Implicit deny per target.** If any rule allows a subject for a specific verb+target, other identities are implicitly denied **for that target**. Targets not mentioned by any rule remain open (or closed, depending on `default`).
- **Top-to-bottom priority.** First matching rule wins. Higher position = higher authority.
- **No overrides.** There is no sudo, no admin escape hatch, no emergency bypass.

## Two Types of Checks

Every git operation may trigger **two independent checks**:

1. **Branch check** — can you perform this branch operation? (`push`, `merge`, `create`, `delete`, `force-push`)
2. **File check** — can you modify these files? (`edit`, `write`, `append`)

**Both must pass.** Having `push` permission on a branch doesn't automatically grant `edit` on every file. But if no `edit` rules exist at all, file editing is unrestricted (with `default: allow`).

This means a minimal config can just use branch rules and skip file rules entirely:

```yaml
permissions:
  default: allow
  rules:
    - @founders push >*
    - @founders merge >*
    - @agents push >feature/**
    - @agents create >feature/**
```

No `edit`/`write`/`append` rules → no file restrictions. Anyone who can push can modify any file. Add file rules only when you need file-level control.

## Rule Syntax

Rules can be written in two forms: **flat** (one-liners) and **nested** (grouped by subject).

### Flat rules

```
<subject> <verb> <target>
```

Examples:
```yaml
rules:
  - @founders edit *
  - @founders push >*
  - @agents not merge >main
  - evm:0xBBB...456 push >feature/**
```

### Nested rules

Group multiple verbs and targets under a subject to avoid repetition:

```yaml
rules:
  - @agents:
      push:
        - >feature/**
        - >fix/**
        - >chore/**
      create:
        - >feature/**
        - >fix/**
        - >chore/**
      merge:
        - >chore/**
      append:
        - .repobox-config
```

**Both forms are equivalent.** Use flat for simple rules, nested when a subject has many verb/target combinations. They can be mixed in the same `rules:` list.

### Subjects

- `@groupname` — group reference (defined in `groups:` section)
- `evm:0x...` — individual identity

### Verbs

**Branch verbs** (control branch operations):

| Verb | Meaning |
|------|---------|
| `push` | Push commits to a branch |
| `merge` | Merge into a branch |
| `create` | Create a new branch matching pattern |
| `delete` | Delete a branch |
| `force-push` | Rewrite history (force push) |

**File verbs** (control file modifications):

| Verb | Meaning |
|------|---------|
| `edit` | Full file modification — add, change, or remove lines |
| `write` | Add lines only — no deletions allowed |
| `append` | Add lines strictly at the end of the file only |

Prefix any verb with `not` to deny: `@agents not merge >main`

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
- `*` — all files / all branches (context-dependent: `push >*` = all branches, `edit *` = all files)
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

### Minimal (branch control only, no file restrictions)

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
    - @founders push >*
    - @founders merge >*
    - @founders create >*
    - @agents:
        push:
          - >feature/**
          - >fix/**
        create:
          - >feature/**
          - >fix/**
```

Agents can push and create feature/fix branches, edit any files on those branches. Only founders can push/merge/create on main and other branches.

### With file restrictions

```yaml
permissions:
  default: allow
  rules:
    - @founders push >*
    - @founders merge >*
    - @founders create >*
    - @founders edit .repobox-config
    - @agents:
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

Now agents can edit files, but only on feature/fix branches. And `.repobox-config` is locked to founders for full edits — agents can only append to it.

### The `default` field

- **`allow`** (default if omitted) — if no rule covers a verb+target combination at all, the action is permitted
- **`deny`** — if no rule covers a verb+target combination at all, the action is denied

**Important:** `default` only matters for verb+target combinations with **zero** matching rules. The moment any rule mentions a verb for a target, implicit deny handles identities not covered by that rule.

## How Implicit Deny Works

This is the most important concept to understand.

**Implicit deny is scoped to the target, not the verb globally.**

When you write `@founders edit .repobox-config`, the system learns: "someone has explicit edit access to `.repobox-config`." Any identity NOT matched by an `edit` rule for `.repobox-config` is implicitly denied. But files NOT mentioned by any `edit` rule are unaffected — they follow `default`.

### Example: selective file protection

```yaml
permissions:
  default: allow
  rules:
    - @founders edit .repobox-config
```

| Action | Result | Why |
|--------|--------|-----|
| @founders edit .repobox-config | ✅ permit | Rule matches |
| @agents edit .repobox-config | ❌ deny | Implicit deny: rule exists for `edit .repobox-config`, @agents not matched |
| @agents edit src/app.rs | ✅ permit | No `edit` rule mentions `src/app.rs` → default: allow |
| @agents edit package.json | ✅ permit | No `edit` rule mentions `package.json` → default: allow |

Only `.repobox-config` is protected. Everything else is open.

### Example: broad file lockdown

```yaml
permissions:
  default: allow
  rules:
    - @founders edit *
    - @agents edit * >feature/**
```

| Action | Result | Why |
|--------|--------|-----|
| @founders edit anything | ✅ permit | `@founders edit *` matches all files |
| @agents edit src/app.rs on feature/fix | ✅ permit | `@agents edit * >feature/**` matches |
| @agents edit src/app.rs on main | ❌ deny | `edit *` has rules, @agents only matched on >feature/** → implicit deny on main |

Here `@founders edit *` covers all files, so implicit deny applies to all files for non-founders. But `@agents edit * >feature/**` carves out an exception for agents on feature branches.

## Evaluation Algorithm

Given an action (subject, verb, target):

1. Collect all rules for this verb whose **target pattern** matches the actual target
2. If **zero rules** match this verb+target:
   - `default: allow` → **permit**
   - `default: deny` → **deny**
3. If matching rules exist, walk them **top-to-bottom**:
   - For each rule, check if subject matches
   - First subject match wins:
     - Allow rule → **permit**
     - `not` (deny) rule → **deny**
4. If no rule matched the subject but rules exist for this verb+target → **deny** (implicit deny)

### Deny ordering

Explicit `not` rules must come **before** broader allows to take effect:

```yaml
rules:
  # ✅ Correct: deny first, then allow
  - @agents not push >main
  - @agents push >*

  # ❌ Wrong: push >* matches first, deny never reached
  - @agents push >*
  - @agents not push >main
```

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
# ❌ denied — implicit deny (rules exist for 'push >main', no match for this identity)

git repobox check evm:0xBBB...456 push >feature/fix
# ✅ allowed — rule: @agents push >feature/**
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
