# repo.box Spec: Permissions

> ⚠️ **NOTE**: The priority model (top-to-bottom vs deny-always-wins) may be revisited. Current decision is top-to-bottom with append-only as the safety mechanism, but this needs real-world validation.

## Overview

Permissions control what identities and groups can do — which branches they can push to, which files they can modify, and how. Rules are defined in `.boxconfig` (YAML), enforced both **locally** (by the `box` CLI) and **server-side** (on push). No bypass possible.

## Core Principles

- **Enforced everywhere.** Not just on the server. The git shim enforces permissions locally. An agent literally cannot merge to main on its own machine.
- **No bypass.** Git hooks can be skipped with `--no-verify`. The shim cannot — it IS the `git` command in the agent's environment. The server also rejects non-compliant pushes as defense in depth.
- **Implicit deny.** If a rule explicitly allows a subject, everyone else is implicitly denied for that target.
- **Top-to-bottom priority.** First matching rule wins. Higher position = higher authority. (See Priority Model section.)
- **No overrides.** There is no sudo, no admin escape hatch, no emergency bypass.

## Config Format

Permissions live in `.boxconfig` (YAML). One file per repo, at the root.

### Style Declaration

Each `.boxconfig` declares one permission style. All rules must follow that style.

```yaml
permissions:
  style: subject-first  # or object-first, verb-first
```

### Notation

- `@name` — group reference (defined in `groups:` section)
- `evm:0x...` — individual identity
- `>branch` — branch target (e.g. `>main`, `>feature/*`)
- bare string — file/path target (e.g. `package.json`, `contracts/**`)
- `not` prefix — deny rule
- Path globs: `*` matches within a directory, `**` matches recursively

## Verbs (Actions)

| Verb | Scope | Meaning |
|------|-------|---------|
| `push` | branch | Push commits to a branch |
| `merge` | branch | Merge into a branch |
| `create` | branch | Create a new branch matching pattern |
| `delete` | branch | Delete a branch |
| `edit` | path | Full file modification — add, change, or remove lines |
| `write` | path | Add lines only — no deletions allowed. Lines can be inserted anywhere. |
| `append` | path | Add lines strictly at the end of the file only |
| `force-push` | branch | Rewrite history (force push) |

### `edit` vs `write` vs `append`

Three levels of file modification, each a subset of the previous:

- **`edit`**: Full power. Add, modify, delete any line. Traditional write access.
- **`write`**: Can add new lines anywhere in the file, but cannot remove or modify existing lines. Diff may only contain `+` lines, no `-` lines. Useful for config files where you want contributors to add entries without changing existing ones.
- **`append`**: Can only add lines at the end of the file. Strictest — preserves the entire existing file and only extends it. Useful for logs, `.boxconfig` permission layering, and append-only data files.

The git shim validates these constraints at commit time by inspecting the diff:
- `write`: rejects any diff hunk containing `-` lines for that file
- `append`: rejects if any `+` lines appear before the last existing line of the file

## Permission Styles

All three styles express the same permission graph. A `.boxconfig` uses exactly one.

### Subject-first

Group rules by who.

```yaml
permissions:
  style: subject-first

  @founders:
    merge:
      - >main
    push:
      - >main
    write:
      - .boxconfig
      - .github/**
      - .env*

  @devs:
    push:
      - >develop
    merge:
      - >develop
    write:
      - package.json
      - contracts/**

  @agents:
    create:
      - >feature/*
      - >agent/*
    push:
      - >feature/*
      - >agent/*
    not write:
      - package.json
      - .boxconfig
      - contracts/**

  evm:0xBob...123:
    not write:
      - contracts/**
```

### Object-first

Group rules by what.

```yaml
permissions:
  style: object-first

  >main:
    push:
      - @founders
    merge:
      - @founders

  >feature/*:
    create:
      - @everyone
    push:
      - @everyone

  contracts/**:
    write:
      - @devs
      - @auditors
    not write:
      - evm:0xBob...123

  .boxconfig:
    write:
      - @founders
```

### Verb-first

Group rules by action.

```yaml
permissions:
  style: verb-first

  merge:
    >main:
      - @founders
    >develop:
      - @devs

  push:
    >main:
      - @founders
    >develop:
      - @devs
    >feature/*:
      - @everyone

  write:
    contracts/**:
      - @devs
      - @auditors
    .boxconfig:
      - @founders

  not write:
    contracts/**:
      - evm:0xBob...123
    package.json:
      - @agents
```

## Combined Targets (Path + Branch)

A rule can combine a path and branch target on the same line:

```yaml
@devs:
  not merge:
    - contracts/** >dev     # cannot merge contract changes into dev
  write:
    - contracts/** >feature/*  # can edit contracts on feature branches
```

