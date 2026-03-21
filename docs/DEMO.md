# repo.box Demo Guide

Complete guide for running the repo.box end-to-end demonstration.

## Quick Start

```bash
# Run the demo (30 seconds, minimal features)
./scripts/demo-e2e.sh --quick

# Full demo with agent simulation (60 seconds)
./scripts/demo-e2e.sh

# Keep files for debugging
./scripts/demo-e2e.sh --quick --no-cleanup
```

## What the Demo Does

The demo script demonstrates the complete repo.box workflow:

1. **🔑 Identity Generation** - Creates EVM identities for signing
2. **📁 Repository Setup** - Initializes a demo repo with configuration
3. **📝 Signed Commits** - Creates commits signed with EVM keys
4. **🚀 Push to Server** - Pushes to the git.repo.box server
5. **🤖 Agent Simulation** - (Full mode) Simulates multi-agent workflow
6. **🔍 Clone Verification** - Clones back and verifies signatures
7. **🌐 Explorer Links** - Provides links to view the repo online

## Demo Modes

### Quick Mode (`--quick`)
- Duration: ~30 seconds
- Single founder identity
- Basic signed commit workflow
- Push and clone verification
- **Use for**: Quick testing, CI/CD, first-time demos

### Full Mode (default)
- Duration: ~60 seconds  
- Founder + agent identities
- Feature branch simulation
- Documentation contributions
- Permission testing
- **Use for**: Complete demonstrations, development testing

## Repository Structure

The demo creates a repository with:

```
demo-hackathon-TIMESTAMP/
├── README.md                    # Project overview
├── package.json                 # Node.js package config
├── .gitignore                   # Git ignore patterns  
├── .repobox/
│   └── config.yml              # Permission configuration
├── src/
│   ├── demo-agent.js           # AI agent simulation
│   └── enhanced-features.js    # (Full mode) Agent enhancements
└── docs/
    └── AGENT_WORKFLOW.md       # (Full mode) Documentation
```

## Configuration File

The demo generates a `.repobox/config.yml` with:

```yaml
groups:
  founders:
    - evm:0x...                 # Generated founder address
  agents:
    - evm:0x...                 # (Full mode) Agent address
  bots:
    # (Empty in demo)

permissions:
  default: deny
  rules:
    - founders all              # Full access
    - agents push >feature/**   # Feature branches only
    - agents edit * >docs/**    # Documentation updates
    - agents not edit ./.repobox/config.yml  # Config protection
```

## Troubleshooting

### Clone Fails

**Symptom**: `git clone` fails with "repository not found"

**Common Causes**:
1. Server still processing the push (wait 5-10 seconds)
2. Wrong clone URL format
3. Network connectivity issues

**Solutions**:
```bash
# Check if server is running
curl -I https://repo.box

# Manual clone with full address path
git clone https://git.repo.box:3490/<address>/<repo>.git

# Check explorer to verify repo exists
open https://repo.box/explore/
```

### Push Fails

**Symptom**: `git push` fails with authentication or connection errors

**Common Causes**:
1. Git server not running on port 3490
2. Binary not built or not in PATH
3. EVM signature generation fails

**Solutions**:
```bash
# Check server status  
curl http://git.repo.box:3490/test-repo.git/info/refs?service=git-receive-pack

# Rebuild binary
cargo build --release --bin repobox-server

# Check repobox binary
./target/release/repobox whoami
```

### Config Template Issues

**Symptom**: All groups get the same address or empty addresses

**Root Cause**: String replacement in config template affects multiple groups

**Solution**: The script now uses targeted sed replacements:
```bash
# Fixed replacement targets specific sections
sed -i "/founders:/,/agents:/ s|# - evm:0x...|    - $FOUNDER_ADDRESS|"
```

### Permission Denials

**Symptom**: Git operations fail with "access denied" during agent simulation

**Expected Behavior**: This is intentional! The demo tests permission boundaries:
- Agents can't modify `.repobox/config.yml`  
- Agents can only push to `feature/**` branches
- Agents can only edit files in `docs/**`

### Server Connectivity

**Symptom**: "Git server may not be responding" warning

**Solutions**:
1. **Start the server**:
   ```bash
   cargo run --bin repobox-server --release
   ```

2. **Check port binding**:
   ```bash
   netstat -tlnp | grep 3490
   ```

3. **Verify git protocol**:
   ```bash
   curl -I http://git.repo.box:3490/test-repo.git/info/refs?service=git-receive-pack
   ```

## Demo Variations

### Testing Different Identities

