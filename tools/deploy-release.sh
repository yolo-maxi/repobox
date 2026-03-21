#!/bin/bash
# repo.box deployment script — copies built release to web root
set -euo pipefail

VERSION="${1:?Usage: deploy-release.sh v0.2.0}"
SRC="target/release-${VERSION}"
WEB_ROOT="/var/www/repo.box/subdomains/root"
DEST="${WEB_ROOT}/releases/${VERSION}"

echo "🚀 Deploying repo.box ${VERSION}..."

# Validate source directory exists
if [ ! -d "$SRC" ]; then
  echo "❌ Build directory not found: $SRC"
  echo "   Run tools/release.sh first to build the release"
  exit 1
fi

# Validate required files exist
REQUIRED_FILES=(
  "$SRC/repobox-linux-x86_64"
  "$SRC/repobox-linux-aarch64" 
  "$SRC/repobox-darwin-x86_64"
  "$SRC/repobox-darwin-aarch64"
  "$SRC/checksums.sha256"
)

for file in "${REQUIRED_FILES[@]}"; do
  if [ ! -f "$file" ]; then
    echo "❌ Missing required file: $file"
    exit 1
  fi
done

echo "📁 Creating release directory: $DEST"
sudo mkdir -p "$DEST"

echo "📦 Copying binaries and checksums..."
sudo cp "$SRC"/repobox-* "$DEST/"
sudo cp "$SRC/checksums.sha256" "$DEST/"

# Set proper permissions
sudo chmod 755 "$DEST"/repobox-*
sudo chmod 644 "$DEST/checksums.sha256"

# Update 'latest' symlink
echo "🔗 Updating latest symlink..."
sudo ln -sfn "$VERSION" "${WEB_ROOT}/releases/latest"

# Copy updated install.sh to web root
echo "📄 Updating install.sh in web root..."
sudo cp install.sh "${WEB_ROOT}/install.sh"
sudo chmod 644 "${WEB_ROOT}/install.sh"

echo ""
echo "✅ Deployed $VERSION successfully!"
echo ""
echo "📦 Release contents:"
ls -la "$DEST/"
echo ""
echo "🔗 Latest symlink:"
ls -la "${WEB_ROOT}/releases/latest"
echo ""
echo "🌐 Available at:"
echo "   https://repo.box/releases/${VERSION}/"
echo "   https://repo.box/releases/latest/ (symlink)"
echo "   https://repo.box/install.sh (updated)"