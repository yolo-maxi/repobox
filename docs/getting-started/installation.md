# Installation

Get repo.box running in 5 minutes. This guide covers installation, basic setup, and your first permission rule.

## Quick Install

**One-line install:** Download and install the latest release automatically

```bash
curl -sSf https://repo.box/install.sh | sh
```

This script:
- Detects your platform (Linux, macOS, Windows)  
- Downloads the appropriate binary from GitHub Releases
- Installs to `/usr/local/bin/repobox` (or `~/.local/bin/` if no sudo)
- Sets up the git shim configuration

**Manual verification:** If you prefer to inspect first:
```bash
curl -sSf https://repo.box/install.sh > install.sh
chmod +x install.sh
./install.sh
```

## Alternative Installation

### From GitHub Releases

Download the binary directly for your platform:

```bash
# Linux (x86_64)
wget https://github.com/yolo-maxi/repobox/releases/latest/download/repobox-linux-x86_64.tar.gz
tar xzf repobox-linux-x86_64.tar.gz
sudo mv repobox /usr/local/bin/

# macOS (ARM64)
wget https://github.com/yolo-maxi/repobox/releases/latest/download/repobox-darwin-aarch64.tar.gz
tar xzf repobox-darwin-aarch64.tar.gz
sudo mv repobox /usr/local/bin/

# macOS (Intel)
wget https://github.com/yolo-maxi/repobox/releases/latest/download/repobox-darwin-x86_64.tar.gz
tar xzf repobox-darwin-x86_64.tar.gz
sudo mv repobox /usr/local/bin/
```

### From Source (Requires Rust)

```bash
git clone https://github.com/yolo-maxi/repobox
cd repobox
cargo build --release
sudo cp target/release/repobox /usr/local/bin/
```

## Verify Installation

Check that repo.box is properly installed:

```bash
repobox --help
```

You should see the help output with all available commands.

```bash
git repobox --help
```

This should also work — the git integration is active.

## First Setup

### 1. Initialize repo.box in a Repository

Navigate to any git repository and initialize repo.box:

```bash
cd your-project
git repobox init
```

This creates `.repobox/config.yml` with a basic template:

```yaml
groups:
  owners:
    - evm:0x0000000000000000000000000000000000000000  # Replace with your address

permissions:
  default: deny
  rules:
    - owners own >*  # Full control for owners
```

### 2. Generate Your Identity

Generate an EVM keypair for yourself:

```bash
git repobox keys generate --alias alice
```

This creates:
- A new EVM keypair stored locally in `~/.repobox/keys/`
- An alias `alice` pointing to your address
- Sets this as your active identity

Check your identity:
```bash
git repobox whoami
# → alice (evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661)
```

### 3. Update Configuration

Edit `.repobox/config.yml` to use your real address:

```yaml
groups:
  owners:
    - evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661  # Your actual address from whoami

permissions:
  default: deny
  rules:
    - owners own >*  # Full control for owners
```

### 4. Test Your Setup

Test that you have permissions:

```bash
git repobox check alice own main
# → ✓ ALLOW: alice can own main
```

Commit the configuration:
```bash
git add .repobox/
git commit -m "configure repo.box permissions"
```

## ENS Setup (Optional)

If you want to use ENS names instead of raw addresses, set up Alchemy API access:

```bash
export ALCHEMY_API_KEY="your-key-here"
```

Add to your shell profile (`~/.bashrc`, `~/.zshrc`) to persist:
```bash
echo 'export ALCHEMY_API_KEY="your-key-here"' >> ~/.bashrc
```

Now you can use ENS names in configurations:
```yaml
groups:
  owners:
    - vitalik.eth  # Resolves automatically
    - alice.eth
```

## Next Steps

**For human users:** Continue to [First Repository](first-repo.md) for a complete setup walkthrough.

**For AI agents:** Jump to [Agent Onboarding](agent-onboarding.md) for agent-specific setup.

**Having issues?** Check [Troubleshooting](../user-guide/troubleshooting.md).

## System Requirements

- **Linux:** x86_64, glibc 2.17+ (Ubuntu 14.04+, RHEL 7+)
- **macOS:** 10.12+ (Sierra), Intel or ARM64
- **Windows:** Windows 10+ (Windows Subsystem for Linux recommended)
- **Git:** 2.0+ (for shim functionality)

## Platform-Specific Notes

### Linux
- Uses system Git installation
- Requires glibc 2.17+ for the binary
- Works with all major distributions

### macOS  
- Universal binary supports both Intel and ARM64
- Uses Homebrew Git if available, falls back to system Git
- No additional dependencies

### Windows
- Recommended: Use Windows Subsystem for Linux (WSL)
- Native Windows support available but less tested
- Git for Windows integration supported

## Security Considerations

**Local keys:** EVM keypairs are stored in `~/.repobox/keys/` with 600 permissions. These are not encrypted — treat them like SSH private keys.

**Git shim:** repo.box intercepts git commands transparently. It only reads/writes the local repository and `~/.repobox/` — no network access except for ENS resolution.

**ENS resolution:** Requires ALCHEMY_API_KEY for ENS name resolution. API calls only resolve names to addresses — no other data is transmitted.

**No server dependency:** Core functionality works entirely locally. Only ENS names and the optional repository server require internet access.