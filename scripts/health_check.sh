#!/bin/bash
set -e

echo "🏥 Starting Health Check..."

# 0. Check Manifest
echo "📜 Checking manifest..."
cargo check

# 1. Format Check
echo "🎨 Checking formatting..."
cargo fmt --all -- --check

# 2. Linting (Clippy)
echo "🧹 Running Clippy (Strict Mode)..."
cargo clippy --all-targets --all-features -- -D warnings

# 3. Tests
echo "🧪 Running unit tests..."
cargo test

# 4. Build Verified
echo "🏗️  Verifying full build..."
cargo build --all-features

# 5. WebUI Validation
if [ -d "webui" ]; then
    echo "📦 Checking WebUI..."
    cd webui

    echo "  - Installing dependencies..."
    npm install

    echo "  - Linting..."
    npm run lint

    echo "  - Building..."
    npm run build

    cd ..
fi

echo "✅ Health Check Passed!"
