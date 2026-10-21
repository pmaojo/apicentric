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
cargo test --all-features

# 4. Feature Flags Check
echo "🚩 Checking feature flags..."
cargo check --no-default-features --features minimal
cargo check --no-default-features --features cli-tools

# 5. Build Verified
echo "🏗️  Verifying full build..."
cargo build --all-features

# 6. Frontend Check
echo "🌐 Checking frontend..."
cd webui
npm run lint
npm run typecheck
cd ..

echo "✅ Health Check Passed!"
