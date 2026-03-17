# repo.box Spec: Workflows

## Overview

Workflows are generic state machines defined in `.boxconfig`. PRs, ideas, releases, governance proposals — all are instances of workflows. States, transitions, guards, and actions are the building blocks. Instances live as JSONL files in the repo.

**Everything as code.** Workflow definitions, instance data, discussions — all are files in the repo, versioned by git, governed by permissions.

## Core Concepts

- **Workflow**: a named state machine with states and transition rules
- **State**: a named position in the machine (e.g. `draft`, `review`, `merged`)
- **Transition**: a movement from one state to another, with rules about who can trigger it
- **Guard**: a condition that must be true for a transition to proceed (onchain call, HTTP check, internal requirement)
- **Action**: a side effect that fires when a transition completes (merge branch, webhook, notification)
- **Instance**: a single item moving through a workflow (a specific PR, idea, release)

## Workflow Definitions

Defined in `.boxconfig` under `workflows:`:

```yaml
workflows:
  pull-request:
    states:
      - draft
      - review
      - approved
      - merged
      - closed

    transitions:
      draft -> review:
        who:
          - $author
        
      review -> approved:
        who:
          - @devs
        requires:
          approvals: 2

      review -> draft:
        who:
          - $author

      approved -> merged:
        who:
          - @founders
        actions:
          - merge-branch
          - close-branch

      * -> closed:
        who:
          - $author
          - @founders
```

### Special Variables

- `$author` — the identity that created the instance
- `$assignee` — the assigned identity (if set)
- `*` — wildcard state (any state)

### Terminal States

States that have no outgoing transitions are terminal. An instance in a terminal state is immutable — no further transitions possible. In the example above, `merged` and `closed` are terminal.

## Guards

Guards are conditions checked **before** a transition is allowed. If any guard fails, the transition is rejected.

### Approval Guard

```yaml
review -> approved:
  who:
    - @devs
  requires:
    approvals: 2                  # need 2 approved reviews from @devs
```

The workflow engine counts `review` entries with `verdict: approved` in the instance thread from identities in the `who` list.

### Onchain Guard

Calls a smart contract function. The function takes a single `uint256` (the instance ID) and returns `bool`.

```yaml
voting -> executed:
  who:
    - @everyone
  guard:
    onchain:
      chain: 8453
      contract: "0xGovernance..."
      function: isApproved          # f(uint256) → bool
      arg: $instance.id
```

Use case: DAO governance. Members vote onchain, setting `proposalId → true` in a mapping. The transition only proceeds if the contract confirms.

