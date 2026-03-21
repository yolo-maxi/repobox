#!/bin/sh
# repo.box installer — downloads pre-built binary and sets up the git shim.
# Usage: curl -sSf https://repo.box/install.sh | sh
set -e

# Configuration
REPO_BOX_VERSION="${REPO_BOX_VERSION:-latest}"
REPOBOX_INSTALL_DIR="${REPOBOX_INSTALL_DIR:-}"
REPOBOX_NO_MODIFY_PATH="${REPOBOX_NO_MODIFY_PATH:-}"
REPOBOX_SKIP_VERIFY="${REPOBOX_SKIP_VERIFY:-}"
BASE_URL="https://repo.box/releases/${REPO_BOX_VERSION}"
REPOBOX_HOME="$HOME/.repobox"

# Cleanup function
cleanup() {
  rm -f /tmp/repobox /tmp/repobox-checksums.sha256 2>/dev/null || true
}
trap cleanup EXIT INT TERM

# Platform detection
detect_platform() {
  local os arch
  
  os="$(uname -s)"
  arch="$(uname -m)"
  
  # Normalize OS
  case "$os" in
    Linux)  os="linux" ;;
    Darwin) os="darwin" ;;
    *)      
      echo "❌ Unsupported OS: $os"
      echo "   Supported: Linux, macOS"
      exit 1
      ;;
  esac
  
  # Normalize architecture
  case "$arch" in
    x86_64|amd64) arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *)
      echo "❌ Unsupported architecture: $arch"
      echo "   Supported: x86_64, aarch64"
      exit 1
      ;;
  esac
  
  # Check for Rosetta on Apple Silicon (suggest native binary)
  if [ "$os" = "darwin" ] && [ "$arch" = "x86_64" ]; then
    if [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || echo 0)" = "1" ]; then
      echo "⚠️  Detected x86_64 emulation (Rosetta) on Apple Silicon"
      echo "   Consider using: REPO_BOX_VERSION=${REPO_BOX_VERSION} curl -sSf https://repo.box/install.sh | sh"
      echo "   Or explicitly use ARM64: download repobox-darwin-aarch64"
      echo ""
    fi
  fi
  
  echo "${os}-${arch}"
}

# Determine install directory
determine_install_dir() {
  if [ -n "$REPOBOX_INSTALL_DIR" ]; then
    echo "$REPOBOX_INSTALL_DIR"
    return
  fi
  
  # Try /usr/local/bin first
  if [ -w "/usr/local/bin" ] 2>/dev/null; then
    echo "/usr/local/bin"
  elif sudo -n true 2>/dev/null; then
    echo "/usr/local/bin"
  else
    # Fall back to user-local directory
    echo "$HOME/.repobox/bin"
  fi
}

# Download with error handling
download_file() {
  local url="$1"
  local output="$2"
  local http_code
  
  if command -v curl >/dev/null 2>&1; then
    http_code=$(curl -sSL -w "%{http_code}" -o "$output" "$url" 2>/dev/null || echo "000")
    if [ "$http_code" != "200" ]; then
      echo "❌ Download failed (HTTP $http_code)"
      return 1
    fi
  elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$output" "$url" || {
      echo "❌ Download failed"
      return 1
    }
  else
    echo "❌ Neither curl nor wget found"
    echo "   Install curl: https://curl.se/download.html"
    exit 1
  fi
  
  return 0
}

