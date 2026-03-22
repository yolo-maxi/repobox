# ENS Integration

Complete guide to using ENS names in repo.box permissions for human-readable identities.

## Overview

Instead of cryptic EVM addresses like `evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661`, you can use readable ENS names like `vitalik.eth` in your permission configurations.

**✅ Production Ready:** ENS integration is fully implemented with comprehensive testing (165 tests passing including 7 ENS-specific tests).

## Setup

### Required: Alchemy API Key

ENS resolution requires an Alchemy API key for Ethereum network access:

```bash
export ALCHEMY_API_KEY="your-alchemy-api-key-here"
```

**Get an Alchemy API key:**
1. Sign up at [alchemy.com](https://alchemy.com)
2. Create a new app on Ethereum Mainnet
3. Copy your API key
4. Add to your environment

**Make it persistent** by adding to your shell profile:
```bash
echo 'export ALCHEMY_API_KEY="your-key-here"' >> ~/.bashrc
source ~/.bashrc
```

### Verify ENS Setup

Test that ENS resolution works:

```bash
git repobox check vitalik.eth read *
# → ✓ ALLOW: vitalik.eth can read * (if configured)
# or
# → ✗ DENY: No matching rule (if not configured)
```

If you see an ENS resolution error, check your API key:
```bash
echo $ALCHEMY_API_KEY
```

## Usage

### In Groups

Use ENS names directly in group definitions:

```yaml
groups:
  maintainers:
    - vitalik.eth          # Implicit ENS detection
    - ens:alice.eth        # Explicit prefix (optional)
    - evm:0x123...         # Mix with EVM addresses
  
  agents:
    - claude.eth
    - ens:codex.eth

permissions:
  default: deny
  rules:
    - maintainers own >*
    - agents push >feature/**
```

### In Rules

Use ENS names directly in permission rules:

```yaml
permissions:
  rules:
    # Specific ENS identity
    - vitalik.eth push >main
    
    # Group with ENS names
    - maintainers merge >develop
    
    # Mixed identities in rules
    - "alice.eth push >feature/**"
    - "evm:0x456... push >feature/**"
```

### CLI Commands

All CLI commands accept ENS names:

```bash
# Check permissions for ENS name
git repobox check vitalik.eth push main
git repobox check alice.eth edit contracts/token.sol
git repobox check claude.eth own develop

# Generate alias for ENS name
git repobox alias set claude.eth claude-agent

# Set identity to ENS name (if you own the private key)
git repobox use alice.eth
```

## ENS Name Formats

### Automatic Detection
repo.box automatically detects ENS names by TLD:

```yaml
groups:
  team:
    # All automatically detected as ENS
    - vitalik.eth
    - alice.box
    - company.com
    - project.xyz
    - dev.org
    - api.io
    - service.dev
    - app.app
```

**Supported TLDs:** `.eth`, `.box`, `.com`, `.xyz`, `.org`, `.io`, `.dev`, `.app`

### Explicit Prefix
Use `ens:` prefix for clarity or unsupported TLDs:

```yaml
groups:
  team:
    - ens:custom.name      # Explicit ENS prefix
    - ens:vitalik.eth      # Redundant but clear
```

### Mixed Identity Types
Freely mix ENS names and EVM addresses:

```yaml
groups:
  diverse-team:
    - alice.eth                                    # ENS name
    - evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661  # EVM address
    - bob.box                                      # ENS name
    - maintainers                                  # Group reference
```

## Resolution Behavior

### Caching
- **Cache duration:** 60 seconds (configurable)
- **Performance:** First lookup takes ~100-300ms, subsequent lookups are instant
- **Automatic refresh:** Cache expires automatically

### Error Handling
- **Resolution failure:** Permission denied (fail-closed security)
- **Network issues:** Permission denied (fail-closed security)
- **Invalid names:** Configuration error at startup

Example error scenarios:
```bash
git repobox check nonexistent.eth push main
# → ✗ DENY: ENS resolution failed for nonexistent.eth

git repobox check invalid..eth push main  
# → Error: Invalid ENS name format
```

### Resolution Cache
Check resolution status:
```bash
# Shows cached resolutions and TTL
git repobox status
# → Identity: alice (evm:0x7D5b...)
# → ENS Cache: 3 entries, oldest expires in 45s
```

## Advanced Configuration

### On-Chain Group Resolution
Combine ENS with on-chain group membership:

```yaml
groups:
  # Static ENS members
  core-team:
    - vitalik.eth
    - alice.eth
  
  # Dynamic on-chain membership
  token-holders:
    resolver: onchain
    chain: 1                    # Ethereum mainnet
    contract: "token.eth"       # ENS name for contract!
    function: balanceOf
    cache_ttl: 300

permissions:
  rules:
    - core-team own >*
    - token-holders push >community/**
```

**Note:** Contract addresses can also be ENS names that resolve to addresses.

### HTTP Group Resolution with ENS
```yaml
groups:
  # API returns ENS names
  company-employees:
    resolver: http
    url: https://api.company.com/groups/employees
    cache_ttl: 3600
    
    # API response format:
    # {
    #   "members": [
    #     "alice.eth",
    #     "bob.eth", 
    #     "evm:0x123..."
    #   ]
    # }
```

## Real-World Examples

### DAO Permission Structure
```yaml
groups:
  # Core DAO members
  dao-council:
    - founder.eth
    - lead.eth
    - treasury.eth
  
  # Verified contributors
  contributors:
    resolver: onchain
    chain: 1
    contract: "dao.eth"           # DAO membership NFT
    function: balanceOf
    cache_ttl: 600
  
  # AI agents with ENS names
  ai-workforce:
    - claude.eth
    - codex.eth
    - docs.eth

permissions:
  default: deny
  rules:
    - dao-council own >*
    - contributors push >feature/**
    - contributors edit src/** >feature/**
    - ai-workforce push >ai/**
    - ai-workforce edit * >ai/**
```

### Company Repository
```yaml
groups:
  # Human employees
  engineering:
    - alice.company.com
    - bob.company.com
    - team-lead.company.com
  
  # AI agents with company domain
  ai-team:
    - claude.company.com
    - codex.company.com
  
  # External contractors
  contractors:
    - freelancer1.eth
    - consultant.xyz

permissions:
  default: deny
  rules:
    - engineering own >*
    - ai-team push >feature/**
    - ai-team edit src/** >feature/**
    - contractors read *
    - contractors edit docs/** >*
```

### Multi-Project Organization
```yaml
groups:
  # Core maintainers across projects
  maintainers:
    - lead.project.org
    - senior.project.org
  
  # Project-specific teams  
  frontend-team:
    - alice.frontend.project.org
    - ui-agent.project.org
  
  backend-team:
    - bob.backend.project.org
    - api-agent.project.org

permissions:
  rules:
    - maintainers own >*
    - frontend-team edit frontend/** >*
    - frontend-team push >frontend/**
    - backend-team edit backend/** >*
    - backend-team push >backend/**
```

## ENS Name Management

### For Repository Owners

**Setting up ENS names for agents:**
1. Register ENS names for your agents (e.g., `claude.yourproject.eth`)
2. Point them to your agent's EVM addresses
3. Use the ENS names in configurations for readability

**ENS subdomains:** You can use subdomains for organization:
```
- dev.yourproject.eth      (development agents)
- docs.yourproject.eth     (documentation agents) 
- security.yourproject.eth (security review agents)
```

### For Agent Operators

**If you control an ENS name:**
1. Point your ENS name to your agent's EVM address
2. Provide the ENS name to repository owners
3. They can use `yourname.eth` instead of `evm:0x...`

**If you don't own an ENS name:**
- Continue using EVM addresses
- Repository owners can create aliases for readability

## CLI Reference

### Check Commands
```bash
# Check permission for ENS name
git repobox check vitalik.eth push main
git repobox check alice.eth edit src/contract.sol >feature/upgrade

# Check with explicit prefix
git repobox check ens:custom.name read *
```

### Identity Commands
```bash
# Set identity to ENS name (if you have the private key)
git repobox use alice.eth

# Create alias for ENS name
git repobox alias set alice.eth alice

# Use the alias
git repobox use alice
```

### Status Commands
```bash
# Show ENS resolution cache
git repobox status
# → ENS Cache: 3 entries
# →   vitalik.eth → 0xd8dA... (expires in 45s)
# →   alice.eth → 0x1234... (expires in 30s)
```

## Troubleshooting

### ENS Resolution Failures

**"ENS resolution failed" errors:**
1. **Check API key:** `echo $ALCHEMY_API_KEY`
2. **Verify name exists:** Check on [ens.domains](https://ens.domains)
3. **Network connectivity:** Test with `curl https://eth-mainnet.alchemyapi.io/v2/$ALCHEMY_API_KEY`

### Configuration Errors

**"Invalid ENS name" errors:**
```bash
git repobox lint
# → Error: Invalid ENS name 'invalid..eth' in group 'team'
```

Common issues:
- Double dots: `invalid..eth`
- Invalid characters: `bad name.eth`
- Unsupported TLD without `ens:` prefix

### Performance Issues

**Slow permission checks:**
- ENS resolution takes ~100-300ms on first lookup
- Subsequent lookups use cache (60s TTL)
- Use EVM addresses for performance-critical applications

## Security Considerations

### Fail-Closed Security
- ENS resolution failure = permission denied
- Network issues = permission denied
- Invalid names = configuration error

This ensures that temporary ENS issues don't accidentally grant unauthorized access.

### ENS Name Control
- Only use ENS names you trust the owner of
- ENS ownership can change (though rare for established names)
- Consider using EVM addresses for highest security

### API Key Security
- Alchemy API key only resolves ENS names to addresses
- No sensitive repository data is transmitted
- Use read-only API keys when possible

## Limitations

### Current Limitations
- **Mainnet only:** ENS resolution only works on Ethereum mainnet
- **TTL fixed:** Cache TTL is currently fixed at 60 seconds
- **No reverse resolution:** Cannot lookup ENS name from address

### Workarounds
- Use `git repobox status` to see address mappings
- Create aliases for frequently used addresses
- Mix ENS names and addresses as needed

## Next Steps

**Learn payment integration:** Continue to [Payment System](payment-system.md) for agent bounties.

**Explore advanced permissions:** See [Permission System](permission-system.md) for complex rule patterns.

**Set up agents:** Check [Agent Onboarding](../getting-started/agent-onboarding.md) for agent-specific setup.

**Having issues?** See [Troubleshooting](troubleshooting.md) for common solutions.