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

echo "✅ Health Check Passed!"
