# CLI Commands Reference

Complete reference for all repo.box CLI commands.

## Overview

repo.box provides a comprehensive command-line interface accessible through:
- `repobox <command>` — Direct binary usage
- `git repobox <command>` — Git integration
- `git rb <command>` — Shorthand alias

All commands support `--help` for detailed usage information.

## Repository Commands

### `init`
Initialize repo.box in an existing git repository.

```bash
git repobox init
```

**What it does:**
- Creates `.repobox/config.yml` with basic template
- Sets up git hooks directory structure
- Configures git to use repo.box as GPG program

**Output:**
```
Initialized repo.box in /path/to/repo
Created .repobox/config.yml
```

**Requirements:** Must be run inside a git repository.

## Identity Management

### `keys generate`
Generate a new EVM keypair.

```bash
git repobox keys generate [--alias <name>]
```

**Options:**
- `--alias <name>` — Set an alias for the generated key

**Examples:**
```bash
git repobox keys generate --alias alice
git repobox keys generate --alias claude-agent
git repobox keys generate  # No alias, use address directly
```

**Output:**
```
Generated new key pair
Address: evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661
Alias: alice
Private key stored in ~/.repobox/keys/alice_private.pem
```

**Storage:** Keys are stored in `~/.repobox/keys/` with 600 permissions.

### `keys list`
List all available EVM keypairs.

```bash
git repobox keys list
```

**Output:**
```
alice      evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661
bob        evm:0x8E6c79FA0d7C8CF2A19B18E912F8B9C7E6A95D38
claude     evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00
```

### `identity set`
Set your active identity.

```bash
git repobox identity set <key> [--alias <name>]
```

**Arguments:**
- `<key>` — EVM address or alias

**Options:**
- `--alias <name>` — Set or update alias for this identity

**Examples:**
```bash
git repobox identity set alice
git repobox identity set evm:0x7D5b... --alias alice
git repobox identity set 0x7D5b...  # evm: prefix optional
```

### `use`
Shorthand for `identity set`.

```bash
git repobox use <alias-or-address>
```

**Examples:**
```bash
git repobox use alice
git repobox use claude-agent  
git repobox use 0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661
```

### `whoami`
Show current active identity.

```bash
git repobox whoami
```

**Output:**
```
alice (evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661)
```

### `alias`
Manage local aliases for identities.

```bash
git repobox alias set <address> <alias>
git repobox alias list
git repobox alias remove <alias>
```

**Examples:**
```bash
# Set alias
git repobox alias set evm:0x7D5b... alice

# List all aliases  
git repobox alias list
# → alice      evm:0x7D5b...
# → claude     evm:0xAAc0...

# Remove alias
git repobox alias remove alice
```

## Permission Testing

### `check`
Check if an identity has permission to perform an action.

```bash
git repobox check <identity> <verb> <target>
```

**Arguments:**
- `<identity>` — EVM address, ENS name, or alias
- `<verb>` — Permission verb (push, edit, read, etc.)  
- `<target>` — Target (>branch, file pattern, etc.)

**Examples:**
```bash
# Check branch permissions
git repobox check alice push main
git repobox check claude-agent push feature/new-api

# Check file permissions  
git repobox check alice edit src/lib.rs
git repobox check claude edit contracts/** >feature/upgrade

# Check combined permissions
git repobox check alice edit src/** >main

# Use ENS names
git repobox check vitalik.eth push main
git repobox check alice.eth edit docs/README.md
```

**Output Examples:**
```
✓ ALLOW: maintainers can push main
✗ DENY: agents cannot push main (no matching rule)
✗ DENY: ENS resolution failed for invalid.eth
```

**Return Codes:**
- `0` — Permission granted (ALLOW)
- `1` — Permission denied (DENY)  
- `2` — Error (config invalid, resolution failed, etc.)

## Configuration Management

### `lint`
Validate `.repobox/config.yml` for errors and warnings.

```bash
git repobox lint
```

**What it checks:**
- YAML syntax and structure
- Unknown group references
- Unreachable rules (shadowed by earlier rules)
- Empty groups
- Duplicate rules
- Invalid glob patterns
- ENS name format validation

**Output Examples:**
```
✓ Configuration is valid
  Groups: 3 defined, all referenced
  Rules: 12 evaluated, no conflicts detected
  Identities: All EVM addresses valid

❌ Configuration has errors:
  Line 15: Unknown group 'typo-agents' in rule
  Line 23: Unreachable rule 'agents not push >feature/test'
  Line 31: Invalid glob pattern 'src/[invalid'

⚠️  Configuration has warnings:
  Line 18: Empty group 'unused-group'
  Line 25: Duplicate rule 'agents push >feature/**'
```

### `status`
Show comprehensive status including identity, groups, and permissions.

```bash
git repobox status
```

**Output Example:**
```
Identity: alice (evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661)

Groups:
  maintainers: alice, bob (2 members)
  agents: claude, codex (2 members)

Permissions (default: deny):
  1. maintainers own >*
  2. agents read *
  3. agents push >feature/**
  4. agents edit * >feature/**
  5. agents not push >main

Applied to current identity:
  ✓ Rule 1 matches: maintainers own >* (alice is in maintainers)

ENS Cache: 2 entries
  vitalik.eth → 0xd8dA... (expires in 45s)
  alice.eth → 0x1234... (expires in 30s)

Repository: /path/to/repo (.repobox/config.yml exists)
```

## Git Integration

### `setup`
Configure git to use repo.box as command interceptor.

```bash
git repobox setup
```

