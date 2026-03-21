# repo.box Spec: Install Script Improvements

## Overview

The install script (`curl -sSf https://repo.box/install.sh | sh`) is the primary distribution channel for the repobox CLI. It must reliably detect the user's platform, download the correct pre-built binary, verify its integrity, install it, and set up the git shim — across Linux and macOS, on both x86_64 and ARM64.

**Priority:** P1
**Tags:** cli, distribution
**EVM Identity:** PM Agent (`0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b`)

## Current State

Two versions of the install script exist:

- **Repo version** (`/home/xiko/repobox/install.sh`): Older skeleton pointing at GitHub releases with Rust-style target triples (`x86_64-unknown-linux-gnu`)
- **Deployed version** (`/var/www/repo.box/subdomains/root/install.sh`): Partially improved — downloads from `https://repo.box/releases/{version}/repobox-{os}-{arch}`, handles sudo fallback, does fish shell support

**Release infrastructure** already partially exists:
- Caddy routes `/releases/*` and `/install.sh` to static files in `/var/www/repo.box/subdomains/root/`
- Directory structure: `/releases/v0.1.0/` with a `latest` symlink
- Currently only `repobox-linux-x86_64` and `repobox-server-linux-x86_64` binaries exist
- No checksums, no signatures, no macOS binaries, no CI pipeline

## Goals

1. **One-liner install works on any developer machine** — Linux x86_64, Linux ARM64, macOS x86_64, macOS ARM64 (Apple Silicon)
2. **Integrity verification** — SHA256 checksums for every binary, with clear path to signing later
3. **Idempotent** — safe to run multiple times (upgrade in place)
4. **AI-agent friendly** — zero interactive prompts, clean exit codes, machine-parseable output
5. **Version pinning** — `REPO_BOX_VERSION=v0.1.0 curl ... | sh` to install specific versions
6. **Uninstall path** — `repobox uninstall` or documented manual steps

---

## 1. Platform Detection

### Supported Targets

| OS | Arch | Binary Name | Status |
|---|---|---|---|
| Linux | x86_64 | `repobox-linux-x86_64` | ✅ Exists |
| Linux | aarch64 | `repobox-linux-aarch64` | 🔨 Needs cross-compile |
| macOS | x86_64 | `repobox-darwin-x86_64` | 🔨 Needs cross-compile |
| macOS | aarch64 | `repobox-darwin-aarch64` | 🔨 Needs cross-compile |

### Detection Logic

```
uname -s → Linux | Darwin         (reject everything else)
uname -m → x86_64 | amd64         → normalize to x86_64
            aarch64 | arm64       → normalize to aarch64
            (everything else)     → error with helpful message
```

### Edge Cases
- **WSL2**: Reports `Linux` / `x86_64` — works with Linux binary. No special handling needed.
- **Rosetta on Apple Silicon**: Reports `x86_64` on ARM Mac. The x86_64 binary works via Rosetta, but we should detect this and suggest the native `aarch64` binary. Check: `sysctl -n sysctl.proc_translated 2>/dev/null` returns `1` under Rosetta.
- **musl vs glibc**: Linux binaries should be statically linked or use musl to avoid glibc version issues on older distros and Alpine. If dynamic linking to glibc, minimum target glibc 2.17 (CentOS 7 era).

---

## 2. Binary Hosting Strategy

### Self-hosted on repo.box (Primary)

Binaries served from the same domain as the install script. No GitHub dependency.

**URL pattern:**
```
https://repo.box/releases/{version}/repobox-{os}-{arch}
https://repo.box/releases/{version}/checksums.sha256
```

**Directory layout on disk:**
```
/var/www/repo.box/subdomains/root/releases/
├── latest -> v0.2.0              # symlink, updated on each release
├── v0.1.0/
│   ├── repobox-linux-x86_64
│   ├── repobox-linux-aarch64
│   ├── repobox-darwin-x86_64
│   ├── repobox-darwin-aarch64
│   ├── repobox-server-linux-x86_64
│   └── checksums.sha256
├── v0.2.0/
│   ├── ...same structure...
│   └── checksums.sha256
```

