#!/bin/sh
# repo.box installer — curl -sSf https://repo.box/install.sh | sh
set -e

REPO="repobox/repobox"
BIN_NAME="repobox"
INSTALL_DIR="$HOME/.repobox/bin"
REAL_GIT_FILE="$HOME/.repobox/real-git"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "error: unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
  linux) TARGET="${ARCH}-unknown-linux-gnu" ;;
  darwin) TARGET="${ARCH}-apple-darwin" ;;
  *) echo "error: unsupported OS: $OS"; exit 1 ;;
esac

# Get latest release URL
ASSET="repobox-${TARGET}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

echo "📦 Installing repo.box..."
echo "   Platform: ${OS}/${ARCH}"

# Create dirs
mkdir -p "$INSTALL_DIR"
mkdir -p "$HOME/.repobox/keys"

# Download binary
echo "   Downloading ${ASSET}..."
if command -v curl >/dev/null 2>&1; then
  curl -sSfL "$DOWNLOAD_URL" -o "$INSTALL_DIR/$BIN_NAME"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$INSTALL_DIR/$BIN_NAME" "$DOWNLOAD_URL"
else
  echo "error: curl or wget required"
  exit 1
fi
chmod +x "$INSTALL_DIR/$BIN_NAME"

# Find real git and store it
REAL_GIT=$(command -v git 2>/dev/null || true)
if [ -z "$REAL_GIT" ]; then
  echo "warning: git not found — install git first, then re-run"
else
  echo "$REAL_GIT" > "$REAL_GIT_FILE"
fi

# Create git shim (symlink repobox as "git")
ln -sf "$INSTALL_DIR/$BIN_NAME" "$INSTALL_DIR/git"
# Also create git-repobox and git-rb for subcommand usage
ln -sf "$INSTALL_DIR/$BIN_NAME" "$INSTALL_DIR/git-repobox"
ln -sf "$INSTALL_DIR/$BIN_NAME" "$INSTALL_DIR/git-rb"

# Add to PATH in shell profile
PATH_LINE='export PATH="$HOME/.repobox/bin:$PATH"'
SHELL_NAME=$(basename "${SHELL:-/bin/sh}")

case "$SHELL_NAME" in
  zsh)  PROFILE="$HOME/.zshrc" ;;
  fish) PROFILE="" ;;  # fish needs different syntax
  *)    PROFILE="$HOME/.bashrc" ;;
esac

if [ -n "$PROFILE" ]; then
  if [ -f "$PROFILE" ] && grep -q '.repobox/bin' "$PROFILE" 2>/dev/null; then
    : # already there
  else
    printf '\n# repo.box\n%s\n' "$PATH_LINE" >> "$PROFILE"
  fi
fi

echo ""
echo "✅ repo.box installed!"
echo ""
echo "   Binary:  $INSTALL_DIR/$BIN_NAME"
echo "   Shim:    $INSTALL_DIR/git"
if [ -n "$REAL_GIT" ]; then
  echo "   Real git: $REAL_GIT"
fi
echo ""

if [ "$SHELL_NAME" = "fish" ]; then
  echo "   Add to ~/.config/fish/config.fish:"
  echo "     fish_add_path \$HOME/.repobox/bin"
  echo ""
  echo "   Then: git repobox init"
else
  echo "   Open a new terminal, then:"
  echo "     git repobox init"
fi
