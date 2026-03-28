# repo.box Spec: Append Merge Driver

## Overview

This specification defines a custom Git merge driver named `append` for files that are intentionally append-only.

The goal is to make true append operations merge cleanly and predictably without manual conflict resolution, while still failing safely when either side performed a non-append edit.

This is **not** a generic no-conflict merge strategy. It is a narrow, explicit merge mode for append-safe file formats.

## Problem Statement

Git does not understand the semantic operation "append". It only understands that a file changed from one snapshot to another.

This creates a mismatch:

- two users may both perform logically valid append operations
- Git may still detect overlapping text edits near the end of the file
- the result is unnecessary merge conflicts for data that should be mechanically mergeable

repo.box should support an explicit append-only merge behavior for eligible files.

## Goals

- Make **strict suffix appends** merge automatically.
- Preserve the destination branch as canonical ordering.
- Fail closed when either side modified pre-existing content.
- Support line-oriented and record-oriented append-safe formats.
- Keep behavior deterministic and easy to explain.

## Non-Goals

- Do not attempt semantic merging for arbitrary text files.
- Do not silently repair edits, reorderings, deletions, or in-place modifications.
- Do not guarantee globally correct timestamp or causal ordering.
- Do not make syntax-sensitive formats safe if they are not append-safe by construction.

## Terminology

For a merge operation, define:

- **base**: the common ancestor version of the file
- **destination**: the version already on the checked-out branch being merged into
- **source**: the version from the branch being merged in
- **destination tail**: content appended to `base` in destination
- **source tail**: content appended to `base` in source

The append merge result is defined in destination-first order:

`result = base + destination_tail + source_tail`

This means that when merging a feature branch into `main`, any append that already landed on `main` stays first, and the feature branch append lands after it.

## Applicability

The `append` merge driver MUST only be used for files explicitly marked as append-safe.

Recommended file types:

- `*.jsonl`
- `*.ndjson`
- append-only log files
- one-record-per-line journals
- immutable operation logs

Discouraged or unsupported file types:

- JSON arrays
- YAML lists
- Markdown lists edited by hand
- files with sorting or normalization passes
- files whose syntax requires rewriting a closing delimiter

## Merge Semantics

The `append` merge driver MUST follow these rules:

1. Treat the merge as valid only if both destination and source preserve `base` exactly as a byte prefix.
2. If destination does not begin with `base`, the merge MUST fail.
3. If source does not begin with `base`, the merge MUST fail.
4. If both are valid strict suffix appends, extract:
   - `destination_tail`
   - `source_tail`
5. Produce merged output as:
   - `base`
   - then `destination_tail`
   - then `source_tail`
6. Write the merged result back as the resolved file.
7. Return success only when all validations pass.

## Strict Append Definition

A file version is a **strict suffix append** of `base` if:

- the entire contents of `base` appear unchanged at the beginning of the file
- the only change is additional content after the end of `base`

Examples:

Valid append:

```txt
base:
a
b

version:
a
b
c
```

Invalid append:

```txt
base:
a
b

version:
a
x
b
```

Invalid append:

```txt
base:
a
b

version:
a
```

Invalid append:

```txt
base:
a
b

version:
a
b 
```

The final example is invalid because even a whitespace mutation of existing content is not a strict append.

## Ordering Policy

The `append` merge driver is **destination-first**.

That means:

- content already on the branch being merged into stays first
- content from the incoming branch is appended after it

Example:

```txt
base:
a
b

destination (main):
a
b
c

source (feature):
a
b
d
```

Merged result:

```txt
a
b
c
d
```

This matches the intended mental model:

- `main` is the canonical timeline
- feature-branch appends should land after what already happened on `main`

## Duplicate Handling

Default v1 behavior:

- The merge driver MUST preserve both tails exactly as written.
- It MUST NOT deduplicate content by default.

Rationale:

- byte-level append merging is deterministic and easy to audit
- content-level deduplication requires file-type-specific semantics
- aggressive deduplication risks silent data loss