**`checksums.sha256` format** (BSD-style, compatible with `sha256sum -c` and `shasum -a 256 -c`):
```
e3b0c44298fc1c149afbf4c8996fb924...  repobox-linux-x86_64
a1b2c3d4e5f6789012345678abcdef01...  repobox-linux-aarch64
f0e1d2c3b4a596870123456789abcdef...  repobox-darwin-x86_64
0123456789abcdef0123456789abcdef...  repobox-darwin-aarch64
```

### GitHub Releases (Fallback — Future)

Not implemented for v1. When GitHub Actions CI is set up, releases can be published to both `https://github.com/yolo-maxi/repobox/releases` and repo.box simultaneously.

### Why Self-hosted First
- No dependency on GitHub availability
- Faster for users (single CDN hop vs GitHub redirect chain)
- Full control over URL structure
- Already working via Caddy static file serving
- GitHub releases can be added as a secondary mirror later

---

## 3. Build & Release Pipeline

### Cross-compilation Script: `tools/release.sh`

New script that builds all platform binaries and produces a versioned release directory.

**Inputs:**
- `VERSION` env var (e.g., `v0.2.0`) — required
- Optional: `--deploy` flag to copy to `/var/www/` and update the `latest` symlink

**Outputs:**
- `/home/xiko/repobox/target/release-{version}/` with all binaries + checksums

**Build targets:**

| Target Triple | Binary Name | Cross Tool |
|---|---|---|
| `x86_64-unknown-linux-gnu` | `repobox-linux-x86_64` | Native (on Hetzner) |
| `aarch64-unknown-linux-gnu` | `repobox-linux-aarch64` | `cross` or `cargo-zigbuild` |
| `x86_64-apple-darwin` | `repobox-darwin-x86_64` | `cargo-zigbuild` or `osxcross` |
| `aarch64-apple-darwin` | `repobox-darwin-aarch64` | `cargo-zigbuild` or `osxcross` |

**Recommended approach: `cargo-zigbuild`**
- Uses Zig as a cross-compilation linker
- No macOS hardware needed for macOS builds
- Handles all four targets from a single Linux machine
- Install: `cargo install cargo-zigbuild && pip3 install ziglang`

**Script pseudocode:**
```bash
#!/bin/bash
set -euo pipefail

VERSION="${VERSION:?VERSION required (e.g. v0.2.0)}"
RELEASE_DIR="target/release-${VERSION}"
mkdir -p "$RELEASE_DIR"

TARGETS=(
  "x86_64-unknown-linux-gnu:repobox-linux-x86_64"
  "aarch64-unknown-linux-gnu:repobox-linux-aarch64"
  "x86_64-apple-darwin:repobox-darwin-x86_64"
  "aarch64-apple-darwin:repobox-darwin-aarch64"
)

for entry in "${TARGETS[@]}"; do
  target="${entry%%:*}"
  name="${entry##*:}"
  
  echo "Building $name ($target)..."
  cargo zigbuild --release --target "$target" -p repobox-cli
  cp "target/$target/release/repobox" "$RELEASE_DIR/$name"
  strip "$RELEASE_DIR/$name" 2>/dev/null || true
done

# Generate checksums
cd "$RELEASE_DIR"
sha256sum repobox-* > checksums.sha256

echo "✅ Release $VERSION built:"
ls -lh repobox-* checksums.sha256
```

### Deploy Script: `tools/deploy-release.sh`

Separate script (or `--deploy` flag) that copies a built release to the web root:

```bash
#!/bin/bash
set -euo pipefail

VERSION="${1:?Usage: deploy-release.sh v0.2.0}"
SRC="target/release-${VERSION}"
DEST="/var/www/repo.box/subdomains/root/releases/${VERSION}"

[ -d "$SRC" ] || { echo "❌ Build dir not found: $SRC"; exit 1; }

sudo mkdir -p "$DEST"
sudo cp "$SRC"/repobox-* "$DEST/"
sudo cp "$SRC/checksums.sha256" "$DEST/"
sudo chmod 755 "$DEST"/repobox-*
sudo ln -sfn "$VERSION" /var/www/repo.box/subdomains/root/releases/latest

echo "✅ Deployed $VERSION"
ls -la "$DEST/"
```

### Syncing install.sh to web root

The canonical `install.sh` lives in the repo at `/home/xiko/repobox/install.sh`. After any edit, copy to the web root:

