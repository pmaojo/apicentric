#!/bin/bash
# Script to update Homebrew formula with SHA256 checksums from a release

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.1"
    exit 1
fi

VERSION="$1"
REPO="pmaojo/apicentric"
BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"

echo "🔍 Fetching checksums for version ${VERSION}..."

# Create temporary directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Download artifacts
echo "📦 Downloading macOS ARM64..."
curl -fsSL "${BASE_URL}/apicentric-macos-arm64.tar.gz" -o apicentric-macos-arm64.tar.gz

echo "📦 Downloading macOS x64..."
curl -fsSL "${BASE_URL}/apicentric-macos-x64.tar.gz" -o apicentric-macos-x64.tar.gz

echo "📦 Downloading Linux x64..."
curl -fsSL "${BASE_URL}/apicentric-linux-x64.tar.gz" -o apicentric-linux-x64.tar.gz

# Calculate checksums
echo ""
echo "🔐 Calculating SHA256 checksums..."
echo ""

if command -v shasum &> /dev/null; then
    SHA_MACOS_ARM64=$(shasum -a 256 apicentric-macos-arm64.tar.gz | awk '{print $1}')
    SHA_MACOS_X64=$(shasum -a 256 apicentric-macos-x64.tar.gz | awk '{print $1}')
    SHA_LINUX_X64=$(shasum -a 256 apicentric-linux-x64.tar.gz | awk '{print $1}')
elif command -v sha256sum &> /dev/null; then
    SHA_MACOS_ARM64=$(sha256sum apicentric-macos-arm64.tar.gz | awk '{print $1}')
    SHA_MACOS_X64=$(sha256sum apicentric-macos-x64.tar.gz | awk '{print $1}')
    SHA_LINUX_X64=$(sha256sum apicentric-linux-x64.tar.gz | awk '{print $1}')
else
    echo "❌ Error: Neither shasum nor sha256sum found"
    exit 1
fi

echo "macOS ARM64: ${SHA_MACOS_ARM64}"
echo "macOS x64:   ${SHA_MACOS_X64}"
echo "Linux x64:   ${SHA_LINUX_X64}"
echo ""

# Clean up
cd - > /dev/null
rm -rf "$TMP_DIR"

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FORMULA_FILE="${SCRIPT_DIR}/apicentric.rb"

if [ ! -f "$FORMULA_FILE" ]; then
    echo "❌ Error: Formula file not found at ${FORMULA_FILE}"
    exit 1
fi

echo "📝 Updating formula..."

# Update version
sed -i.bak "s/version \".*\"/version \"${VERSION}\"/" "$FORMULA_FILE"

# Update checksums
sed -i.bak "s/sha256 \"PLACEHOLDER_SHA256_MACOS_ARM64\"/sha256 \"${SHA_MACOS_ARM64}\"/" "$FORMULA_FILE"
sed -i.bak "s/sha256 \"PLACEHOLDER_SHA256_MACOS_X64\"/sha256 \"${SHA_MACOS_X64}\"/" "$FORMULA_FILE"
sed -i.bak "s/sha256 \"PLACEHOLDER_SHA256_LINUX_X64\"/sha256 \"${SHA_LINUX_X64}\"/" "$FORMULA_FILE"

# Also update any existing checksums (in case they're already set)
sed -i.bak "s/sha256 \"[a-f0-9]\{64\}\" # macOS ARM64/sha256 \"${SHA_MACOS_ARM64}\"/" "$FORMULA_FILE" 2>/dev/null || true
sed -i.bak "s/sha256 \"[a-f0-9]\{64\}\" # macOS x64/sha256 \"${SHA_MACOS_X64}\"/" "$FORMULA_FILE" 2>/dev/null || true
sed -i.bak "s/sha256 \"[a-f0-9]\{64\}\" # Linux x64/sha256 \"${SHA_LINUX_X64}\"/" "$FORMULA_FILE" 2>/dev/null || true

# Remove backup file
rm -f "${FORMULA_FILE}.bak"

echo "✅ Formula updated successfully!"
echo ""
echo "📋 Next steps:"
echo "1. Review the changes: git diff Formula/apicentric.rb"
echo "2. Test the formula: brew install --build-from-source Formula/apicentric.rb"
echo "3. Run tests: brew test apicentric"
echo "4. Commit and push: git add Formula/apicentric.rb && git commit -m 'Update to v${VERSION}' && git push"
