# repo.box Spec: Groups

## Overview

Groups are named collections of identities. The system asks one question: **is identity X in group Y?**

Groups don't grant permissions directly — they're just lists. The permissions layer maps groups to actions.

## Core Interface

Three operations:

```typescript
interface GroupResolver {
  isMember(group: string, address: string): Promise<boolean>;
  listMembers(group: string): Promise<string[]>;  // May not be available for all resolvers
}
```

Mutation (add/remove) is only available for static groups and happens by editing `.repobox-config`.

`isMember()` is the only required method. `listMembers()` is best-effort.

## Resolver Types

### 1. Static (local list)

Members listed directly in `.repobox-config`. No external calls.

```ini
[group "core-team"]
  member = evm:0xAAA...
  member = evm:0xBBB...
  member = evm:0xCCC...
```

- `isMember()`: O(1) lookup from parsed config
- `listMembers()`: returns the full list
- Mutation: edit the config file

### 2. Onchain (trustless verification)

Membership checked by calling a smart contract function on a specific chain. The contract must implement:

```solidity
function <name>(address) external view returns (bool)
```

Any function that takes a single `address` and returns `bool` qualifies.

```ini
[group "token-holders"]
  resolver = onchain
  chain = 8453
  contract = 0xDDD...
  function = isMember
  indexer = https://api.example.com/groups/token-holders
```

Config fields:
- `chain` — Chain ID (EIP-155) where the contract lives
- `contract` — Contract address
- `function` — Function name. Must have signature `f(address) → bool`
- `indexer` — (Optional) HTTP endpoint for full member list

Verification flow:
1. Server encodes `function(signerAddress)` as calldata
2. Sends `eth_call` to the contract on the specified chain via server-side RPC
3. Decodes bool result

**Trustless**: the membership check is verified onchain. No trust in third parties.

**Full list is trustful**: Since you can't efficiently enumerate all addresses that return `true` from an onchain function, the optional `indexer` URL provides the full list. This is used for display/admin only, never for access control decisions.

If no `indexer` is configured, `listMembers()` returns only static members (if the group also has `include` or `member` entries).

**RPC is server-side**: Users and CLIs never need their own RPC node. repo.box maintains RPC connections per chain and handles all onchain calls.

**Caching**: Results cached with configurable TTL per group to avoid excessive RPC calls. Default: 5 minutes.

```ini
[group "token-holders"]
  resolver = onchain
  chain = 8453
  contract = 0xDDD...
  function = isMember
  cache-ttl = 300
```

#### What about multi-param functions (e.g. Hats Protocol)?

Hats uses `isWearerOfHat(uint256 hatId, address account) → bool` which doesn't fit the `f(address) → bool` interface.

**Solution**: Deploy a tiny wrapper contract that hardcodes the extra params:

```solidity
contract HatsGroupCheck {
    IHats public immutable hats;
    uint256 public immutable hatId;
    
    constructor(address _hats, uint256 _hatId) {
        hats = IHats(_hats);
        hatId = _hatId;
    }
    
    function isMember(address account) external view returns (bool) {
        return hats.isWearerOfHat(hatId, account);
    }
}
```

This keeps our interface dead simple — one function signature, no ABI complexity on the server side. The ecosystem can compose whatever logic they want behind `f(address) → bool`.

### 3. HTTP (external API)

Membership checked via HTTP call to an external endpoint.

```ini
[group "company"]
  resolver = http
  url = https://api.example.com/groups/company
```

The endpoint must implement:

```
GET <url>/members/:address → { "member": true | false }
GET <url>/members → ["evm:0xAAA...", "evm:0xBBB..."]
```

- First route: membership check (required)
- Second route: full list (optional, for display/admin)

**Trustful**: the server trusts the API response. Use for internal systems, indexers, or any source you control.

## Recursive Groups

Groups can include other groups. Membership cascades.

```ini
[group "frontend-team"]
  member = evm:0xAAA...
  member = evm:0xBBB...

[group "backend-team"]
  member = evm:0xCCC...

[group "core-team"]
  include = frontend-team
  include = backend-team
  member = evm:0xDDD...
```

`isMember("core-team", 0xAAA)` → checks direct members first, then recurses into `frontend-team` and `backend-team`. Returns `true` because 0xAAA is in frontend-team.

### Composition Rules

- A group can have any combination of: direct `member` entries, `include` references, and a `resolver`
- Membership is the **union** of all sources (direct members ∪ included groups ∪ resolver)
- Included groups can themselves have resolvers — an onchain group can be included in a static group
- `listMembers()` flattens recursively

### Safety

- **Cycle detection**: mandatory at config parse time. If A includes B and B includes A → config is invalid, server rejects it.
- **Max depth**: 5 levels. Config with deeper nesting is rejected.
- **Resolver timeouts**: if an included group's resolver is slow/unreachable, the membership check for that branch fails closed (not a member).

## Server-Side API

The server exposes group operations for CLI and web UI:

```
GET /api/repos/:owner/:repo/groups                    → list all groups
GET /api/repos/:owner/:repo/groups/:group              → group info + members
GET /api/repos/:owner/:repo/groups/:group/check/:addr  → { "member": bool }
```

The CLI uses the `check` endpoint for pre-flight access verification before pushing.

## What Groups Does NOT Cover

- **What can a group do?** — That's permissions (03-permissions.md)
- **Who created this group?** — The person who committed the `.repobox-config` change
- **Governance / voting** — Out of scope. Groups are just membership lists.