```bash
cp install.sh /var/www/repo.box/subdomains/root/install.sh
```

This should be part of the release process. The `deploy-release.sh` script should also copy the latest `install.sh`.

---

## 4. Checksum Verification

### In the Install Script

After downloading the binary, the script downloads the checksums file and verifies:

```sh
# Download checksums
CHECKSUM_URL="${BASE_URL}/checksums.sha256"
curl -sSfL "$CHECKSUM_URL" -o /tmp/repobox-checksums.sha256 || {
  echo "⚠️  Checksum file not available — skipping verification"
  echo "   (Install continues, but binary is unverified)"
}

# Verify if checksums were downloaded
if [ -f /tmp/repobox-checksums.sha256 ]; then
  EXPECTED=$(grep "$BINARY" /tmp/repobox-checksums.sha256 | awk '{print $1}')
  if [ -n "$EXPECTED" ]; then
    if command -v sha256sum >/dev/null 2>&1; then
      ACTUAL=$(sha256sum /tmp/repobox | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
      ACTUAL=$(shasum -a 256 /tmp/repobox | awk '{print $1}')
    else
      echo "⚠️  No sha256sum or shasum found — skipping verification"
      ACTUAL="$EXPECTED"  # skip check
    fi
    
    if [ "$EXPECTED" != "$ACTUAL" ]; then
      echo "❌ Checksum verification FAILED"
      echo "   Expected: $EXPECTED"
      echo "   Got:      $ACTUAL"
      echo "   The binary may be corrupted or tampered with."
      rm -f /tmp/repobox /tmp/repobox-checksums.sha256
      exit 1
    fi
    echo "✅ Checksum verified"
  fi
  rm -f /tmp/repobox-checksums.sha256
fi
```

### Graceful Degradation
- If checksums file doesn't exist (older releases): warn but continue
- If sha256sum/shasum not available: warn but continue
- If checksum mismatch: **hard fail** — never install a corrupted binary
- `--skip-verify` env var for CI environments that need to bypass: `REPOBOX_SKIP_VERIFY=1`

### Future: GPG/Minisign Signatures
Not in v1. When needed:
- Use `minisign` (simpler than GPG, single-purpose)
- Sign `checksums.sha256` file, not individual binaries
- Public key embedded in install script AND hosted at `https://repo.box/repobox.pub`

---

## 5. Installation Locations & PATH Setup

### Binary Location

**Primary:** `/usr/local/bin/repobox` (requires sudo on most systems)
**Fallback:** `$HOME/.repobox/bin/repobox` (no sudo needed)

Logic:
```sh
if [ -w /usr/local/bin ] || command -v sudo >/dev/null 2>&1; then
  INSTALL_DIR="/usr/local/bin"
else
  INSTALL_DIR="$HOME/.repobox/bin"
fi
```

Allow override: `REPOBOX_INSTALL_DIR=/custom/path curl ... | sh`

### Symlinks Created

| Symlink | Target | Purpose |
|---|---|---|
| `$INSTALL_DIR/git-repobox` | `$INSTALL_DIR/repobox` | Enables `git repobox` subcommand |
| `$INSTALL_DIR/git-rb` | `$INSTALL_DIR/repobox` | Short alias: `git rb init` |
| `$HOME/.repobox/bin/git` | `$INSTALL_DIR/repobox` | Git shim (intercepts git commands) |

### Directory Structure Created

```
$HOME/.repobox/
├── bin/
│   └── git -> /usr/local/bin/repobox    # git shim symlink
├── keys/                                  # EVM keypair storage
└── real-git                               # path to actual git binary
```

### PATH Setup

The git shim requires `$HOME/.repobox/bin` to be **before** the system git in `$PATH`.

**Shell detection and RC file:**

| Shell | RC File | PATH Line |
|---|---|---|
| bash | `~/.bashrc` | `export PATH="$HOME/.repobox/bin:$PATH"` |
| zsh | `~/.zshrc` | `export PATH="$HOME/.repobox/bin:$PATH"` |
| fish | `~/.config/fish/config.fish` | `fish_add_path --prepend $HOME/.repobox/bin` |

**Idempotency:** Check `grep -qF ".repobox/bin"` before appending. Never duplicate the PATH entry.

