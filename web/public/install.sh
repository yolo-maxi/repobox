#!/bin/sh
set -eu

REPO="yolo-maxi/repobox"
VERSION="${REPOBOX_VERSION:-latest}"
INSTALL_DIR="${REPOBOX_INSTALL_DIR:-/usr/local/bin}"
BIN_NAME="repobox"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: required command not found: $1" >&2
    exit 1
  }
}

need_cmd uname
need_cmd curl
need_cmd chmod
need_cmd mktemp

OS_RAW=$(uname -s)
ARCH_RAW=$(uname -m)

case "$OS_RAW" in
  Linux) OS="linux" ;;
  Darwin) OS="darwin" ;;
  *)
    echo "error: unsupported OS: $OS_RAW" >&2
    exit 1
    ;;
esac

case "$ARCH_RAW" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *)
    echo "error: unsupported architecture: $ARCH_RAW" >&2
    exit 1
    ;;
esac

ASSET="${BIN_NAME}-${OS}-${ARCH}"

if [ "$VERSION" = "latest" ]; then
  TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)
  if [ -z "$TAG" ]; then
    echo "error: could not resolve latest release tag" >&2
    exit 1
  fi
else
  TAG="$VERSION"
fi

URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"
TMP="$(mktemp)"

echo "Installing ${BIN_NAME} ${TAG} (${OS}/${ARCH})..."
if ! curl -fL "$URL" -o "$TMP"; then
  echo "error: failed to download ${ASSET} from release ${TAG}" >&2
  if [ "$OS" = "darwin" ] && [ "$ARCH" = "x86_64" ]; then
    echo "hint: Intel macOS binary may not be published yet for this release." >&2
  fi
  rm -f "$TMP"
  exit 1
fi

chmod +x "$TMP"

if [ -w "$INSTALL_DIR" ] || [ ! -d "$INSTALL_DIR" ] 2>/dev/null; then
  mkdir -p "$INSTALL_DIR"
  mv "$TMP" "$INSTALL_DIR/$BIN_NAME"
  echo "Installed to $INSTALL_DIR/$BIN_NAME"
else
  if command -v sudo >/dev/null 2>&1; then
    sudo mkdir -p "$INSTALL_DIR"
    sudo mv "$TMP" "$INSTALL_DIR/$BIN_NAME"
    echo "Installed to $INSTALL_DIR/$BIN_NAME"
  else
    FALLBACK_DIR="$HOME/.local/bin"
    mkdir -p "$FALLBACK_DIR"
    mv "$TMP" "$FALLBACK_DIR/$BIN_NAME"
    echo "Installed to $FALLBACK_DIR/$BIN_NAME"
    echo "Add this to your shell profile if needed:"
    echo "  export PATH=\"$FALLBACK_DIR:\$PATH\""
  fi
fi

echo "Run: repobox --help"
