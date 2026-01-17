#!/bin/bash
set -e

echo "🏥 Starting Health Check..."

# 1. Format Check
echo "🎨 Checking formatting..."
cargo fmt --all -- --check

# 2. Linting (Clippy)
echo "🧹 Running Clippy..."
cargo clippy

# 3. Tests
echo "🧪 Running unit tests..."
cargo test

# 4. Build Verified
echo "🏗️  Verifying build..."
cargo build

echo "✅ Health Check Passed!"
