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

# 5. Frontend Verified
echo "🌐 Verifying frontend..."
if [ -d "webui" ]; then
    cd webui
    # Install dependencies if node_modules doesn't exist
    if [ ! -d "node_modules" ]; then
        echo "📦 Installing frontend dependencies..."
        npm install
    fi

    echo "🧹 Linting frontend..."
    npm run lint

    echo "🏗️  Building frontend..."
    npm run build

    cd ..
else
    echo "⚠️  webui directory not found, skipping frontend check."
fi

echo "✅ Health Check Passed!"
