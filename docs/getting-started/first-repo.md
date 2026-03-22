# First Repository Setup

Complete walkthrough for setting up your first repo.box repository with multiple users and basic agent permissions.

## Prerequisites

- repo.box installed ([Installation Guide](installation.md))
- A git repository (create one if needed: `git init my-project`)

## 1. Repository Initialization

Navigate to your repository and initialize repo.box:

```bash
cd my-project
git repobox init
```

This creates `.repobox/config.yml` with a minimal template. Let's build a real configuration step by step.

## 2. Generate Identities

Create keypairs for yourself and any agents:

```bash
# Your identity
git repobox keys generate --alias alice

# Your teammate's identity  
git repobox keys generate --alias bob

# An AI agent
git repobox keys generate --alias claude-agent
```

Check what was created:
```bash
git repobox keys list
# → alice    evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661
# → bob      evm:0x8E6c79FA0d7C8CF2A19B18E912F8B9C7E6A95D38
# → claude   evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00
```

## 3. Create a Realistic Permission Configuration

Edit `.repobox/config.yml` with a practical multi-user setup:

```yaml
# Define groups of users
groups:
  # Human maintainers with full access
  maintainers:
    - evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661  # alice
    - evm:0x8E6c79FA0d7C8CF2A19B18E912F8B9C7E6A95D38  # bob
  
  # AI agents with limited access
  agents:
    - evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00  # claude-agent

# Permission rules (evaluated top-to-bottom, first match wins)
permissions:
  default: deny  # Secure by default
  rules:
    # Maintainers have full control
    - maintainers own >*                    # All verbs, all branches
    
    # Agents can work on feature branches
    - agents push >feature/**               # Push to feature branches
    - agents branch feature/**              # Create feature branches
    - agents edit * >feature/**             # Edit any file on feature branches
    - agents read *                         # Read all files (for cloning)
    
    # Agents can append to certain files anywhere
    - agents append CHANGELOG.md            # Update changelog
    - agents append docs/README.md          # Update documentation
    
    # Explicit denials (redundant with default: deny, but clear intent)
    - agents not push >main                 # Cannot push to main
    - agents not merge >*                   # Cannot merge any branch
    - agents not edit .repobox/config.yml  # Cannot modify permissions
```

## 4. Test Permissions

Verify the configuration works as expected:

```bash
# Set your identity
git repobox use alice

# Test maintainer permissions
git repobox check alice push main
# → ✓ ALLOW: maintainers can push main

git repobox check alice edit .repobox/config.yml
# → ✓ ALLOW: maintainers can edit .repobox/config.yml

# Test agent permissions
git repobox check claude-agent push feature/new-api
# → ✓ ALLOW: agents can push feature/new-api

git repobox check claude-agent push main
# → ✗ DENY: agents cannot push main

git repobox check claude-agent edit src/lib.rs >feature/new-api
# → ✓ ALLOW: agents can edit * >feature/**
```

## 5. Validate Configuration

Use the built-in linter to check for common issues:

```bash
git repobox lint
# → ✓ Configuration is valid
#   Groups: 2 defined, all referenced
#   Rules: 7 evaluated, no conflicts detected
#   Identities: All EVM addresses valid
```

## 6. Commit the Setup

Add the configuration to your repository:

```bash
git add .repobox/
git commit -m "setup repo.box permissions

- Add maintainer and agent groups
- Configure feature branch workflow for agents
- Secure main branch with maintainer-only access"
```

## 7. Share Keys with Team Members

**For human team members:** They need to either:
1. Generate their own keys and you update the config with their addresses
2. Or you share the generated private key securely (not recommended)

**For AI agents:** Provide the private key to the agent system securely.

### Finding Private Keys
```bash
# List key files
ls ~/.repobox/keys/
# → alice_private.pem  bob_private.pem  claude-agent_private.pem

# View a private key (share this with the agent)
cat ~/.repobox/keys/claude-agent_private.pem
```

## 8. Test the Complete Workflow

Simulate an agent workflow:

```bash
# Switch to agent identity
git repobox use claude-agent

# Try to create a feature branch and make changes
git checkout -b feature/add-api-docs
echo "# API Documentation" > docs/api.md
git add docs/api.md
git commit -m "docs: add API documentation"

# This should work - agent can push to feature branches
git push origin feature/add-api-docs

# Try to push to main (should fail)
git checkout main
echo "unauthorized change" >> README.md
git add README.md
git commit -m "unauthorized change"
git push origin main
# → Error: Permission denied: agents cannot push main
```

## Common Patterns

### ENS Names (If Configured)
If you have ENS names, the config becomes more readable:

```yaml
groups:
  maintainers:
    - alice.eth
    - bob.eth
  agents:
    - claude.eth
```

### Mixed Identity Types
You can mix EVM addresses and ENS names freely:

```yaml
groups:
  core-team:
    - alice.eth                                    # ENS name
    - evm:0x8E6c79FA0d7C8CF2A19B18E912F8B9C7E6A95D38  # Raw address
    - maintainers                                  # Include another group
```

### File-Specific Permissions
Control access to specific files or directories:

```yaml
permissions:
  rules:
    # Only maintainers can edit critical config
    - maintainers edit package.json
    - maintainers edit Cargo.toml
    
    # Agents can only edit source code
    - agents edit src/**
    - agents edit tests/**
    
    # Anyone can edit documentation
    - "* edit docs/**"
```

## Next Steps

**Set up agent workflows:** Continue to [Agent Onboarding](agent-onboarding.md) for agent-specific configuration.

**Learn advanced patterns:** Check the [User Guide](../user-guide/permission-system.md) for complex permission scenarios.

**Having issues?** See [Troubleshooting](../user-guide/troubleshooting.md).

## Troubleshooting

**"Permission denied" when committing:** Check that your current identity has the required permissions:
```bash
git repobox whoami
git repobox check $(git repobox whoami | cut -d' ' -f1) push $(git branch --show-current)
```

**"Config parse error":** Run the linter for specific error messages:
```bash
git repobox lint
```

**"Key not found":** Verify your keys exist and your alias is set:
```bash
git repobox keys list
git repobox use alice  # Set identity explicitly
```