# Verify checksum
verify_checksum() {
  local binary_file="$1"
  local binary_name="$2"
  local checksum_url="${BASE_URL}/checksums.sha256"
  
  if [ "$REPOBOX_SKIP_VERIFY" = "1" ]; then
    echo "⚠️  Skipping checksum verification (REPOBOX_SKIP_VERIFY=1)"
    return 0
  fi
  
  echo "🔐 Downloading checksums..."
  if ! download_file "$checksum_url" /tmp/repobox-checksums.sha256; then
    echo "⚠️  Checksum file not available — skipping verification"
    echo "   (Install continues, but binary is unverified)"
    return 0
  fi
  
  # Extract expected checksum
  local expected
  expected=$(grep "$binary_name" /tmp/repobox-checksums.sha256 2>/dev/null | awk '{print $1}' || true)
  if [ -z "$expected" ]; then
    echo "⚠️  Binary not found in checksums file — skipping verification"
    return 0
  fi
  
  # Calculate actual checksum
  local actual
  if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$binary_file" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    actual=$(shasum -a 256 "$binary_file" | awk '{print $1}')
  else
    echo "⚠️  No sha256sum or shasum found — skipping verification"
    return 0
  fi
  
  # Compare checksums
  if [ "$expected" != "$actual" ]; then
    echo "❌ Checksum verification FAILED"
    echo "   Expected: $expected"
    echo "   Got:      $actual"
    echo "   The binary may be corrupted or tampered with."
    exit 2
  fi
  
  echo "✅ Checksum verified"
}

# Install binary
install_binary() {
  local src="$1"
  local dest_dir="$2"
  local dest_file="$dest_dir/repobox"
  
  echo "📁 Installing to $dest_file..."
  
  # Create destination directory
  if [ "$dest_dir" = "/usr/local/bin" ]; then
    sudo mkdir -p "$dest_dir"
    sudo mv "$src" "$dest_file"
    sudo chmod 755 "$dest_file"
  else
    mkdir -p "$dest_dir"
    mv "$src" "$dest_file"
    chmod 755 "$dest_file"
  fi
  
  # Create symlinks for git integration
  echo "🔗 Creating git-repobox and git-rb symlinks..."
  if [ "$dest_dir" = "/usr/local/bin" ]; then
    sudo ln -sf "$dest_file" "$dest_dir/git-repobox"
    sudo ln -sf "$dest_file" "$dest_dir/git-rb"
  else
    ln -sf "$dest_file" "$dest_dir/git-repobox"
    ln -sf "$dest_file" "$dest_dir/git-rb"
  fi
}

# Setup git shim
setup_git_shim() {
  local install_dir="$1"
  
  echo "🔧 Setting up git shim..."
  mkdir -p "$REPOBOX_HOME/bin"
  mkdir -p "$REPOBOX_HOME/keys"
  
  # Find and store path to real git
  local real_git
  real_git="$(command -v git 2>/dev/null || echo /usr/bin/git)"
  
  # Don't overwrite if it's already our shim
  case "$real_git" in
    */.repobox/bin/git) real_git="/usr/bin/git" ;;
  esac
  
  if [ ! -x "$real_git" ]; then
    echo "⚠️  git not found. repo.box requires git to function."
    echo "   Install git: https://git-scm.com/downloads"
    echo "   repobox binary installed, but git shim won't work until git is available."
    echo "$real_git" > "$REPOBOX_HOME/real-git"
    return 0
  fi
  
  echo "$real_git" > "$REPOBOX_HOME/real-git"
  
  # Create shim symlink
  ln -sf "$install_dir/repobox" "$REPOBOX_HOME/bin/git"
  
  echo "   Real git: $real_git"
}

# Setup shell PATH
setup_shell_path() {
  if [ "$REPOBOX_NO_MODIFY_PATH" = "1" ]; then
    echo "⏭️  Skipping PATH modification (REPOBOX_NO_MODIFY_PATH=1)"
    return 0
  fi
  
  echo "🐚 Configuring shell PATH..."
  
  # Detect shell
  local shell_name rc_file path_line
  shell_name="$(basename "${SHELL:-/bin/sh}")"
  
  case "$shell_name" in
    zsh)  rc_file="$HOME/.zshrc" ;;
    fish) rc_file="$HOME/.config/fish/config.fish" ;;
    *)    rc_file="$HOME/.bashrc" ;;
  esac
  
  # Configure PATH line based on shell
  if [ "$shell_name" = "fish" ]; then
    path_line="fish_add_path --prepend \$HOME/.repobox/bin"
  else
    path_line="export PATH=\"\$HOME/.repobox/bin:\$PATH\""
  fi
  
  # Add to shell RC if not already present
  if [ -f "$rc_file" ] && grep -qF ".repobox/bin" "$rc_file" 2>/dev/null; then
    echo "   PATH already configured in $rc_file"
  else
    mkdir -p "$(dirname "$rc_file")"
    printf '\n# repo.box git shim\n%s\n' "$path_line" >> "$rc_file"
    echo "   Added PATH to $rc_file"
  fi
}