**What it configures:**
- Sets `gpg.program` to `repobox` for commit signing
- Creates `git rb` alias for shorthand access
- Sets up git hooks integration

**Git Config Changes:**
```bash
git config gpg.program "repobox"
git config alias.rb "!repobox"
git config commit.gpgsign true  # Optional
```

## Environment Variables

### `REPOBOX_IDENTITY`
Override active identity for a single command.

```bash
REPOBOX_IDENTITY="evm:0xAAA..." git repobox check alice push main
REPOBOX_IDENTITY="claude.eth" git commit -m "fix: update docs"
```

### `REPOBOX_SERVER`  
Set custom resolver server URL.

```bash
export REPOBOX_SERVER="https://your-server.com/api"
git repobox check token-holder push main  # Uses custom server
```

### `ALCHEMY_API_KEY`
Required for ENS name resolution.

```bash
export ALCHEMY_API_KEY="your-alchemy-key"
git repobox check vitalik.eth push main
```

### `RUST_LOG`
Enable debug logging for troubleshooting.

```bash
export RUST_LOG=debug
git repobox check alice push main  # Shows detailed rule evaluation
```

## Command Aliases

### Git Integration Shortcuts

```bash
# These are equivalent:
git repobox status
git rb status

# These are equivalent:
git repobox check alice push main  
git rb check alice push main

# Direct binary usage:
repobox status
```

### Common Command Patterns

```bash
# Quick identity switch
alias use-alice='git rb use alice'
alias use-claude='git rb use claude-agent'

# Quick permission testing
alias can-push='git rb check $(git rb whoami | cut -d" " -f1) push'
alias can-edit='git rb check $(git rb whoami | cut -d" " -f1) edit'

# Configuration validation
alias check-config='git rb lint && git rb status'
```

## Advanced Usage

### Batch Operations

Test multiple permissions:
```bash
#!/bin/bash
identities=("alice" "bob" "claude")
actions=("push >main" "edit src/**" "merge >develop")

for identity in "${identities[@]}"; do
    for action in "${actions[@]}"; do
        git repobox check "$identity" $action
    done
done
```

### Configuration Testing

Validate config changes before committing:
```bash
#!/bin/bash
# test-config.sh

# Backup current config
cp .repobox/config.yml .repobox/config.yml.bak

# Apply new config
cp new-config.yml .repobox/config.yml

# Test critical permissions
if git repobox check maintainers push main && \
   git repobox check agents not push main; then
    echo "✓ Config looks good"
else
    echo "❌ Config failed tests, rolling back"
    cp .repobox/config.yml.bak .repobox/config.yml
    exit 1
fi
```

### ENS Batch Resolution

Pre-warm ENS cache:
```bash
#!/bin/bash
ens_names=("vitalik.eth" "alice.eth" "project.eth")

for name in "${ens_names[@]}"; do
    git repobox check "$name" read "*" >/dev/null 2>&1
    echo "Cached $name"
done
```

## Exit Codes

All commands use consistent exit codes:

- **0** — Success
- **1** — Permission denied or logical failure  
- **2** — Configuration error or invalid input
- **3** — File system or network error
- **4** — Identity or key management error

Examples:
```bash
git repobox check alice push main
echo $?  # 0 = allowed, 1 = denied, 2 = config error

git repobox lint
echo $?  # 0 = valid, 2 = syntax errors

git repobox keys generate --alias alice
echo $?  # 0 = success, 4 = key generation failed
```

## Integration Examples

### CI/CD Integration

```bash
# In CI pipeline
if git repobox check ci-agent push main; then
    git push origin main
else
    echo "CI agent cannot push to main"
    exit 1
fi
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Validate config if it's being committed
if git diff --cached --name-only | grep -q ".repobox/config.yml"; then
    git repobox lint || {
        echo "❌ Invalid repo.box config"
        exit 1
    }
fi

# Check if current identity can push
current_branch=$(git branch --show-current)
current_identity=$(git repobox whoami | cut -d' ' -f1)

if ! git repobox check "$current_identity" push ">$current_branch"; then
    echo "❌ $current_identity cannot push to $current_branch"
    exit 1
fi
```

### Agent Deployment Script

```bash
#!/bin/bash
# deploy-agent.sh <agent-name> <private-key-path>

agent_name="$1"
key_path="$2"

# Import agent key
cp "$key_path" ~/.repobox/keys/${agent_name}_private.pem
chmod 600 ~/.repobox/keys/${agent_name}_private.pem

# Set as active identity
git repobox use "$agent_name"

# Configure git
git config user.name "$agent_name"  
git config user.email "${agent_name}@agents.example.com"
git config commit.gpgsign true

# Test permissions
if git repobox check "$agent_name" push ">feature/**"; then
    echo "✓ Agent $agent_name deployed successfully"
else
    echo "❌ Agent $agent_name has insufficient permissions"
    exit 1
fi
```

## Troubleshooting Commands

### Debug Permission Issues
```bash
# Show full status
git repobox status

# Test specific permission with verbose output
RUST_LOG=debug git repobox check alice push main

# Validate configuration  
git repobox lint

# Check identity setup
git repobox whoami
git repobox keys list
```

### Reset Configuration
```bash
# Backup and recreate config
mv .repobox/config.yml .repobox/config.yml.bak
git repobox init

# Restore git configuration
git repobox setup
```

### Clear ENS Cache
```bash
# ENS cache is automatically managed, but you can force refresh by waiting 60s
# or restarting the CLI
```

See [Troubleshooting Guide](../user-guide/troubleshooting.md) for complete problem-solving workflows.