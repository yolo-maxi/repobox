# repo.box Spec: Server-First Test Matrix Architecture

## Purpose

This document captures the testing architecture discussed for repo.box policy enforcement, with one core decision:

**the server integration suite is the canonical source of truth for security behavior.**

Shim tests matter for UX and early blocking, but they do not prove the real security boundary. A buggy shim, outdated client, or direct `git` client must still be rejected by the server when policy requires it.

The goal of this spec is to make the test suite:

- **server-first**
- **modular**
- **matrix-driven**
- **coverage-auditable**
- easy to extend when adding new policy dimensions

Specifically, when adding a new row or column to the policy matrix, we should be able to answer:

- which policy families are affected
- which scenarios must be added
- which existing scenarios are now incomplete
- which exclusions are intentional vs accidental

## Core Principles

### 1. Server tests are canonical

If a policy case exists only in shim tests, it is not truly covered.

The server must prove:

- raw `git push` without shim enforcement
- authoritative allow/reject behavior
- bypass resistance
- end-to-end Git transport behavior
- branch/ref update semantics
- commit-set validation on what actually lands remotely

### 2. Shim tests are parity and UX tests

The shim suite exists to prove:

- early local feedback
- helpful error messages
- preflight blocking before network transfer
- parity with the server on shared decisions

A shim is a convenience layer, not the security boundary.

### 3. Unit tests isolate pure logic

Unit tests should cover deterministic internals such as:

- parser behavior
- matcher behavior
- change classification
- append validator logic
- escalation/precedence logic

Unit coverage is necessary, but never substitutes for server integration coverage.

### 4. The suite should be driven by policy dimensions, not ad hoc test files

The suite should be modeled as a set of policy families, dimensions, and required scenario combinations.

That means test organization is based on:

- policy area
- operation
- actor state
- repo state
- change shape
- transport/client path
- expected result / reason code

not on whatever seemed convenient when the first tests were written.

## Suite Hierarchy

The intended hierarchy is:

### Source of truth
- server integration tests

### Consistency layer
- shim parity tests

### Fast logic layer
- unit tests

This hierarchy should be explicit in file layout, naming, CI reporting, and coverage requirements.

## What the Server Suite Must Cover

The server integration suite should be organized around the policy promises repo.box makes to users.

### Required server policy areas

1. **Repo lifecycle / ref setup**
   - first push to empty repo
   - first protected branch creation
   - push to existing repo
   - branch create / update / delete
   - non-fast-forward attempts
   - force-push attempts
   - pushing new branch from allowed vs disallowed base

2. **Commit-set validation**
   - single allowed commit
   - mixed allowed + forbidden commits in one push
   - multiple commits where only one violates policy
   - historical bad commit included in pushed range
   - rewritten history introducing violations
   - merge commit containing forbidden tree changes
   - empty/no-op commit handling

3. **File policy enforcement**
   - owned file modified by owner
   - owned file modified by non-owner
   - unowned file modified
   - restricted-path creation
   - delete in protected path
   - rename across policy boundaries
   - chmod / mode-only changes if relevant
   - symlink/submodule edge cases if supported

4. **Append / mutation semantics**
   - strict append allowed
   - insertion in the middle rejected
   - deletion rejected
   - reorder rejected
   - whole-file rewrite rejected
   - newline-only / tail-edge cases
   - binary behavior if append semantics apply to binary payloads

5. **Identity / authorship**
   - valid signed commit by permitted actor
   - unsigned commit rejected
   - malformed signature rejected
   - valid signature but wrong actor rejected
   - mixed-signature push
   - bot/delegated identity variants if supported

6. **Merge semantics**
   - merge commit accepted when valid
   - merge commit rejected when resulting diff violates policy
   - squash merge path
   - rebase-updated branch path
   - conflict-resolution commit crossing a policy boundary

7. **Addressing / target resolution**
   - addressed remote path
   - addressless remote path
   - malformed target path
   - ambiguous target resolution
   - namespace escape attempts

8. **Bypass resistance**
   - raw `git push` with no shim
   - push from outdated shim
   - push with shim false-allow behavior
   - mixed ref updates
   - alternate client implementation path

9. **Error quality / determinism**
   - exact offending commit/file/rule identified when possible
   - deterministic rejection ordering for multi-error cases
   - no partial acceptance when atomic reject is intended

## Dimensions

The suite should model scenarios as combinations of dimensions.

Not every policy family uses every dimension, but the dimensions must be explicit and reusable.

### Recommended base dimensions

- **layer**
  - `unit`
  - `shim`
  - `server`

- **policyArea**
  - `ownership`
  - `appendOnly`
  - `signatures`
  - `branchTopology`
  - `mergeRules`
  - `routing`
  - etc.

