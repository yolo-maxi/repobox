---
title: "Examples"
description: "Worked examples of .repobox.yml configurations from minimal to strict."
---

# Examples

Real `.repobox.yml` configurations for common setups.

## Minimal: Just Protect Main

```yaml
permissions:
  default: allow
  rules:
    - founders merge >main
```

- Only founders can merge to main
- Everything else is open
- Agents can push anywhere, edit anything

## Standard: Branch Control

```yaml
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
    - evm:0xAAA0000000000000000000000000000000000002
  agents:
    - evm:0xBBB0000000000000000000000000000000000001
    - evm:0xBBB0000000000000000000000000000000000002

permissions:
  default: allow
  rules:
    - founders push >*
    - founders merge >*
    - founders create >*
    - agents:
        push:
          - ">feature/**"
          - ">fix/**"
        create:
          - ">feature/**"
          - ">fix/**"
```

- Founders control all branches
- Agents work on feature/fix branches only
- No file restrictions

## Strict: Branch + File Control

```yaml
permissions:
  default: allow
  rules:
    - founders push >*
    - founders merge >*
    - founders create >*
    - founders edit ./.repobox.yml
    - agents:
        push:
          - ">feature/**"
          - ">fix/**"
        create:
          - ">feature/**"
          - ">fix/**"
        edit:
          - "./* >feature/**"
          - "./* >fix/**"
        append:
          - "./.repobox.yml"
```

- Agents edit files only on their branches
- `.repobox.yml` is founder-only for full edits, agents can append
- Agents can't edit anything on main

## CI Bot: File-Scoped Main Access

```yaml
groups:
  founders:
    - evm:0xAAA0000000000000000000000000000000000001
  deploy-bot:
    - evm:0xCCC0000000000000000000000000000000000001

permissions:
  default: allow
  rules:
    - founders push >*
    - founders merge >*
    - deploy-bot:
        push:
          - ">main"
        edit:
          - "./CHANGELOG.md >main"
          - "./k8s/** >main"
```

- deploy-bot can push to main but only touch CHANGELOG.md and k8s/
- A commit touching anything else is blocked

## Priority Model

Rules evaluate top-to-bottom, first match wins. This is safe because:

- **Founders** write rules at the top (they have full `edit` on `.repobox.yml`)
- **Agents** can only `append` (bottom of file = lowest priority)
- Append-only + top-wins = permission escalation is structurally impossible
