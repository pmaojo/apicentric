#!/bin/bash
set -e

echo "🚀 Starting Release Process..."

# 1. Health Check
echo "🩺 Running Health Check..."
./scripts/health_check.sh

# 2. Build Release Binary
echo "🔨 Building Release Binary..."
cargo build --release

# 3. Docker Build
echo "🐳 Building Docker Image..."
docker build -f Dockerfile.production -t apicentric:latest .

echo "✅ Release Build Support Complete!"
echo "   - Binary: target/release/apicentric"
echo "   - Docker Image: apicentric:latest"