# Check if upgrading
check_existing_version() {
  local install_dir="$1"
  local existing_version
  
  if [ -x "$install_dir/repobox" ]; then
    existing_version=$("$install_dir/repobox" --version 2>/dev/null | awk '{print $2}' || echo "unknown")
    echo "🔄 Upgrading from $existing_version to $REPO_BOX_VERSION"
    return 0
  else
    echo "📦 Fresh installation"
    return 1
  fi
}

# Main installation
main() {
  echo "📦 repo.box installer"
  echo "   Version: $REPO_BOX_VERSION"
  
  # Detect platform
  local platform
  platform=$(detect_platform)
  echo "   Platform: $platform"
  
  # Determine install directory
  local install_dir
  install_dir=$(determine_install_dir)
  echo "   Install dir: $install_dir"
  
  # Check for existing installation
  check_existing_version "$install_dir" || true
  
  echo ""
  
  # Download binary
  local binary_name="repobox-${platform}"
  local download_url="${BASE_URL}/${binary_name}"
  
  echo "⬇️  Downloading $download_url..."
  if ! download_file "$download_url" /tmp/repobox; then
    echo "   Pre-built binary not available for $platform (version $REPO_BOX_VERSION)"
    echo ""
    echo "   Try:"
    echo "   1. Check available versions: https://repo.box/releases/"
    echo "   2. Build from source: cargo install --git https://github.com/yolo-maxi/repobox repobox-cli"
    echo "   3. Report issue: https://github.com/yolo-maxi/repobox/issues"
    exit 1
  fi
  
  chmod +x /tmp/repobox
  
  # Verify checksum
  verify_checksum /tmp/repobox "$binary_name"
  
  # Install binary and create symlinks
  install_binary /tmp/repobox "$install_dir"
  
  # Setup git shim
  setup_git_shim "$install_dir"
  
  # Configure shell PATH
  setup_shell_path
  
  echo ""
  echo "✅ repo.box installed successfully!"
  echo ""
  echo "   Binary:  $install_dir/repobox"
  echo "   Shim:    $REPOBOX_HOME/bin/git"
  
  if [ "$REPOBOX_NO_MODIFY_PATH" != "1" ]; then
    local shell_name
    shell_name="$(basename "${SHELL:-/bin/sh}")"
    
    if [ "$shell_name" = "fish" ]; then
      echo ""
      echo "   Restart your shell, then:"
      echo "     git repobox init"
    else
      echo ""
      echo "   Restart your shell or run:"
      echo "     export PATH=\"\$HOME/.repobox/bin:\$PATH\""
      echo ""
      echo "   Then initialize a repo:"
      echo "     git repobox init"
    fi
  fi
  
  echo ""
  echo "   Docs: https://repo.box"
}

# Handle script arguments
case "${1:-}" in
  --version)
    echo "repo.box installer (latest)"
    exit 0
    ;;
  --help)
    echo "repo.box installer"
    echo ""
    echo "Usage: curl -sSf https://repo.box/install.sh | sh"
    echo ""
    echo "Environment variables:"
    echo "  REPO_BOX_VERSION      Pin to specific version (default: latest)"
    echo "  REPOBOX_INSTALL_DIR   Override install directory"
    echo "  REPOBOX_NO_MODIFY_PATH Skip shell RC modifications"
    echo "  REPOBOX_SKIP_VERIFY   Skip checksum verification"
    echo ""
    echo "Examples:"
    echo "  REPO_BOX_VERSION=v0.2.0 curl -sSf https://repo.box/install.sh | sh"
    echo "  REPOBOX_INSTALL_DIR=~/bin curl -sSf https://repo.box/install.sh | sh"
    echo ""
    exit 0
    ;;
esac

# Run main installation
main