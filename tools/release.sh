#!/bin/bash
# repo.box release script — builds all platform binaries + checksums
set -euo pipefail

VERSION="${VERSION:?VERSION required (e.g. v0.2.0)}"
RELEASE_DIR="target/release-${VERSION}"

echo "🔨 Building repo.box release: ${VERSION}"
echo "   Output directory: ${RELEASE_DIR}"

# Clean and create release directory
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Add cargo bin to PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Target configurations: target-triple:binary-name
TARGETS=(
  "x86_64-unknown-linux-gnu:repobox-linux-x86_64"
  "aarch64-unknown-linux-gnu:repobox-linux-aarch64"
  "x86_64-apple-darwin:repobox-darwin-x86_64"
  "aarch64-apple-darwin:repobox-darwin-aarch64"
)

# Build for each target
for entry in "${TARGETS[@]}"; do
  target="${entry%%:*}"
  name="${entry##*:}"
  
  echo "🏗️  Building $name ($target)..."
  
  # Use cargo-zigbuild for cross-compilation
  cargo zigbuild --release --target "$target" --package repobox-cli
  
  # Copy binary to release dir
  cp "target/$target/release/repobox" "$RELEASE_DIR/$name"
  
  # Strip binary to reduce size (ignore errors for non-compatible targets)
  strip "$RELEASE_DIR/$name" 2>/dev/null || true
  
  echo "   ✅ Built $name ($(du -h "$RELEASE_DIR/$name" | cut -f1))"
done

# Generate checksums
echo "🔐 Generating checksums..."
cd "$RELEASE_DIR"
sha256sum repobox-* > checksums.sha256

echo ""
echo "✅ Release $VERSION built successfully!"
echo ""
echo "📦 Contents:"
ls -lh repobox-* checksums.sha256
echo ""
echo "🔐 Checksums:"
cat checksums.sha256
echo ""

if [ "${1:-}" = "--deploy" ]; then
  echo "🚀 Deploying to web root..."
  ../tools/deploy-release.sh "$VERSION"
fi