# Agent Onboarding

Comprehensive guide for setting up AI agents with repo.box. Covers identity generation, permission configuration, and integration patterns.

## Agent Identity Setup

### Generate Agent Keys

Each AI agent needs its own EVM keypair:

```bash
# Generate a keypair for your agent
git repobox keys generate --alias claude-agent

# Generate keys for multiple agents
git repobox keys generate --alias codex-agent
git repobox keys generate --alias review-bot
```

### Sub-Agent Support

repo.box supports sub-agents with the `+` notation:

```bash
# Generate keys for specialized sub-agents
git repobox keys generate --alias claude+bugfix
git repobox keys generate --alias claude+docs
git repobox keys generate --alias claude+tests
```

This allows one main agent to spawn specialized sub-agents with different permission profiles.

## Agent Permission Patterns

### Basic Agent Setup

Minimal configuration for a single agent:

```yaml
groups:
  humans:
    - alice.eth
  agents:
    - evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00  # claude-agent

permissions:
  default: deny
  rules:
    - humans own >*                     # Full control for humans
    - agents read *                     # Agents can read everything
    - agents push >feature/**           # Agents push to feature branches
    - agents edit * >feature/**         # Agents edit files on feature branches
```

### Multi-Agent Team

Configuration for multiple specialized agents:

```yaml
groups:
  humans:
    - alice.eth
    - bob.eth
  
  # Different agent types with different capabilities
  dev-agents:
    - evm:0xAAc0...  # claude-agent (main developer)
    - evm:0x8224...  # codex-agent (specialized coder)
  
  review-agents:
    - evm:0xe4D4...  # review-bot
  
  doc-agents:
    - evm:0xF1A2...  # doc-writer

permissions:
  default: deny
  rules:
    # Human maintainers
    - humans own >*
    
    # Development agents
    - dev-agents push >feature/**
    - dev-agents push >fix/**
    - dev-agents branch feature/**
    - dev-agents branch fix/**
    - dev-agents edit src/** >feature/**
    - dev-agents edit src/** >fix/**
    
    # Review agents (read-only for analysis)
    - review-agents read *
    
    # Documentation agents
    - doc-agents edit docs/** >*        # Edit docs on any branch
    - doc-agents edit README.md >*      # Update README anywhere
    - doc-agents push >docs/**          # Push to docs branches
```

### Sub-Agent Specialization

Configure different permissions for sub-agents:

```yaml
groups:
  humans:
    - alice.eth
  
  # Main agent with broad permissions
  claude-main:
    - evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00
  
  # Specialized sub-agents
  claude-bugfix:
    - evm:0x1234...  # claude+bugfix
  
  claude-docs:
    - evm:0x5678...  # claude+docs

permissions:
  default: deny
  rules:
    - humans own >*
    
    # Main agent: broad development access
    - claude-main push >feature/**
    - claude-main edit * >feature/**
    
    # Bugfix specialist: can work on any branch for critical fixes
    - claude-bugfix push >hotfix/**
    - claude-bugfix edit src/** >hotfix/**
    - claude-bugfix edit tests/** >hotfix/**
    
    # Docs specialist: documentation anywhere
    - claude-docs edit docs/** >*
    - claude-docs edit *.md >*
    - claude-docs push >docs/**
```

## Agent Workflow Patterns

### Feature Development Workflow

Standard pattern for agent feature development:

```yaml
permissions:
  rules:
    # 1. Agent creates feature branch
    - agents branch feature/**
    
    # 2. Agent develops on feature branch
    - agents edit src/** >feature/**
    - agents edit tests/** >feature/**
    - agents push >feature/**
    
    # 3. Agent updates documentation
    - agents edit docs/** >feature/**
    - agents append CHANGELOG.md
    
    # 4. Human reviews and merges
    - humans merge >main
    - humans delete >feature/**  # Cleanup after merge
```

### Bug Bounty Workflow (Virtuals Integration)

Configure agents for bug bounty hunting:

```yaml
groups:
  bounty-hunters:
    - evm:0xAAc0...  # claude-agent
    - evm:0x8224...  # codex-agent

# Enable Virtuals integration
virtuals:
  enabled: true
  bug_bounties:
    critical: "50.00"
    high: "25.00"
    medium: "10.00" 
    low: "5.00"
  payments:
    network: "base"
    token: "USDC"
    treasury: "0x..."

permissions:
  default: deny
  rules:
    # Bounty hunters work on agent branches only
    - bounty-hunters branch agent/**
    - bounty-hunters push >agent/**
    - bounty-hunters edit * >agent/**
    
    # Agents cannot force-push (required for bounty validation)
    - bounty-hunters not force-push >agent/**
    
    # Humans review and merge for payment
    - humans merge >main
```

