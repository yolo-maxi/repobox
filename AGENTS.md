# AGENTS.md — AI Agent Integration Guide

> Everything an AI agent needs to know to work with repo.box.

## What is repo.box?

repo.box gives AI agents their own cryptographic identities in git. Each agent gets an EVM keypair, commits are signed, and a `.repobox/config.yml` file controls exactly what each agent can do.

No more "give the agent full SSH access" — fine-grained permissions per agent, per branch, per file.

## Quick Start for Agents

### 1. Get repo.box CLI
```bash
curl -sSf https://repo.box/install.sh | sh
```

### 2. Generate Your Identity  
```bash
repobox keys generate --alias your-agent-name
# → Generates EVM keypair, saves to ~/.repobox/keys/
# → Your address is now your git identity
```

### 3. Initialize in a Repo
```bash
git clone your-repo.git
cd your-repo
repobox init
# → Creates .repobox/config.yml template
# → Sets up git config to use repobox for signing
```

### 4. Configure Permissions
Edit `.repobox/config.yml`:
```yaml
groups:
  founders:
    - evm:0xYourFounderAddress...
  agents:
    - evm:0xYourAgentAddress...

permissions:
  default: allow
  rules:
    - founders own >*                    # founders control everything
    - agents push >feature/**            # agents push to feature branches
    - agents edit * >feature/**          # agents edit files on feature branches  
    - agents not edit ./.repobox/config.yml  # agents cannot edit config
```

### 5. Work Normally
```bash
git checkout -b feature/your-feature
# Edit files...
git add .
git commit -S -m "your commit message"  # -S flag for signing
git push origin feature/your-feature
```

## Key Concepts for Agents

### Identity = EVM Address
- Your identity is your Ethereum address (e.g. `evm:0xAAA...123`)
- Private key stored in `~/.repobox/keys/your-address.key`
- Commits are cryptographically signed and verifiable
- No SSH keys, no GitHub tokens needed

### Permission Model
- **Branch verbs**: `push`, `merge`, `branch`, `delete`, `force-push`
- **File verbs**: `read`, `edit`, `write`, `append`, `create`  
- **Targets**: File paths and branch patterns
- **First match wins**: Rules evaluated top-to-bottom

### Common Permission Patterns

**Agent on feature branches only:**
```yaml
rules:
  - agents push >feature/**
  - agents edit * >feature/**
```

**Read-only access:**
```yaml
rules:
  - agents read *
```

**Append-only logging:**
```yaml
rules:
  - agents append ./logs/**
```

**Prevent config changes:**
```yaml
rules:
  - agents not edit ./.repobox/config.yml
```

## Agent Development Workflow

### 1. Request Access
Ask the repo owner to add your agent address to the appropriate group in `.repobox/config.yml`:

```yaml
groups:
  agents:
    - evm:0xYourAgentAddress...  # ← Add this line
```

### 2. Work on Feature Branches
Always create feature branches for your work:
```bash
git checkout -b feature/agent-improvement-xyz
```

### 3. Test Permissions Locally
```bash
# Check if you can do something before trying
repobox check $(repobox whoami) push >feature/test
# ✅ allowed or ❌ denied
```

### 4. Commit with Signatures
```bash
git commit -S -m "descriptive commit message"
# The -S flag is critical — enables signature verification
```

## Advanced Features

### ENS Support
Instead of raw addresses, you can use ENS names:
```yaml
groups:
  agents:
    - claude.eth
    - codex.eth
```

### Token-Gated Groups
Membership based on token holdings:
```yaml
groups:
  premium-agents:
    resolver: onchain
    chain: 8453  # Base
    contract: "0xTokenContractAddress"
    function: balanceOf
    cache_ttl: 300
```

### X402 Paid Access
Repositories can require payment for access:
```yaml
x402:
  read_price: "1.00"  # 1 USDC
  recipient: "0xRecipientAddress"
  network: "base"
```

## Troubleshooting

### Permission Denied
```bash
# Check your current permissions
repobox check $(repobox whoami) verb >target

# Example: can I push to main?
repobox check $(repobox whoami) push >main
```

### Signature Issues
```bash
# Verify you have a key
repobox keys list

# Check your identity 
repobox whoami
```

### Config Validation
```bash
# Lint the config file
repobox lint
```

## Integration Examples

### GitHub Actions
```yaml
- name: Setup repo.box
  run: |
    curl -sSf https://repo.box/install.sh | sh
    echo "${{ secrets.REPOBOX_PRIVATE_KEY }}" > ~/.repobox/keys/agent.key
    repobox init
```

### Docker
```dockerfile
RUN curl -sSf https://repo.box/install.sh | sh
COPY agent.key /root/.repobox/keys/
```

### Node.js
```javascript
const { execSync } = require('child_process');

// Generate agent identity
execSync('repobox keys generate --alias ci-agent');

// Get agent address
const address = execSync('repobox whoami').toString().trim();
console.log(`Agent identity: ${address}`);
```

## Security Best Practices

### Key Management
- Store private keys securely (GitHub Secrets, encrypted volumes)
- Never commit private keys to repositories
- Use distinct keys per agent/environment

### Permission Principle
- Grant minimum necessary permissions
- Use feature branch workflows  
- Regularly audit `.repobox/config.yml`

### Monitoring
- Monitor agent commits via git log
- Set up alerts for permission violations
- Use `repobox audit` for permission reports

## Getting Help

- **Documentation**: https://repo.box/SKILL.md
- **Source**: https://github.com/yolo-maxi/repobox
- **Issues**: Report bugs with specific error messages
- **CLI Help**: Run `repobox --help` for command reference

---

*This guide covers everything agents need to work safely with git repositories using repo.box. For repository owners, see the main documentation at repo.box.*