Optional future extensions MAY add file-type-specific deduplication, for example:

- exact line dedupe for plaintext logs
- record ID dedupe for JSONL records with stable IDs

Any such extension MUST be explicit and format-aware.

## Failure Conditions

The merge driver MUST fail with conflict status when any of the following is true:

- destination is not a strict suffix append of base
- source is not a strict suffix append of base
- the file is missing or unreadable in any required merge input
- file-type validation fails, if validation is enabled
- configured append-safe constraints are violated

Failure MUST prefer explicit conflict over silent corruption.

## Recommended Validation Modes

### Mode 1: Raw Byte Prefix Validation

Minimum required behavior for v1.

- compare raw bytes
- require exact `base` prefix in destination and source
- concatenate tails in destination-first order

This mode is simple, predictable, and format-agnostic.

### Mode 2: Line Boundary Validation

Optional improvement.

In addition to raw prefix validation:

- require `base` to end on a complete line boundary when used for line-oriented formats
- reject malformed partial-line appends if the file type requires one-record-per-line discipline

### Mode 3: Record-Aware Validation

Optional future improvement for formats like JSONL.

Examples:

- validate each appended line parses as JSON
- reject duplicate IDs with differing payloads
- allow exact duplicate record collapse only when explicitly configured

## Configuration

Files opt into the append merge driver through `.gitattributes`.

Example:

```gitattributes
*.jsonl merge=append
*.ndjson merge=append
*.log merge=append
```

Git config defines the driver command.

Example:

```ini
[merge "append"]
    name = repo.box append-only merge driver
    driver = repobox-append-merge %O %A %B %L %P
```

Git-provided placeholders:

- `%O` = base file
- `%A` = destination file (current checked-out side, overwritten with result)
- `%B` = source file (incoming side)
- `%L` = conflict marker length
- `%P` = path being merged

Implementation MUST treat `%A` as destination and `%B` as source.

## Reference Algorithm

Pseudocode:

```text
read base from %O
read destination from %A
read source from %B

if !destination.starts_with(base):
  fail

if !source.starts_with(base):
  fail

destination_tail = destination[len(base):]
source_tail = source[len(base):]

result = base + destination_tail + source_tail
write result to %A
return success
```

## Examples

### Example 1: Plain append on both sides

```txt
base:
hello


destination:
hello
world

source:
hello
moon
```

Result:

```txt
hello
world
moon
```

### Example 2: Source edited earlier content

```txt
base:
a
b


destination:
a
b
c

source:
a
x
b
```

Result:

- merge fails
- user resolves manually or uses a different merge strategy

### Example 3: Empty source tail

```txt
base:
a
b


destination:
a
b
c

source:
a
b
```

Result:

```txt
a
b
c
```

This is a valid no-op append on source.

## Safety Properties

If implemented correctly, the append merge driver guarantees:

- no conflict for true strict appends
- no silent acceptance of in-place edits
- deterministic merge order
- easy explainability to users

It does **not** guarantee:

- semantic uniqueness
- timestamp order correctness
- causality preservation across branches
- syntax safety for non-append-safe formats

## Open Questions / Future Work

- Should repo.box expose `append` as a first-class policy verb in addition to a merge driver?
- Should JSONL support optional record-ID dedupe?
- Should append-safe files require schema validation on merge?
- Should repo.box surface a clearer user-facing error than generic merge conflict when strict append validation fails?
- Should rebases also reuse the same append semantics where technically possible?

## Recommendation

repo.box SHOULD implement `append` as a strict, destination-first, fail-closed merge driver for explicitly opted-in append-safe file types.

The first implementation SHOULD:

- use raw byte prefix validation
- preserve destination-first ordering
- avoid deduplication
- fail on any non-append edit

That gives repo.box a precise and honest guarantee:

**append actions merge automatically when they are truly append-only, and conflict when they are not.**