Server calls the contract via its own RPC (same as group resolvers — users don't need nodes).

### HTTP Guard

POSTs instance data to an HTTP endpoint. Expects `{"approved": true}` or `{"approved": false}` in response.

```yaml
review -> approved:
  who:
    - @devs
  guard:
    http:
      url: https://ci.example.com/status
      # POSTs: {"instance_id": 1, "workflow": "pull-request", "branch": "feature/...", ...}
```

Use case: CI status checks, external review tools (Watson/Gaston), security scanners.

### Combined Guards

Multiple guards can be required. All must pass.

```yaml
approved -> merged:
  who:
    - @founders
  requires:
    approvals: 2
  guard:
    http:
      url: https://ci.example.com/status
    onchain:
      chain: 8453
      contract: "0xApprovalRegistry..."
      function: isApproved
      arg: $instance.id
  actions:
    - merge-branch
```

This transition requires: 2 approvals from @founders + CI passing + onchain approval. All three must be true.

## Actions

Actions fire **after** a transition completes successfully. They are side effects.

### Built-in Actions

| Action | Meaning |
|--------|---------|
| `merge-branch` | Merge the instance's source branch into its target branch |
| `close-branch` | Delete the source branch after merge |

### Webhook Action

Fire-and-forget HTTP POST with instance data.

```yaml
actions:
  - webhook:
      url: https://hooks.example.com/deploy
      # POSTs full instance data
```

Use cases: trigger deploys, notify external systems, update dashboards.

### Onchain Action

Call a contract function after transition.

```yaml
actions:
  - onchain:
      chain: 8453
      contract: "0xRegistry..."
      function: recordMerge           # f(uint256) → void
      arg: $instance.id
```

Use case: record events onchain (audit trail, reputation, rewards).

## Instance Storage

Instances are JSONL files in `.box/workflows/<workflow-name>/`.

### Directory Structure

```
.box/
  workflows/
    pull-request/
      index.jsonl              # master index — one line per instance
      fix-auth-bug.jsonl       # discussion thread for PR #1
      add-streaming.jsonl      # discussion thread for PR #2
    idea/
      index.jsonl
      monorepo-migration.jsonl
    release/
      index.jsonl
```

### `index.jsonl` — Master Index

One JSON line per instance. **Line number = instance ID** (1-indexed).

```jsonl
{"title":"Fix auth bug","slug":"fix-auth-bug","branch":"feature/fix-auth","target":"main","author":"evm:0xAlice...","state":"review","labels":["bug","security"],"created":"2026-03-17T10:00:00Z","updated":"2026-03-17T11:15:00Z"}
{"title":"Add streaming support","slug":"add-streaming","branch":"feature/streaming","target":"main","author":"evm:0xBob...","state":"draft","labels":["feature"],"created":"2026-03-17T11:00:00Z","updated":"2026-03-17T11:00:00Z"}
```

Fields:
- `title` — human-readable title
- `slug` — URL/file-safe name, used for the thread filename
- `branch` — source branch (optional — ideas don't have branches)
- `target` — target branch for merge (optional)
- `author` — identity that created the instance
- `state` — current state in the workflow
- `labels` — freeform tags
- `created` / `updated` — timestamps
- Any additional fields defined by the workflow

**State transitions** modify the instance's line in `index.jsonl`. This requires `edit` permission on the index file, which is controlled by the workflow's transition rules.

### `<slug>.jsonl` — Discussion Thread

One file per instance. Append-only bulletin board. Each line is a self-contained JSON event.

#### Line Types

**Description** (first line, by author):
```jsonl
{"type":"description","author":"evm:0xAlice...","body":"This PR fixes the auth bypass in the middleware. The bug was caused by...","ts":"2026-03-17T10:00:00Z"}
```

**Comment:**
```jsonl
{"type":"comment","author":"evm:0xBob...","body":"Looks good but check line 42 in auth.ts","ts":"2026-03-17T10:30:00Z"}
```

**Code suggestion** (inline review comment):
```jsonl
{"type":"suggestion","author":"evm:0xBob...","file":"src/auth.ts","line":42,"old":"if (token) {","new":"if (token && verify(token)) {","body":"Should verify the token, not just check existence","ts":"2026-03-17T10:31:00Z"}
```

**Review verdict:**
```jsonl
{"type":"review","author":"evm:0xBob...","verdict":"approved","body":"LGTM after the fix","ts":"2026-03-17T11:15:00Z"}
```

Verdict values: `approved`, `changes-requested`, `comment-only`.

**Suggestion accepted:**
```jsonl
{"type":"suggestion-accepted","ref":3,"by":"evm:0xAlice...","commit":"abc123","ts":"2026-03-17T11:00:00Z"}
```

`ref` is the line number of the original suggestion.

**Transition log:**
```jsonl
{"type":"transition","from":"review","to":"approved","by":"evm:0xBob...","ts":"2026-03-17T11:15:00Z"}
```

**Cross-reference:**
```jsonl
{"type":"reference","ref":"idea/3","body":"Implements idea #3","author":"evm:0xAlice...","ts":"2026-03-17T10:00:00Z"}
```

### Why JSONL

- **Append-friendly**: new events are new lines. Works perfectly with `append` permission.
- **Machine-readable**: every line is valid JSON. Agents parse it trivially.
- **Git-diffable**: each line is self-contained. Merges are clean. Conflicts are rare (append-only).
- **No schema migration**: new line types can be added without changing existing data.
- **Line number as ID**: implicit, stable, free.

## Creating Instances

To open a new PR/idea/etc:

1. Append a line to `index.jsonl` with metadata and initial state
2. Create a new `<slug>.jsonl` file with the description as the first line

The shim provides a helper: `git box pr create --title "Fix auth bug" --branch feature/fix-auth --target main` (which generates the JSONL entries and commits them). But you can also just manually append to the files with `git commit`.

### Permission Model for Instances

Controlled by the standard file permissions in `.boxconfig`:

```yaml
.box/workflows/pull-request/index.jsonl:
  append: @everyone              # anyone can open a PR (append to index)
  edit: @founders                # only founders can change state

.box/workflows/pull-request/*.jsonl:
  append: @everyone              # anyone can comment on any PR
```

The workflow engine validates transitions additionally — even if you have `edit` on `index.jsonl`, you can only perform transitions allowed by the workflow definition for your identity.

## Example Workflows

### Idea / Discussion

No branch required. Pure discussion workflow.

```yaml
workflows:
  idea:
    states:
      - open
      - discussing
      - accepted
      - rejected
      - implemented

    transitions:
      open -> discussing:
        who:
          - @everyone

      discussing -> accepted:
        who:
          - @founders

      discussing -> rejected:
        who:
          - @founders

      accepted -> implemented:
        who:
          - @devs
        guard:
          requires:
            linked-pr: merged     # must link to a merged PR
```

### Release

```yaml
workflows:
  release:
    states:
      - proposed
      - testing
      - approved
      - published

    transitions:
      proposed -> testing:
        who:
          - @devs

      testing -> approved:
        who:
          - @auditors
        requires:
          approvals: 1
        guard:
          http:
            url: https://ci.example.com/release-check

      approved -> published:
        who:
          - @founders
        actions:
          - webhook:
              url: https://deploy.example.com/publish
```

### Governance Proposal

Token-gated voting with onchain confirmation.

```yaml
workflows:
  proposal:
    states:
      - draft
      - voting
      - passed
      - rejected
      - executed

    transitions:
      draft -> voting:
        who:
          - @token-holders

      voting -> passed:
        who:
          - @everyone
        guard:
          onchain:
            chain: 8453
            contract: "0xGovernor..."
            function: hasPassed
            arg: $instance.id

      voting -> rejected:
        who:
          - @everyone
        guard:
          onchain:
            chain: 8453
            contract: "0xGovernor..."
            function: hasRejected
            arg: $instance.id

      passed -> executed:
        who:
          - @founders
        actions:
          - merge-branch
          - onchain:
              chain: 8453
              contract: "0xGovernor..."
              function: markExecuted
              arg: $instance.id
```

## Tooling

### `git box pr create`
Create a pull request (appends to index.jsonl + creates thread file).

### `git box pr review --approve|--request-changes`
Submit a review verdict (appends review line to thread).

### `git box pr merge`
Attempt the `approved → merged` transition.

### `git box workflow list <workflow-name>`
List all instances of a workflow (reads index.jsonl).

### `git box workflow transition <workflow-name> <id> <target-state>`
Generic transition command for any workflow.

### `git box workflow create <workflow-name> --title "..." [--branch ...] [--target ...]`
Create a new instance of any workflow.

Note: All of these are convenience wrappers. The underlying operation is always a `git commit` that modifies JSONL files — users and agents can do it manually too.

## What Workflows Does NOT Cover

- **CI/CD execution** — workflows can trigger webhooks to external CI systems, but don't run builds themselves.
- **Notifications** — webhook actions can ping notification services, but there's no built-in notification system.
- **Time-based transitions** — "auto-close after 30 days of inactivity" requires an external cron-like system. Could be a future addition.
- **Voting weight** — the onchain guard is boolean (passed/not passed). Vote counting, quorum rules, etc. live in the contract, not in repo.box.
