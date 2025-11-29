#!/bin/bash
# Build optimized release binaries for multiple platforms

set -e

echo "Building genesis-preflight release binaries"
echo "==========================================="
echo ""

# Get version from Cargo.toml
VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
echo "Version: $VERSION"
echo ""

# Build release binary
echo "Building release binary..."
cargo build --release

# Strip binary (reduces size)
if command -v strip &> /dev/null; then
    echo "Stripping debug symbols..."
    strip target/release/genesis-preflight
fi

# Get binary size
BINARY_SIZE=$(wc -c < target/release/genesis-preflight)
BINARY_SIZE_MB=$((BINARY_SIZE / 1024 / 1024))
echo "Binary size: ${BINARY_SIZE_MB}MB"
echo ""

# Create release directory
RELEASE_DIR="target/release-dist"
mkdir -p "$RELEASE_DIR"

# Copy binary
cp target/release/genesis-preflight "$RELEASE_DIR/"

# Copy documentation
cp README.md "$RELEASE_DIR/"
cp LICENSE "$RELEASE_DIR/"
cp CHANGELOG.md "$RELEASE_DIR/"

# Create archive
echo "Creating release archive..."
cd "$RELEASE_DIR"
tar -czf "genesis-preflight-v${VERSION}-$(uname -s)-$(uname -m).tar.gz" \
    genesis-preflight \
    README.md \
    LICENSE \
    CHANGELOG.md

echo ""
echo "Release build complete!"
echo "Archive: target/release-dist/genesis-preflight-v${VERSION}-$(uname -s)-$(uname -m).tar.gz"
echo ""
echo "Installation test:"
echo "  tar -xzf genesis-preflight-v${VERSION}-$(uname -s)-$(uname -m).tar.gz"
echo "  ./genesis-preflight --version"
