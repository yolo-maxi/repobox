# Permission System

Complete reference for repo.box permissions, covering rule syntax, evaluation order, and advanced patterns.

## Core Concepts

### Identities
Every user and agent is identified by an EVM address:
- **EVM format:** `evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661`
- **Short format:** `0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661` (evm: prefix optional)
- **ENS format:** `vitalik.eth` or `ens:vitalik.eth`
- **Aliases:** Local names like `alice` pointing to addresses

### Groups
Named collections of identities:
```yaml
groups:
  maintainers:
    - alice.eth
    - evm:0xAAA...
  agents:
    - claude.eth
    - evm:0xBBB...
```

### Permission Verbs

| Verb | Description | Git Commands |
|------|-------------|--------------|
| `read` | Clone repo, view files, fetch branches | `git clone`, `git pull`, `git fetch` |
| `push` | Push commits to branches | `git push` |
| `merge` | Merge branches | `git merge`, PR merges |
| `branch` | Create new branches | `git branch`, `git checkout -b` |
| `delete` | Delete branches | `git branch -d`, `git push -d` |
| `force-push` | Rewrite git history | `git push --force` |
| `edit` | Modify existing files | Any file modification |
| `insert` | Add lines within files | Insertions without deletions |
| `append` | Add to end of files | Append-only changes |
| `upload` | Create new files | New file creation |
| `own` | All of the above | Expands to all verbs |

**Verb Hierarchy:** `edit` > `insert` > `append` > `upload`
- Having `edit` permission includes `insert`, `append`, and `upload`
- Having `insert` permission includes `append` and `upload`
- Having `append` permission includes `upload`

### Targets

**Branch targets:**
- `>main` — Specific branch
- `>feature/**` — Glob pattern (any branch starting with "feature/")
- `>*` — All branches

**File targets:**
- `*` — All files
- `src/**` — All files in src/ directory
- `*.rs` — All Rust files
- `.repobox/config.yml` — Specific file

**Combined targets:**
- `* >feature/**` — Any file on feature branches
- `src/** >main` — Source files on main branch only

## Configuration Formats

### Format A: Flat Rules
Simple one-liner format:

```yaml
permissions:
  default: deny
  rules:
    - maintainers own >*
    - agents read *
    - agents push >feature/**
    - agents edit * >feature/**
    - agents not push >main
```

### Format B: Subject-Grouped
Group rules by subject:

```yaml
permissions:
  default: deny
  rules:
    maintainers:
      - own >*
    agents:
      - read *
      - push >feature/**
      - edit * >feature/**
      - not push >main
```

### Format C: Verb-Mapping
Organize by verb:

```yaml
permissions:
  default: deny
  rules:
    maintainers:
      own:
        - ">*"
    agents:
      read:
        - "*"
      push:
        - ">feature/**"
      edit:
        - "* >feature/**"
      push:  # Note: "not" prefix goes on the verb
        - "not >main"
```

### Mixed Format
You can combine approaches:

```yaml
permissions:
  rules:
    - maintainers own >*        # Flat rule
    - agents:                   # Subject-grouped
        push:
          - ">feature/**"
        edit:
          - "* >feature/**"
```

## Rule Evaluation

### First Match Wins
Rules are evaluated top-to-bottom. The first rule that matches the subject and verb determines the result:

```yaml
rules:
  - agents push >feature/special   # ✓ ALLOW (specific)
  - agents not push >feature/**    # This would deny, but never reached
  - agents push >feature/**        # ✓ ALLOW (general)
```

### Default Policy
When no rules match:
- `default: allow` — Grant permission
- `default: deny` — Deny permission (recommended)

```yaml
permissions:
  default: deny   # Secure by default
  rules:
    - maintainers own >*
    # All unmatched cases are denied
```

### Explicit Denials
Use `not` prefix to explicitly deny:

```yaml
rules:
  - agents not push >main          # Explicit deny
  - agents not edit .repobox/**    # Protect config directory
  - agents push >feature/**        # Allow feature branches
```

## Advanced Patterns

### File Protection
Protect critical files while allowing general access:

```yaml
permissions:
  default: deny
  rules:
    # Protect critical files first (higher priority)
    - maintainers edit .repobox/config.yml
    - maintainers edit package.json
    - maintainers edit Cargo.toml
    
    # General file access (lower priority)
    - agents edit src/** >feature/**
    - agents edit docs/** >*
    - agents not edit *  # Deny all other file edits
```

### Branch Workflows
Model common Git workflows:

```yaml
# GitFlow-style workflow
permissions:
  rules:
    # Production branch (maintainers only)
    - maintainers push >main
    - maintainers merge >main
    
    # Development branch
    - developers push >develop
    - developers merge >develop
    
    # Feature branches (anyone can create)
    - "* branch >feature/**"
    - "* push >feature/**"
    - "* edit * >feature/**"
    
    # Hotfix branches (senior devs only)
    - senior-devs branch >hotfix/**
    - senior-devs push >hotfix/**
    - senior-devs merge >main"  # Emergency fixes
```

### Role-Based Access

```yaml
groups:
  # Role definitions
  architects:
    - alice.eth
  senior-developers:
    - bob.eth
    - charlie.eth
    - architects  # Include architects in senior developers
  developers:
    - dave.eth
    - eve.eth
    - senior-developers  # Include senior developers
  agents:
    - claude.eth
    - codex.eth

permissions:
  default: deny
  rules:
    # Architects: Full control
    - architects own >*
    
    # Senior developers: Most operations
    - senior-developers push >main
    - senior-developers merge >develop
    - senior-developers delete >feature/**
    - senior-developers edit * >*
    
    # Regular developers: Feature work
    - developers branch >feature/**
    - developers push >feature/**
    - developers push >develop
    - developers edit * >feature/**
    
    # Agents: Supervised development
    - agents branch >feature/**
    - agents push >feature/**
    - agents edit src/** >feature/**
    - agents append docs/** >feature/**
```