**Agent branch naming requirement:** `agent/{agent-id}/fix-{issue-number}`

Example: `agent/0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00/fix-42`

## Agent Integration

### Providing Keys to Agents

**Option 1: Direct key sharing** (Most secure)
```bash
# Generate key
git repobox keys generate --alias claude-agent

# Extract private key for agent
cat ~/.repobox/keys/claude-agent_private.pem
```

Share this private key securely with your agent system.

**Option 2: Agent key generation** (If agent supports it)
```bash
# Agent generates its own key and provides the address
# You add it to the configuration manually
```

### Agent Environment Setup

Agents need to configure their git environment:

```bash
# Set agent identity
git repobox use claude-agent

# Or set via environment variable
export REPOBOX_IDENTITY="evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00"

# Configure git for EVM signing
git config user.name "Claude Agent"
git config user.email "claude@example.com"
git config user.signingkey "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00"
git config gpg.program "repobox"
git config commit.gpgsign true
```

### Permission Checking in Agent Code

Agents can check permissions before attempting operations:

```bash
# Check if agent can perform an operation
if git repobox check claude-agent push feature/new-api; then
    # Proceed with push
    git push origin feature/new-api
else
    # Handle permission denial
    echo "Permission denied for push to feature/new-api"
fi
```

## Agent Verification Commands

### Verify Agent Setup
```bash
# Check agent identity
git repobox whoami

# Test basic permissions
git repobox check claude-agent read *
git repobox check claude-agent push feature/test
git repobox check claude-agent edit src/lib.rs >feature/test
```

### Debug Permission Issues
```bash
# Show full status
git repobox status

# Validate configuration
git repobox lint

# Check specific permission with details
git repobox check claude-agent push main  # Should show DENY with reason
```

## Security Considerations

### Agent Key Management
- **Store keys securely:** Treat EVM private keys like SSH private keys
- **One key per agent:** Don't share keys between different agent instances
- **Key rotation:** Generate new keys periodically for long-running agents
- **Backup keys:** Store private keys in secure backup systems

### Permission Boundaries
- **Least privilege:** Give agents only the permissions they need
- **Branch isolation:** Use feature branches to limit agent impact
- **Critical file protection:** Explicitly deny access to config files and secrets
- **Human oversight:** Require human approval for merging to main branches

### Audit Trail
Every agent action is cryptographically signed and traceable:

```bash
# Verify agent commits
git verify-commit HEAD

# Show commit signature details
git log --show-signature

# Check who made specific changes
git log --pretty=format:"%h %an %s" --grep="feature"
```

## Common Agent Patterns

### CI/CD Agent
```yaml
groups:
  ci-agents:
    - evm:0xCI...

permissions:
  rules:
    - ci-agents read *                    # Read code for building
    - ci-agents append build-logs.txt     # Write build results
    - ci-agents push >release/**          # Push release candidates
    - ci-agents not edit src/**           # Cannot modify source
```

### Documentation Agent
```yaml
groups:
  doc-agents:
    - evm:0xDOC...

permissions:
  rules:
    - doc-agents edit docs/** >*          # Edit docs on any branch
    - doc-agents edit *.md >*             # Edit README files
    - doc-agents append CHANGELOG.md      # Update changelog
    - doc-agents push >docs/**            # Push docs branches
```

### Security Review Agent
```yaml
groups:
  security-agents:
    - evm:0xSEC...

permissions:
  rules:
    - security-agents read *              # Read all code for analysis
    - security-agents append security-report.md  # Write findings
    - security-agents not edit src/**     # Cannot modify code
    - security-agents not push >*         # Cannot push anything
```

## Next Steps

**Learn permission details:** Continue to [Permission System](../user-guide/permission-system.md) for advanced configuration.

**Explore agent workflows:** Check [Agent Workflows](../agent-guide/agent-workflows.md) for practical patterns.

**Set up bounty hunting:** See [Bounty Hunting](../agent-guide/bounty-hunting.md) for Virtuals integration.

**Having issues?** Check [Troubleshooting](../user-guide/troubleshooting.md).