```bash
# Generate additional identities
./target/release/repobox keys generate --alias test-agent-1
./target/release/repobox keys generate --alias test-agent-2

# Switch between identities  
./target/release/repobox use test-agent-1
./target/release/repobox whoami

# Run demo with different identity
./scripts/demo-e2e.sh --quick
```

### Custom Repositories

```bash
# Create custom demo repo
mkdir my-custom-demo && cd my-custom-demo

# Initialize with custom config
../target/release/repobox init
# Edit .repobox/config.yml as needed

# Manual workflow
../target/release/repobox keys generate --alias custom-founder
../target/release/repobox use custom-founder
git add . && git commit -m "Initial commit"
git remote add origin https://git.repo.box:3490/my-custom-repo.git
git push -u origin main
```

### Testing Permissions

```bash
# Run demo to create repo
./scripts/demo-e2e.sh --no-cleanup

# Try permission violations
cd /tmp/repobox-demo-*/demo-hackathon-*

# Switch to agent identity (if available)
../../target/release/repobox use demo-agent

# Try to modify config (should fail)
echo "# Modified" >> .repobox/config.yml
git add .repobox/config.yml
git commit -m "Try to modify config"  # Should be blocked

# Try to push to main (should fail) 
git checkout main
echo "# Main change" >> README.md
git add README.md
git commit -m "Direct main change"
git push origin main  # Should be blocked
```

## Cleanup

### Remove Demo Repositories

```bash
# Remove all demo repos
./scripts/demo-reset.sh --all

# Remove specific pattern
./scripts/demo-reset.sh --pattern 'demo-hackathon-*'

# Dry run (see what would be removed)
./scripts/demo-reset.sh --dry-run --all
```

### Reset Environment

```bash
# Clean temporary files
rm -rf /tmp/repobox-demo-*

# Reset git config (if needed)
git config --global --unset user.signingkey
git config --global --unset gpg.program
git config --global --unset commit.gpgsign
```

## Environment Variables

The demo respects these environment variables:

- `REPOBOX_DATA_DIR` - Data storage directory (default: `/tmp/repobox-data`)
- `REPOBOX_BINARY` - Path to repobox binary (default: `./target/release/repobox`)

```bash
# Custom data directory
export REPOBOX_DATA_DIR=/home/user/repobox-data
./scripts/demo-e2e.sh --quick

# Custom binary path
export REPOBOX_BINARY=/usr/local/bin/repobox  
./scripts/demo-e2e.sh --quick
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: repo.box Demo
on: [push]
jobs:
  demo:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build repo.box
        run: cargo build --release
      - name: Run demo
        run: ./scripts/demo-e2e.sh --quick
      - name: Cleanup
        run: ./scripts/demo-reset.sh --all
```

### Docker Integration

```dockerfile
FROM rust:1.75 as builder
COPY . /app
WORKDIR /app
RUN cargo build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y git curl
COPY --from=builder /app/target/release/repobox /usr/local/bin/
COPY --from=builder /app/scripts/ /app/scripts/
WORKDIR /app
CMD ["./scripts/demo-e2e.sh", "--quick"]
```

## Advanced Features

### Multi-Address Repositories

The demo shows single-address repositories, but repo.box supports:

- **Ownership transfer** - Move repos between addresses
- **Multi-signature governance** - Require multiple signers
- **Time-locked permissions** - Temporary access grants

### Custom Permission Rules

Beyond the demo config, you can create complex rules:

```yaml
permissions:
  rules:
    # Time-based access
    - contractors push >feature/** after:2024-01-01 before:2024-12-31
    
    # File-specific permissions  
    - security edit * >security/** 
    - security not edit >src/**
    
    # Branch-specific rules
    - agents push >feature/** not:main,release/**
    
    # Conditional access
    - bots push >bot/** if:automated-signer
```

### Explorer Integration

After running the demo, explore the web interface:

1. **Repository Browser**: `https://repo.box/explore/`
2. **Commit Verification**: Click commits to see EVM signatures  
3. **Permission Viewer**: View effective permissions for addresses
4. **Audit Trail**: Complete history of all repository access

## Getting Help

- **Issues**: Check the troubleshooting section above
- **Configuration**: See `.repobox/config.yml` documentation  
- **API Reference**: `./target/release/repobox --help`
- **Server Logs**: Check console output from `repobox-server`

## Contributing to the Demo

The demo script is designed to be:

- **Fast** - Quick mode completes in ~30 seconds
- **Reliable** - Handles network failures and retries
- **Educational** - Shows all key features clearly
- **Debuggable** - `--no-cleanup` preserves all files

When modifying:

1. Test both `--quick` and full modes
2. Verify cleanup works properly  
3. Check error handling for network failures
4. Update this documentation for any new features