- **operation**
  - `push`
  - `createBranch`
  - `deleteBranch`
  - `forcePush`
  - `merge`
  - `rebaseResultPush`
  - etc.

- **repoState**
  - `emptyRepo`
  - `existingBranch`
  - `divergedHistory`
  - `mergePresent`
  - `protectedBranch`
  - etc.

- **changeShape**
  - `create`
  - `modify`
  - `delete`
  - `rename`
  - `append`
  - `rewrite`
  - `modeOnly`
  - etc.

- **actorState**
  - `allowed`
  - `disallowed`
  - `owner`
  - `nonOwner`
  - `signed`
  - `unsigned`
  - `wrongSigner`
  - etc.

- **clientPath**
  - `rawGit`
  - `shim`
  - `outdatedShim`
  - `alternateClient`

- **expectedResult**
  - `allow`
  - `reject`

- **reasonCode**
  - `ownership_violation`
  - `append_violation`
  - `signature_required`
  - `non_ff_denied`
  - `branch_create_denied`
  - etc.

These dimensions are the schema for the matrix. They are what make future additions inspectable.

## Policy Families

Each major policy family should own:

- its relevant dimensions
- its canonical reason codes
- its required server scenarios
- its optional shim parity scenarios
- its valid exclusions

### Example policy families

- `ownership`
- `appendOnly`
- `signatures`
- `branchUpdates`
- `mergeRules`
- `routing`

A policy family should explicitly declare which dimensions matter to it. This avoids meaningless full cross-products.

For example:

- **ownership** cares about:
  - `operation`
  - `changeShape`
  - `actorState`
  - `clientPath`

- **signatures** cares about:
  - `operation`
  - `actorState`
  - `clientPath`
  - `repoState`

- **appendOnly** cares about:
  - `operation`
  - `changeShape`
  - `clientPath`
  - maybe file type / file state

## Coverage Contracts

Each policy family should define a coverage contract.

A coverage contract says:

- which dimensions are relevant
- which combinations are mandatory
- which combinations are invalid/irrelevant
- which exclusions are allowed
- which layer(s) are required for each combination

This is what gives the suite the property:

**“if we add a new row or column, we know exactly which tests must be added.”**

### Example

Ownership might require server coverage for:

- operation: `push`, `merge`
- changeShape: `modify`, `delete`, `rename`, `create`
- actorState: `owner`, `nonOwner`
- clientPath: `rawGit`

and shim parity coverage for:

- the same combinations where local preflight is expected to block or allow early

If a new operation `squashMerge` is added to the dimension registry, the coverage checker should immediately report the new required ownership cases.

## Scenario Schema

Tests should be declared as data, not handwritten imperatively one-by-one.

A scenario should carry enough metadata to support execution, reporting, and coverage auditing.

### Recommended scenario shape

```ts
type Scenario = {
  id: string;
  policyArea: string;
  layer: 'server' | 'shim' | 'unit';
  dimensions: {
    operation?: string;
    repoState?: string;
    changeShape?: string;
    actorState?: string;
    clientPath?: string;
    expectedResult?: 'allow' | 'reject';
    reasonCode?: string;
    [key: string]: string | undefined;
  };

  fixture: string;
  action: string;

  expected: {
    result: 'allow' | 'reject';
    reasonCode?: string;
    messageIncludes?: string[];
  };

  parityWith?: string;
  covers?: string[];
  notes?: string;
};
```

### Required fields

At minimum every non-unit scenario should declare:

- `id`
- `policyArea`
- `layer`
- `dimensions`
- `fixture`
- `action`
- `expected`
- `reasonCode` when rejected

### Optional but recommended fields

- `parityWith` for shim/server linked scenarios
- `covers` for human-readable coverage reporting
- `notes` for rationale or edge-case warnings

## Test Naming

Test names should encode dimensions, not vague prose.

### Good examples

- `server/ownership/push_modify_nonOwner_rawGit_rejects`
- `server/appendOnly/push_append_signed_rawGit_allows`
- `server/signatures/push_unsigned_rawGit_rejects`
- `shim/ownership/push_modify_nonOwner_shim_rejects_matches_server`

### Why this matters

Dimension-based names make it much easier to:

- scan coverage visually
- spot duplicates
- identify missing combinations
- map failures back to policy contracts

## File Layout

Suggested structure:

```text
tests/
  policies/
    ownership.ts
    appendOnly.ts
    signatures.ts
    branchUpdates.ts
    mergeRules.ts

  matrix/
    dimensions.ts
    coverage.ts
    ownership.server.matrix.ts
    ownership.shim.matrix.ts
    appendOnly.server.matrix.ts
    signatures.server.matrix.ts

  fixtures/
    repoBuilder.ts
    actors.ts
    actions.ts
    assertions.ts

  runners/
    runServerMatrix.ts
    runShimMatrix.ts
    runUnitMatrix.ts

  generated/
    expanded-scenarios.json
    coverage-report.json
```