## Group Definitions

### Static Groups
Simple lists of identities:

```yaml
groups:
  core-team:
    - alice.eth
    - evm:0xAAA...
    - evm:0xBBB...
```

### Group Inclusion
Include other groups:

```yaml
groups:
  maintainers:
    - alice.eth
    - bob.eth
  
  developers:
    - charlie.eth
    - dave.eth
  
  all-humans:
    - maintainers  # Include all maintainers
    - developers   # Include all developers
    - eve.eth      # Plus additional members
```

### Dynamic Groups (ENS + On-chain)
Groups resolved from external sources:

```yaml
groups:
  # Token holders (on-chain resolution)
  token-holders:
    resolver: onchain
    chain: 8453           # Base
    contract: "0x..."
    function: balanceOf   # Any non-zero return = member
    cache_ttl: 300
  
  # Company members (HTTP API)
  company:
    resolver: http
    url: https://api.company.com/groups/engineering
    cache_ttl: 60
```

**Supported chains:** Ethereum, Base, Optimism, Arbitrum, Polygon, and [40+ more](https://chainlist.org).

## Permission Testing

### CLI Testing
Test permissions before committing rules:

```bash
# Test specific permission
git repobox check alice push main
# → ✓ ALLOW: maintainers can push main

git repobox check claude edit src/lib.rs >feature/new-api
# → ✓ ALLOW: agents can edit * >feature/**

git repobox check claude push main
# → ✗ DENY: No matching rule (default: deny)
```

### Configuration Validation
Use the built-in linter:

```bash
git repobox lint
```

The linter checks for:
- Syntax errors in YAML
- Unknown group references
- Unreachable rules (shadowed by earlier rules)
- Empty groups
- Duplicate rules
- Invalid glob patterns

## Common Mistakes

### Rule Order Issues
❌ **Wrong:** Specific rules after general ones
```yaml
rules:
  - agents push >feature/**      # General rule matches first
  - agents not push >feature/protected  # Never reached!
```

✅ **Correct:** Specific rules first
```yaml
rules:
  - agents not push >feature/protected   # Specific denial first
  - agents push >feature/**              # General allow second
```

### Missing Default Policy
❌ **Wrong:** No explicit default
```yaml
permissions:
  rules:
    - maintainers own >*
    # What happens to unmatched rules? Unclear!
```

✅ **Correct:** Explicit default
```yaml
permissions:
  default: deny  # Clear default behavior
  rules:
    - maintainers own >*
```

### Overprivileged Agents
❌ **Wrong:** Too broad permissions
```yaml
rules:
  - agents edit * >*  # Agents can edit anything anywhere!
```

✅ **Correct:** Scoped permissions
```yaml
rules:
  - agents edit src/** >feature/**     # Limited scope
  - agents not edit .repobox/**        # Explicit protection
```

### Group Reference Typos
❌ **Wrong:** Typo in group name
```yaml
groups:
  maintainers:
    - alice.eth

permissions:
  rules:
    - maintainer own >*  # Typo: "maintainer" vs "maintainers"
```

The linter catches these errors:
```
Error: Unknown group 'maintainer' in rule 'maintainer own >*'
```

## Best Practices

### 1. Use Secure Defaults
```yaml
permissions:
  default: deny  # Always start with deny-by-default
```

### 2. Order Rules by Specificity
```yaml
rules:
  # Most specific first
  - agents not edit .repobox/config.yml
  - agents not push >main
  
  # General permissions last
  - agents edit * >feature/**
  - agents push >feature/**
```

### 3. Document Complex Rules
```yaml
permissions:
  rules:
    # Emergency hotfix workflow
    - oncall-engineers push >hotfix/**
    - oncall-engineers merge >main   # Bypass normal review for emergencies
    
    # Standard development workflow
    - developers push >feature/**
    - developers edit src/** >feature/**
```

### 4. Test Configurations
Always test new rules:

```bash
# Test the happy path
git repobox check agent-name push feature/test-branch

# Test the deny path
git repobox check agent-name push main

# Validate the entire config
git repobox lint
```

### 5. Use Meaningful Group Names
```yaml
groups:
  # Good: Clear purpose
  frontend-developers:
    - alice.eth
  backend-developers:
    - bob.eth
  ai-agents:
    - claude.eth
  
  # Avoid: Generic names
  group1:
    - ...
```

## Troubleshooting

### Permission Denied Errors
1. **Check current identity:**
   ```bash
   git repobox whoami
   ```

2. **Test specific permission:**
   ```bash
   git repobox check $(git repobox whoami | cut -d' ' -f1) push $(git branch --show-current)
   ```

3. **Review configuration:**
   ```bash
   git repobox lint
   git repobox status  # Shows applied rules
   ```

### Configuration Errors
1. **YAML syntax errors:** Use `git repobox lint` for specific line numbers
2. **Unknown groups:** Linter will list all undefined group references
3. **Unreachable rules:** Linter identifies rules that can never match

### ENS Resolution Issues
1. **Check API key:** `echo $ALCHEMY_API_KEY`
2. **Test resolution:** `git repobox check vitalik.eth read *`
3. **Cache issues:** ENS names are cached for 60 seconds

## Next Steps

**Learn ENS integration:** Continue to [ENS Integration](ens-integration.md) for readable identities.

**Set up payments:** See [Payment System](payment-system.md) for agent bounties.

**Explore agent patterns:** Check [Agent Workflows](../agent-guide/agent-workflows.md) for practical examples.

**Having issues?** See [Troubleshooting](troubleshooting.md) for common solutions.