When a target has both path and branch, **both must match** for the rule to apply. `contracts/** >dev` means "changes to files matching `contracts/**` being merged into branch `dev`."

If only a branch is specified (e.g. `>main`), the rule applies to all files on that branch. If only a path is specified (e.g. `contracts/**`), the rule applies on all branches.

## Priority Model

> ⚠️ This model may be revisited based on real-world usage.

**Top-to-bottom, first match wins.** Rules higher in the file have higher priority.

### Why this works safely

The priority model is safe because of **who can write where**:

- **Founders** have full `write` access to `.boxconfig`. They control the top of the file — the highest priority rules. They're trusted to get ordering right.
- **Devs** have `append` access to `.boxconfig` on feature branches. Their rules land at the bottom — lowest priority. They can never override founder rules.
- **Agents** can only append on their own branches. Lowest priority, most constrained.

A dev appending `@devs: merge: contracts/** >dev` at the bottom is useless if a higher `not merge` rule already blocks it.

**Append-only + top-wins = permission escalation is structurally impossible.**

### Evaluation algorithm

Given an action (subject, verb, path, branch):

1. Walk rules top-to-bottom
2. For each rule, check if subject, verb, path, and branch match
3. First matching rule determines outcome:
   - If it's an allow rule → **permit**
   - If it's a `not` (deny) rule → **deny**
4. If no rule matches:
   - If ANY explicit allow exists for this verb+target → **deny** (implicit deny)
   - If no rules exist for this verb+target → **permit** (unrestricted)

### Implicit deny in detail

The moment you write `write: @founders` for a path, everyone else is implicitly denied for that path. You don't need to enumerate who's blocked — the allow list IS the deny list (inverted).

But if no rule mentions a path at all, it's unrestricted — anyone who can push to the branch can modify it.

## Branch-Scoped Agent Permissions

A key use case: devs spin up constrained agents on feature branches.

### Flow

1. Dev creates `feature/my-thing`
2. Dev appends to `.boxconfig`:
   ```yaml
   @my-agent:
     push:
       - >feature/my-thing
     not write:
       - package.json
   ```
   Plus a group definition:
   ```yaml
   my-agent:
     - evm:0xAgentKey...
   ```
3. Agent works within constraints on the feature branch
4. When feature merges to dev/main, `.boxconfig` changes are rejected (dev doesn't have merge permission for `.boxconfig` changes on protected branches)
5. Agent permissions die with the branch

### Why this is safe

- Dev can only `append` — cannot modify or remove existing rules
- Appended rules land at the bottom — lowest priority, can't override founder rules
- Agent permissions are branch-scoped and ephemeral
- Permission escalation is structurally impossible: a dev can't grant an agent more access than they have themselves

## Enforcement Layers

### Layer 1: Git shim (local)

- Intercepts `git commit`, `git merge`, `git push`, `git checkout -b`, `git branch`
- Validates every commit against `.boxconfig` before delegating to real git
- Checks branch permissions before push/merge/create/delete
- Checks file permissions by inspecting the diff
- Validates `append` constraints (only `+` lines in diff)
- **Cannot be bypassed** — it IS the `git` command in the agent's environment. `--no-verify` is irrelevant.
- Read-only commands (`git status`, `git log`, `git diff`, etc.) pass through unchanged

### Layer 2: Server (remote)

- Validates every push against `.boxconfig` on the target branch
- Rejects non-compliant pushes with descriptive errors
- Defense in depth — catches anything that bypasses the CLI (e.g. raw `git` usage)
- Uses the `.boxconfig` from the **target branch** (not the incoming changes) to prevent permission smuggling

### Permission smuggling prevention

The server always evaluates permissions from the `.boxconfig` on the **target branch**, not from incoming changes. If someone modifies `.boxconfig` to grant themselves access and pushes that change, the server uses the pre-push version of `.boxconfig` to evaluate. The modified permissions don't take effect until they're already on the branch — which requires someone with existing access to merge them.

## Tooling

### `git box lint` (shim subcommand)

- Validates `.boxconfig` syntax
- Detects undefined group references
- Warns about ordering issues (allow below deny for same target)
- Normalizes rules to canonical form (SVO)
- Detects circular group includes

### `git box check <identity> <verb> <target>` (shim subcommand)

- Answers "can this identity do this?" against the current `.boxconfig`
- Shows which rule matched and why
- Essential for debugging permission issues

### `git box diff` (shim subcommand)

- Shows permission changes between two `.boxconfig` versions
- Used in code review to understand impact of permission changes

## What Permissions Does NOT Cover

- **Who is in a group** — See 02-groups.md
- **PR/merge workflows** — See 04-workflows.md
- **Read access to files** — Parked for future consideration
- **Tag permissions** — Future addition