**Real git detection:**
- Find the real git *before* modifying PATH
- Skip if the found `git` resolves to our own shim (check for `.repobox/bin/git` in the path)
- Default fallback: `/usr/bin/git`
- Store in `$HOME/.repobox/real-git` for the shim to read

---

## 6. Error Handling & Fallbacks

### Exit Codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | General error (unsupported platform, download failure, etc.) |
| 2 | Checksum verification failure |
| 3 | Permission error (can't write to install dir, no sudo) |

### Error Scenarios

**Download fails (HTTP non-200):**
```
❌ Download failed (HTTP 404)
   Pre-built binary not available for darwin/aarch64 (version v0.1.0).
   
   Try:
   1. Check available versions: https://repo.box/releases/
   2. Build from source: cargo install --git https://github.com/yolo-maxi/repobox repobox-cli
   3. Report issue: https://github.com/yolo-maxi/repobox/issues
```

**No curl or wget:**
```
❌ Neither curl nor wget found.
   Install curl: https://curl.se/download.html
```

**No sudo and /usr/local/bin not writable:**
Install to `$HOME/.repobox/bin/repobox` instead. No error — just adjust the install location silently.

**Git not found:**
```
⚠️  git not found. repo.box requires git to function.
   Install git: https://git-scm.com/downloads
   
   repobox binary installed, but git shim won't work until git is available.
```
Continue installation — the binary is still useful for `repobox init` etc. Just skip the shim setup.

**Already installed (upgrade):**
- Detect existing installation
- Show current version vs. new version
- Replace binary in place
- Don't re-add PATH entries
- Print "upgraded from vX to vY" instead of "installed"

### Non-interactive Guarantee

The script must **never** prompt for input. Specifically:
- No `read` calls
- No `sudo -v` (interactive password prompt) — instead, test `sudo -n true 2>/dev/null` for non-interactive sudo. If it fails, fall back to user-local install
- All decisions made automatically based on environment

This is critical for the AI agent use case — agents pipe `curl | sh` and can't respond to prompts.

**For CI environments:** `REPOBOX_NO_MODIFY_PATH=1` skips all shell RC modifications.

---

## 7. Upgrade & Uninstall

### Upgrade

Running the installer again with a newer version (or `latest`):

1. Download new binary to `/tmp/repobox`
2. Verify checksum
3. Detect existing install: `command -v repobox` or check `$INSTALL_DIR/repobox`
4. If exists, get current version: `repobox --version` (must be implemented in CLI)
5. Replace binary (mv is atomic on same filesystem)
6. Update symlinks
7. Don't touch shell RC (already configured)

**Version check shortcut:** `repobox self-update` (future CLI command, not in v1 script)

### Uninstall

Add `repobox uninstall` subcommand to the CLI that:

1. Removes `$INSTALL_DIR/repobox`
2. Removes `$INSTALL_DIR/git-repobox` and `$INSTALL_DIR/git-rb`
3. Removes `$HOME/.repobox/bin/git` shim
4. Removes PATH lines from shell RCs (grep + sed)
5. Asks before removing `$HOME/.repobox/keys/` (contains user keys!)
6. Does NOT remove `$HOME/.repobox/` entirely if keys exist

Alternatively, document manual steps:
```bash
rm -f /usr/local/bin/repobox /usr/local/bin/git-repobox /usr/local/bin/git-rb
rm -rf ~/.repobox/bin
# Remove the PATH line from your shell RC (~/.bashrc, ~/.zshrc, etc.)
# Optionally: rm -rf ~/.repobox (WARNING: deletes your keys)
```

---

## 8. Full Install Script (Target State)

The canonical `install.sh` should be updated in the repo and synced to the web root. Key differences from current deployed version:

1. **Add checksum verification** (Section 4)
2. **Add Rosetta detection** for Apple Silicon
3. **Add non-interactive sudo check** (`sudo -n true`)
4. **Add user-local fallback** when no sudo
5. **Add upgrade detection** (show current vs. new version)
6. **Add proper exit codes** (Section 6)
7. **Add env var overrides** (`REPOBOX_INSTALL_DIR`, `REPOBOX_NO_MODIFY_PATH`, `REPOBOX_SKIP_VERIFY`)
8. **Fix fish shell support** (use `fish_add_path` not `set -gx`)
9. **Add `--version` / `--help` flags** to the script itself
10. **Clean up /tmp artifacts** on all exit paths (trap)

---

## 9. Testing Strategy

### Unit Tests (Script)

Use `shellcheck` for static analysis + `bats` (Bash Automated Testing System) for functional tests.

**Test file:** `tests/install_test.bats`

```bash
# Platform detection tests
@test "detects Linux x86_64" {
  OS=Linux ARCH=x86_64 run detect_platform
  [ "$output" = "linux-x86_64" ]
}

@test "detects macOS ARM64" {
  OS=Darwin ARCH=arm64 run detect_platform
  [ "$output" = "darwin-aarch64" ]
}

@test "rejects unsupported OS" {
  OS=FreeBSD ARCH=x86_64 run detect_platform
  [ "$status" -eq 1 ]
}

@test "normalizes amd64 to x86_64" {
  OS=Linux ARCH=amd64 run detect_platform
  [ "$output" = "linux-x86_64" ]
}
```

### Integration Tests

**Test matrix (manual, run before each release):**

| Environment | Method | Expected Result |
|---|---|---|
| Ubuntu 22.04 x86_64 (Hetzner) | Direct | ✅ Native install |
| Debian ARM64 (Docker) | `docker run --rm -it arm64v8/debian` | ✅ ARM install |
| macOS x86_64 | Fran's machine or CI Mac | ✅ Intel install |
| macOS ARM64 | Fran's machine or CI Mac | ✅ Apple Silicon install |
| Alpine Linux | `docker run --rm -it alpine` | ✅ If musl build; ❌ if glibc |
| Ubuntu 20.04 (old glibc) | Docker | ✅ Verify glibc compat |
| WSL2 | Manual test | ✅ Uses Linux binary |
| No sudo available | Docker (non-root) | ✅ Falls back to user-local |
| No git installed | Docker (minimal) | ⚠️ Warns, installs binary only |
| Already installed | Run twice | ✅ Upgrade path |
| Bad checksum | Tamper with downloaded file | ❌ Exits with code 2 |
| Network error | Block repo.box | ❌ Clean error message |
| fish shell | Docker with fish | ✅ Correct PATH setup |

**Docker-based test harness:** `tests/install-docker-test.sh`

```bash
#!/bin/bash
# Run install script in various Docker containers

test_in_docker() {
  local image="$1"
  local label="$2"
  echo "Testing: $label ($image)"
  docker run --rm "$image" sh -c \
    'apt-get update -qq && apt-get install -yqq curl git > /dev/null 2>&1; 
     curl -sSf https://repo.box/install.sh | sh && 
     repobox --version'
  echo "  Result: $?"
}

test_in_docker "ubuntu:22.04" "Ubuntu 22.04 x86_64"
test_in_docker "ubuntu:20.04" "Ubuntu 20.04 (old glibc)"
test_in_docker "debian:bullseye" "Debian Bullseye"
test_in_docker "alpine:3.19" "Alpine (musl)"
# ARM tests need --platform flag or ARM host
```

### CI Smoke Test (Future)

When GitHub Actions is set up, add a workflow that:
1. Builds all targets
2. Runs install script in a matrix of Docker containers
3. Verifies `repobox --version` outputs the expected version
4. Runs `repobox init` in a test repo
5. Verifies the git shim intercepts commands correctly

---

## 10. File-by-File Changes

### New Files

**`tools/release.sh`** — Cross-compilation build script
- Builds all 4 platform binaries using `cargo-zigbuild`
- Generates `checksums.sha256`
- Outputs to `target/release-{version}/`

**`tools/deploy-release.sh`** — Deploy built release to web root
- Copies binaries + checksums to `/var/www/repo.box/subdomains/root/releases/{version}/`
- Updates `latest` symlink
- Copies `install.sh` to web root

**`tests/install_test.bats`** — Install script tests (bats framework)
- Platform detection tests
- Checksum verification tests
- Idempotency tests

**`tests/install-docker-test.sh`** — Docker-based integration tests
- Runs install in Ubuntu, Debian, Alpine containers
- Verifies binary works after install

### Modified Files

**`install.sh`** (root of repo) — Complete rewrite based on deployed version + improvements
- Add checksum verification
- Add non-interactive sudo detection
- Add user-local fallback
- Add upgrade detection
- Add Rosetta detection
- Add env var overrides
- Add cleanup trap
- Add proper exit codes
- Fix fish shell PATH syntax

**`repobox-cli/src/main.rs`** — Add `--version` flag
- Clap already supports this via `#[command(version)]`
- Ensure `Cargo.toml` version is propagated

**`repobox-cli/Cargo.toml`** — Ensure version field is set correctly

**`KANBAN.md`** — Move task from "In Progress" to "Done" after implementation

---

## 11. Acceptance Criteria

- [ ] `curl -sSf https://repo.box/install.sh | sh` successfully installs on Linux x86_64
- [ ] `curl -sSf https://repo.box/install.sh | sh` successfully installs on Linux aarch64
- [ ] `curl -sSf https://repo.box/install.sh | sh` successfully installs on macOS x86_64
- [ ] `curl -sSf https://repo.box/install.sh | sh` successfully installs on macOS aarch64
- [ ] `repobox --version` outputs the correct version after install
- [ ] `git repobox init` works after install (git subcommand registered)
- [ ] `git rb init` works (short alias)
- [ ] Checksum verification catches a tampered binary (exits with code 2)
- [ ] Running the installer twice upgrades cleanly without duplicating PATH entries
- [ ] Works without sudo (falls back to `~/.repobox/bin/`)
- [ ] Works without git installed (warns, installs binary only)
- [ ] `REPO_BOX_VERSION=v0.1.0 curl ... | sh` installs the pinned version
- [ ] `shellcheck install.sh` passes with no errors
- [ ] All Docker integration tests pass
- [ ] `tools/release.sh` builds all 4 platform binaries
- [ ] `checksums.sha256` is generated and served alongside binaries

---

## 12. Dependencies & Prerequisites

Before implementation:

1. **Install `cargo-zigbuild`**: `cargo install cargo-zigbuild`
2. **Install Zig**: `pip3 install ziglang` (or system package)
3. **Add Rust targets**: 
   ```bash
   rustup target add x86_64-unknown-linux-gnu
   rustup target add aarch64-unknown-linux-gnu
   rustup target add x86_64-apple-darwin
   rustup target add aarch64-apple-darwin
   ```
4. **Install `shellcheck`**: `apt install shellcheck`
5. **Install `bats`**: `apt install bats` or `git clone https://github.com/bats-core/bats-core`

## 13. Implementation Order

1. **Phase 1 — Build pipeline** (`tools/release.sh`, `tools/deploy-release.sh`)
   - Get cross-compilation working for all 4 targets
   - Generate checksums
   - Deploy v0.2.0 with all binaries

2. **Phase 2 — Install script rewrite** (`install.sh`)
   - Checksum verification
   - Sudo fallback
   - Upgrade detection
   - Env var overrides
   - Deploy to web root

3. **Phase 3 — Testing** (`tests/`)
   - shellcheck pass
   - Docker integration tests
   - Manual macOS test (need Fran or CI)

4. **Phase 4 — CLI support** (`repobox-cli`)
   - `repobox --version` (may already work via clap)
   - `repobox uninstall` subcommand

---

## 14. Open Questions

1. **Static linking on Linux?** — `musl` target (`x86_64-unknown-linux-musl`) would give a single binary that works on Alpine + any glibc version. Tradeoff: slightly larger binary, possible issues with DNS resolution (musl's `getaddrinfo` is simpler). **Recommendation:** Ship musl builds as the Linux binaries.

2. **macOS code signing?** — Unsigned macOS binaries trigger Gatekeeper. Users need to `xattr -d com.apple.quarantine repobox` or right-click → Open. For CLI tools installed via `curl | sh` this is standard (Rust, Deno, etc. all do this). Not a blocker for v1.

3. **ARM Linux priority?** — How many users are on ARM Linux (Raspberry Pi, Graviton, etc.)? If low priority, can ship x86_64-only initially and add ARM later. **Recommendation:** Ship all 4 targets from the start — `cargo-zigbuild` makes it trivial.

4. **GitHub Releases mirror?** — Should we also publish to GitHub Releases? Adds discoverability but requires CI setup. **Recommendation:** After CI is established, not for v1.
