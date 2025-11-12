#!/bin/bash
# Production build script for Apicentric

set -e

echo "🏗️  Building Apicentric for production..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build frontend
echo -e "${BLUE}📦 Building frontend...${NC}"
cd webui
npm ci --only=production
npm run build
cd ..

echo -e "${GREEN}✅ Frontend build complete${NC}"

# Build backend
echo -e "${BLUE}🦀 Building backend...${NC}"
cargo build --release --features "gui,cloud"

echo -e "${GREEN}✅ Backend build complete${NC}"

# Create deployment directory
echo -e "${BLUE}📁 Creating deployment package...${NC}"
mkdir -p deploy
cp target/release/apicentric deploy/
cp -r webui/.next/standalone deploy/webui
cp -r webui/.next/static deploy/webui/.next/static
cp -r webui/public deploy/webui/public

echo -e "${GREEN}✅ Deployment package created in ./deploy${NC}"

# Create tarball
echo -e "${BLUE}📦 Creating tarball...${NC}"
tar -czf apicentric-$(date +%Y%m%d-%H%M%S).tar.gz -C deploy .

echo -e "${GREEN}✅ Build complete!${NC}"
echo ""
echo "To deploy:"
echo "  1. Extract tarball on target server"
echo "  2. Set environment variables (see .env.example)"
echo "  3. Run: ./apicentric cloud --port 8000"
