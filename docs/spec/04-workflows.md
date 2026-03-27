# repo.box Workflows v1

This file is both:

- the product spec for workflow definitions
- the operating guide for agents writing or validating workflow transactions

It defines:

- what a workflow config file must contain
- how workflow rules interact with `.repobox/config.yml`
- how workflow state is stored in Git refs
- what the server clerk must validate
- what local shim validation should mirror

---

## 1. Goal

A workflow file defines a **deterministic state machine** for a tracked object.

It tells repo.box:

- what states exist
- which transitions are allowed
- who may perform them
- what approvals are required
- which canonical events the core emits

It does **not** define delivery, execution, or automation behavior.

---

## 2. Core boundary

### Core owns
The repo.box core engine, both server-side and local where possible, is responsible for:

- schema validation
- transition validation
- permission validation
- approval validation
- state mutation validation
- canonical event emission

### Agents / integrations own
Agents and integrations are responsible for:

- observing emitted events
- doing external work
- checking external systems
- writing new events or state updates back into repo.box

### Therefore workflow files must not contain
- webhook URLs
- delivery config
- retries
- ack subsystems
- CI/GitHub-specific checks
- arbitrary scripts
- side effects
- execution plans

If an external fact matters, an agent must first write it into canonical state. Then workflow validation may check that state.

---

## 3. Workflow definition location

Recommended path:

```text
.repobox/workflows/<name>.yaml
```

Example:

```text
.repobox/workflows/feature_delivery.yaml
```

Workflow definitions live on `main` and are versioned as normal repo code.

---

## 4. Required YAML shape

A workflow definition must follow this shape:

```yaml
workflow: feature_delivery
version: 1

object:
  type: feature
  state_field: state
  required:
    - id
    - title
    - state

states:
  - idea
  - scoped
  - planned
  - building
  - review
  - staged
  - prod
  - rejected

initial: idea

terminal:
  - prod
  - rejected

paths:
  - null->idea->scoped->planned->building->review->staged->prod

allow:
  - review->building
  - staged->building
  - idea->rejected

permissions:
  default: deny
  rules:
    - "* transition null->idea"
    - "pm transition idea->scoped->planned"
    - "dev transition planned->building"
    - "reviewer transition building->review"
    - "release transition review->staged->prod"

requires:
  - when: review->staged
    approvals:
      from: group:project:reviewers
      threshold: majority

events:
  emit:
    - transition.requested
    - transition.committed
    - transition.rejected
```

---

## 5. Field semantics

### `workflow`
Human-readable workflow id.

### `version`
Workflow schema version. Start with `1`.

### `object`
Defines the governed object shape.

```yaml
object:
  type: feature
  state_field: state
  required:
    - id
    - title
    - state
```

Meaning:

- `type`: object kind
- `state_field`: field storing workflow state
- `required`: fields required on every object

### `states`
All valid states.

### `initial`
State used on creation.

### `terminal`
Final states. No further transitions out unless a future schema explicitly permits them.

### `paths`
Canonical lifecycle chains.

Example:

```yaml
paths:
  - null->idea->scoped->planned->building->review->staged->prod
```

This implies allowed edges:

- `null->idea`
- `idea->scoped`
- `scoped->planned`
- `planned->building`
- `building->review`
- `review->staged`
- `staged->prod`

### `allow`
Extra edges outside the canonical paths.

Example:

```yaml
allow:
  - review->building
  - idea->rejected
```

### `permissions`
Workflow-layer transition permissions.

Example:

```yaml
permissions:
  default: deny
  rules:
    - "* transition null->idea"
    - "reviewer transition review->approved"
```

Interpretation:

- `default` is the fallback workflow policy
- each rule grants transition capability
- chained edges expand left-to-right

Example:

```yaml
"pm transition idea->scoped->planned"
```

expands to:

- `idea->scoped`
- `scoped->planned`

### `requires`
Approval requirements for specific transitions.

Example:

```yaml
requires:
  - when: in_review->approved
    approvals:
      from: group:project:reviewers
      threshold: majority
```

### `events.emit`
Canonical event types emitted by the core after validation.

This declares event types, not delivery mechanisms.

---

## 6. Selector syntax

Selectors identify users or groups.

Recommended shape:

```text
<kind>:<namespace>:<value>
```

Examples:

- `user:evm:0xF053A15C36f1FbCC2A281095e6f1507ea1EFc931`
- `user:ens:fran.eth`
- `group:project:reviewers`
- `group:workflow:maintainers`

### Rule
Selectors may be human-friendly in config, but the engine should normalize them to canonical principals during evaluation.

Example:

- `user:ens:fran.eth` may resolve internally to an EVM identity

---

## 7. Approval semantics

### Shape

```yaml
requires:
  - when: in_review->approved
    approvals:
      from: group:project:reviewers
      threshold: majority
```

### `approvals.from`
Defines the candidate signer set.

Can be:

- a user selector
- a group selector

### `approvals.threshold`
Allowed v1 values:

- integer, e.g. `1`, `2`, `3`
- `all`
- `majority`

### Validation semantics
At validation time:

- resolve the selector set from canonical state/config
- dedupe signers by canonical identity
- evaluate threshold against the resolved set

### Majority rule

```text
majority(X) = floor(|X| / 2) + 1
```

### Empty-set rule
If a selector resolves to an empty set:

