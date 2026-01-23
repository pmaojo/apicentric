# Apicentric Makefile
# Comprehensive build and management system

.PHONY: all help build build-backend build-frontend run dev test lint format clean docker-build package doctor demo install

# Variables
BINARY=target/debug/apicentric
RELEASE_BINARY=target/release/apicentric
SERVICES_DIR=examples
DB_PATH=data/apicentric.db

# Default target
all: build

help:
	@echo "🔍 Available commands:"
	@echo "  make install        - Install dependencies (Rust & Node.js)"
	@echo "  make build          - Build backend and frontend"
	@echo "  make build-backend  - Build Rust backend"
	@echo "  make build-frontend - Build Next.js frontend"
	@echo "  make run            - Run the simulator with example services"
	@echo "  make dev            - Run backend and frontend in dev mode"
	@echo "  make test           - Run all tests (Rust & Integration)"
	@echo "  make lint           - Run clippy and frontend linter"
	@echo "  make format         - Format code (Rust & Frontend)"
	@echo "  make clean          - Remove build artifacts and logs"
	@echo "  make doctor         - Run diagnostic checks"
	@echo "  make demo           - Run the full demo suite"
	@echo "  make package        - Create a production deployment package"

install:
	@echo "📦 Installing dependencies..."
	cargo fetch
	cd webui && npm install

build: build-backend build-frontend

build-backend:
	@echo "🦀 Building backend (debug)..."
	cargo build --features full

build-frontend:
	@echo "📦 Building frontend..."
	cd webui && npm run build

run: build-backend
	@echo "🏃 Starting simulator..."
	mkdir -p data
	$(BINARY) --db-path $(DB_PATH) simulator start --services-dir $(SERVICES_DIR)

dev:
	@echo "🚀 Starting development environment..."
	# This runs both in parallel, might need adjustment for local workflow
	(cargo watch -x run & cd webui && npm run dev)

test:
	@echo "🧪 Running tests..."
	cargo test
	# Run scripts/test_mcp.sh if it exists
	@[ -f scripts/test_mcp.sh ] && ./scripts/test_mcp.sh || echo "No MCP tests found"

lint:
	@echo "🧹 Linting..."
	cargo clippy -- -D warnings
	cd webui && npm run lint

format:
	@echo "🎨 Formatting..."
	cargo fmt
	cd webui && npm run format

clean:
	@echo "🧹 Cleaning up..."
	cargo clean
	rm -rf webui/.next
	rm -rf logs/*
	rm -rf demo_output

doctor: build-backend
	@$(BINARY) doctor

demo: build-backend
	@echo "🎬 Running demo..."
	./scripts/demo_all.sh

package:
	@echo "📦 Packaging for production..."
	./scripts/build-production.sh

docker-build:
	@echo "🐳 Building Docker images..."
	docker build -t apicentric:latest .