This is a logical structure, not a required literal one, but the important separation is:

- policy definition
- matrix declaration
- fixture/action builders
- runners
- generated coverage outputs

## Runner Responsibilities

The runner layer should:

- expand scenario declarations into executable tests
- create consistent names
- provision fixtures
- execute the action
- assert the expected result and reason code
- emit coverage metadata

Recommended helpers:

- `runServerMatrix(matrixDefinition)`
- `runShimMatrix(matrixDefinition)`
- `runCoverageAudit(policyDefinitions, scenarios)`

## Coverage Auditor

A small coverage auditor is the key to keeping the suite maintainable.

It should:

- expand required combinations from each policy family’s coverage contract
- compare them to declared scenarios
- report missing cases
- report duplicate cases
- report orphan cases not covered by any contract
- require justification for exclusions

### Example outputs

- `ownership.server: missing [merge × rename × nonOwner × rawGit]`
- `appendOnly.server: missing [push × rewrite × rawGit]`
- `signatures.shim: orphan case [merge × unsigned × shim]`

CI should fail if required combinations are missing.

## Full Cross-Product vs Controlled Expansion

Do **not** blindly cross-product every dimension with every other dimension.

That will create a huge, noisy suite with many meaningless combinations.

Instead:

- define relevant dimensions per policy family
- use full cross-product only for security-critical invariants
- use smaller pairwise or hand-curated expansion for lower-risk variations
- explicitly list exclusions when a combination is not meaningful

### Good candidates for full coverage

- raw-git vs shim path
- allow vs reject
- signed vs unsigned
- owner vs non-owner
- append vs rewrite
- branch create/update/delete where policy semantics differ

### Good candidates for pairwise or selective coverage

- empty repo vs existing repo
- addressless vs addressed path when behavior is otherwise identical
- branch naming variants
- formatting/error string variants

## Required Parity Rule

For every shim-blocked case:

- there must be a corresponding server case unless the behavior is explicitly shim-only UX
- the server must be equally strict or stricter
- any shim/server difference must be documented and justified

In other words:

- shim may fail earlier
- shim may explain better
- shim may offer friendlier recovery
- shim must not be the only enforcement of a security promise

## Recommended Initial Rollout Order

### P0

- raw push without shim
- commit-set validation
- file authorship enforcement
- branch/ref update rules
- merge acceptance/rejection
- non-fast-forward / force-push rules

### P1

- append-only semantics
- signature variants
- rename/delete/create edge cases
- addressed/addressless routing parity

### P2

- deterministic error ordering
- pathological histories
- stress / large push-set behavior
- rare Git object edge cases

## Example Test Case Template

Each scenario should be easy to understand as a policy witness.

### Example

- **Name**: non-owner cannot modify owned file via raw push
- **Policy area**: ownership
- **Initial state**: `main` contains `docs/spec.md` owned by `alice`
- **Action**: `bob` commits an edit and pushes directly
- **Expected server result**: reject
- **Expected reason code**: `ownership_violation`
- **Shim expectation**: local block if shim path is used
- **Bypass variant**: same push with no shim still rejects

That should map cleanly into scenario data and coverage reporting.

## What Changes Mean in This Architecture

This architecture is specifically designed to make extension predictable.

### Adding a new policy family

Required work:

- add a policy module
- define relevant dimensions
- define coverage contract
- define reason codes
- add fixtures/actions if needed
- add server matrix
- optionally add shim parity matrix

### Adding a new dimension value

Examples:

- new operation: `squashMerge`
- new change shape: `modeOnly`
- new client path: `outdatedShim`

Required outcome:

- coverage auditor reports which policy families now need additional scenarios
- CI fails until those scenarios or explicit exclusions are added

### Adding a new transport path

Required work:

- add runner support
- declare which policy families care about the new path
- add required scenarios where bypass/security semantics differ

### Adding a new repo-state pattern

Required work:

- add fixture builder support
- attach that state to affected policy families
- extend coverage contract where behavior genuinely differs

## Non-Goals

This spec does **not** require:

- a perfect full Cartesian product of all dimensions
- immediate migration of every existing test
- replacing useful hand-written edge-case tests where data-driven generation is awkward

The goal is not total mechanization. The goal is predictable, auditable structure.

## Final Rule

The most important rule from this discussion is:

> If a case only exists in shim tests, treat it as not really covered.

repo.box makes security promises at the server boundary. The test architecture should reflect that.

## Recommended Next Step

Build a small internal framework for:

- dimension registry
- policy coverage contracts
- scenario declarations
- matrix runners
- coverage auditing

Then migrate the highest-risk policy families into that system first, starting with server integration.
