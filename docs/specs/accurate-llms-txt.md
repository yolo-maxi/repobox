# Accurate llms.txt Specification - Critical for Agent Judges

**Task**: Update `repo.box/llms.txt` to be fully accurate and current for hackathon agent-judges

**Priority**: Critical - this file will be read by AI judges to understand repo.box capabilities

**Context**: The current `llms.txt` contains inaccuracies and describes some features that don't work yet or are implemented differently than documented.

## Current Issues Identified

### 1. Incorrect Verbs Listed

**Current llms.txt says:**
```
- **`edit`** — Modify existing files  
- **`write`** — Overwrite files completely
- **`append`** — Add content to end of files
- **`create`** — Create new files
```

**Actual implementation (from config.rs):**
- `read` (access verb)
- `push`, `merge`, `branch`, `delete`, `force-push` (branch verbs) 
- `edit`, `insert`, `append`, `upload` (file verbs)
- `write` and `create` are **deprecated aliases** for `upload` (warns users)

### 2. Config Path Error

**Current llms.txt says:** `.repobox/config.yml`
**Actual path:** `.repobox/config.yml` ✅ (this is correct)

### 3. Missing/Incorrect Verb Details

**Missing verbs that are implemented:**
- `insert` - Add lines to files (between `edit` and `append` in hierarchy)
- `upload` - Create new files (replaces deprecated `create`/`write`)

**Incorrect verb descriptions:**
- Current says `own` checks "all verbs" - actually checks 8 specific verbs: push, merge, create, delete, force-push, edit, write, append

### 4. ENS Implementation Status

**Current status:** FULLY IMPLEMENTED ✅
- Both explicit (`ens:vitalik.eth`) and implicit (`vitalik.eth`) syntax supported
- Works in groups and rules
- CLI supports ENS names: `git repobox check vitalik.eth push main`
- Resolution via ALCHEMY_API_KEY with 60-second caching
- 165 tests passing including ENS integration tests

### 5. X402 Payment Implementation Status

**Current status:** IMPLEMENTED ✅
- Config supports x402 section with read_price, recipient, network
- Server implements payment flow via routes
- Credential helper supports x402 authentication  
- Integrated with repository access control

### 6. Force Push Policy 

**Current llms.txt says:** Generic mention
**Actual implementation:** 
- `force-push` is a distinct verb separate from `push`
- Can be controlled per-branch: `- agents force-push >feature/**`
- Git hooks validate force-push attempts on agent branches
- Virtuals integration specifically blocks force-push on `agent/` branches

### 7. Explorer Features

**Current status:** FULLY IMPLEMENTED ✅
- Working explorer at `/explore` route
- AddressDisplay component with ENS resolution
- Repository browsing with commit history
- Address and repository pages with human-readable URLs
- Integration with ENS names in URLs (`/explore/vitalik.eth/`)

### 8. Credential Helper Flow

**Current llms.txt mentions:** Generic credential helper
**Actual implementation:**
- Implements `git credential-helper` interface  
- Signs authentication with EVM key: `keccak256("{repo_path}:{timestamp}")` 
- Uses signature + timestamp as password for repo.box hosts
- Integrates with x402 payment verification

### 9. Stats and Current Numbers

**Current llms.txt says:** "160+ Rust tests passing"
**Actual status:** Need to verify current test count (cargo not available in audit)

**Current llms.txt says:** "161 commits in main repo"  
**Actual status:** Need to verify current commit count

### 10. Virtuals Integration Status

**Current status:** FULLY IMPLEMENTED ✅
- Complete bug bounty system for AI agents
- Agent branch naming validation: `agent/{agent-id}/fix-{issue-number}`
- Commit message validation with issue references
- Payment processing via post-receive hooks
- Treasury and gas sponsor configuration
- Comprehensive test suite in `virtuals_integration_test.rs`

### 11. Missing Implementation Details

**Not clearly documented but implemented:**
- Verb hierarchy for file operations: edit > insert > append > upload
- Remote group resolvers (HTTP and on-chain)
- Comprehensive git hook system (pre-commit, pre-push, pre-receive, post-receive)
- Shim vs GPG program modes in CLI
- Identity alias system for human-readable agent names

## Recommended Updates

### 1. Fix Verb Documentation

Replace current verb list with accurate implementation:

```markdown
## Available Verbs

repo.box supports these permission verbs exactly:

### Access Verbs
- **`read`** — Clone repo, view files, fetch branches

### Branch Verbs  
- **`push`** — Push commits to branches
- **`merge`** — Merge branches (pull requests)
- **`branch`** — Create new branches
- **`delete`** — Delete branches
- **`force-push`** — Rewrite git history (distinct from push)

### File Verbs (Hierarchy: edit > insert > append > upload)
- **`edit`** — Modify existing files (highest permission)
- **`insert`** — Add lines within files
- **`append`** — Add content to end of files  
- **`upload`** — Create new files (replaces deprecated create/write)

### Notes
- `write` and `create` are deprecated aliases for `upload` (CLI shows warning)
- `own` checks these 8 verbs: push, merge, create, delete, force-push, edit, write, append
```

### 2. Add Force Push Policy Details

```markdown
## Force Push Policy

Force push is a separate verb from regular push:

```yaml
rules:
  - founders force-push >*              # Allow force push everywhere
  - agents push >feature/**             # Allow regular push on feature branches
  - agents force-push >feature/**       # Also allow force push on feature branches  
  - agents not force-push >main         # Explicitly deny force push to main
```

- Git hooks validate force-push attempts
- Virtuals integration blocks force-push on `agent/` branches
- Use for repository cleanup and history management
```

### 3. Update ENS Section

```markdown
## ENS Support (Fully Implemented)

- **Syntax**: `ens:vitalik.eth` or `vitalik.eth` (implicit detection)
- **Resolution**: Via ALCHEMY_API_KEY with 60-second caching  
- **CLI support**: `git repobox check vitalik.eth push main`
- **URLs**: Human-readable explorer URLs work: `/explore/vitalik.eth/`
- **Groups**: Mix EVM and ENS identities freely
- **Fail-closed**: Resolution errors deny permission

Requirements: Set `ALCHEMY_API_KEY` environment variable.
```

### 4. Add Virtuals Integration

```markdown
## Virtuals Integration (AI Bug Bounties)

repo.box includes full integration with Virtuals protocol for AI agent bug bounties:

```yaml
virtuals:
  enabled: true
  bug_bounties:
    critical: "100.00"
    high: "25.00"  
    medium: "10.00"
    low: "5.00"
  payments:
    network: "base"
    token: "USDC"
    treasury: "0x..."
```

### Agent Requirements
- Branch naming: `agent/{agent-id}/fix-{issue-number}`
- Commit format: `fix: description fixes #42`  
- No force-push on agent branches
- Human review before merge

### Payment Flow
1. Agent submits fix via proper branch/commit format
2. Human reviews and merges to main
3. Post-receive hook automatically processes payment
4. Bounty transferred to agent's EVM address
```

### 5. Update Current Stats

Need to determine current accurate numbers for:
- Test count: `cargo test --list | wc -l`
- Commit count: `git rev-list --count main`  
- Source files: `find repobox-*/src -name "*.rs" | wc -l`

### 6. Add Implementation Architecture

```markdown
## Architecture Details

### Three-Binary System
- **repobox-core**: Permission engine, config parsing, ENS resolution
- **repobox-cli**: Git shim, hooks, GPG program interface  
- **repobox-server**: HTTP API, git hosting, x402 payments

### Git Integration Modes
1. **Shim mode**: Transparent git command interception
2. **GPG program**: EVM-signed commits via git config
3. **Hooks**: Pre/post validation and payment processing
4. **Credential helper**: x402 payment authentication

### Security Model
- Local-first: All permission checks happen locally
- Fail-closed: Errors deny access by default  
- EVM signatures: Every commit cryptographically verifiable
- Git hosting agnostic: Works with GitHub, GitLab, self-hosted
```

## Files to Update

1. **Primary**: `/llms.txt` - main file agent judges will read
2. **Secondary**: `README.md` - keep consistent with llms.txt
3. **Reference**: `docs/SKILL.md` - technical documentation

## Verification Steps

Before publishing updates:

1. **Test current CLI commands** mentioned in llms.txt
2. **Verify all URLs** work and return expected content  
3. **Check current stats** (tests, commits, files)
4. **Validate configuration examples** parse correctly
5. **Test ENS resolution** with real names
6. **Confirm explorer features** work as described

## Success Criteria

Agent judges reading the updated `llms.txt` should have:
- **Accurate understanding** of all repo.box capabilities  
- **Correct syntax examples** for configuration
- **Working URLs** for exploration and demos
- **Clear implementation status** - what works vs. what's planned
- **Complete feature coverage** - no missing major functionality

The file must serve as the definitive, machine-readable specification of repo.box capabilities for AI agent evaluation.