- reject the transition
- error: `ERR_GUARD_EMPTY_SET`

If no approval rule is wanted, omit `requires`.

---

## 8. Relationship to `.repobox/config.yml`

There are **two permission layers**.

### Layer 1: base repo permissions
Defined in:

```text
.repobox/config.yml
```

This governs repository capabilities such as:

- who may read
- who may push
- who may write refs
- who may update canonical workflow state refs
- who may edit workflow definitions on `main`

This is the coarse security perimeter.

### Layer 2: workflow permissions
Defined in the workflow YAML.

This governs workflow behavior such as:

- who may move `idea -> scoped`
- who may approve `review -> staged`
- who may close a bug
- who may trigger a release transition

This is the process layer.

### Critical rule
A transition is allowed only if **both** layers allow it.

That means:

- passing workflow rules is not enough
- passing repo config rules is not enough
- both must pass

### Practical split
A user may be allowed to:

- push proposal refs
- submit transition requests
- comment on workflow objects

without being allowed to:

- mutate canonical workflow state refs directly
- approve protected transitions
- edit workflow definitions

That split is intentional.

### Recommended responsibility split
`.repobox/config.yml` should define:

- identities
- groups
- base read/write capabilities
- which refs are writable by which actors
- who can edit workflow definitions

Workflow YAML should define:

- valid states
- valid transitions
- transition permissions
- approval rules
- emitted canonical events

Workflow YAML should preferably reference groups rooted in `.repobox/config.yml` rather than duplicating identity truth.

---

## 9. Git storage model

Workflow state is stored in Git refs, not only in files on `main`.

### Definitions
Workflow definitions live in versioned files on `main`, for example:

```text
.repobox/workflows/feature_delivery.yaml
```

### Canonical state refs
Recommended pattern:

```text
refs/repobox/workflows/<workflow>/<object-id>
```

Example:

```text
refs/repobox/workflows/feature_delivery/feat-123
```

That ref points to the canonical serialized state for that object.

### Transition request refs
Optional proposal/request refs should live separately, for example:

```text
refs/repobox/workflow-queue/<workflow>/<object-id>/<request-id>
```

This lets actors propose transitions without directly mutating canonical state.

### Event refs
Canonical event history may also be materialized separately, for example:

```text
refs/repobox/workflow-events/<workflow>/<object-id>
```

Exact storage layout may evolve, but the distinction should remain:

- definitions in `main`
- canonical state in refs
- requests/events in dedicated workflow refs

---

## 10. Canonical object state

Each workflow object should serialize enough data for deterministic validation.

Minimum recommended fields:

```yaml
id: feat-123
type: feature
title: Add YAML workflow parser
state: review
version: 7
workflow: feature_delivery
```

Additional fields may include:

- approvals
- signers
- timestamps
- linked refs
- attached artifact refs
- review metadata

### Rule
Core validation may only rely on:

- current canonical state
- proposed next state
- workflow definition
- attached signed payload

It must not fetch external truth.

So this is allowed:

- `review.approval_count >= 2` if that field is already in canonical state

This is not allowed:

- “GitHub checks are green” unless an agent first writes that fact into canonical state

---

## 11. Transaction model

A transition transaction should minimally include:

- target object id
- workflow id
- current version or base ref
- proposed state change
- actor identity
- signatures
- optional code diff / artifact refs
- timestamp

Intent:

- signer submits an event
- signer includes `codeDiff` or artifact refs when relevant
- clerk checks whether the transaction is well-formed
- clerk checks whether the transition is allowed
- clerk rejects malformed or invalid transactions

### Clerk must reject when
- workflow file is invalid
- object shape is invalid
- state is unknown
- transition edge is not allowed
- actor lacks base repo permission
- actor lacks workflow permission
- approval threshold is unsatisfied
- selector resolves to empty set
- version/base is stale
- payload/signature is malformed

---

## 12. Local validation objective

Workflow validation should work locally as well as on the server.

### Why
This reduces back-and-forth and lets users/agents catch invalid transitions before push.

### Objective
The local shim should mirror server validation as closely as possible for deterministic checks.

That includes:

- workflow schema parsing
- transition edge checks
- workflow permission checks
- approval-shape checks
- object/state shape checks
- stale-base/version checks when enough local information exists

### Server remains canonical
Local validation is a fast fail layer.
Server validation remains the final authority.

If local and server validation diverge, that is a bug to fix.

---

## 13. Agent rules

When acting against workflows, agents should:

1. read `.repobox/config.yml`
2. read the referenced workflow YAML
3. resolve the object’s current canonical state from refs
4. propose only valid transitions
5. attach only well-formed payloads
6. never assume workflow permission implies ref-write permission
7. never assume external facts unless already written into canonical state

Agents must not:

- invent side effects from workflow YAML
- treat delivery as workflow semantics
- bypass canonical refs
- mutate workflow state directly unless base repo permissions allow it

---

## 14. Non-goals for v1

Not part of v1:

- webhook delivery
- retries
- ack primitives
- arbitrary expression languages
- external API querying inside validation
- custom scripting in workflow files
- platform-specific automation rules

Keep v1 small, deterministic, and enforceable.

---

## 15. Short rule

If a workflow rule requires the core to understand the outside world, it is probably the wrong rule.

The core should validate **state**, not discover reality.

Reality should arrive through signed events first.
