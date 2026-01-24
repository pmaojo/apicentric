#   ___  ______ _____ _____ _____ _   _ _____ ______ _____ _____ 
#  / _ \ | ___ \_   _/  __ \  ___| \ | |_   _|| ___ \_   _/  __ \
# / /_\ \| |_/ / | | | /  \/ |__ |  \| | | |  | |_/ / | | | /  \/
# |  _  ||  __/  | | | |   |  __|| . ` | | |  |    /  | | | |    
# | | | || |    _| |_| \__/\ |___| |\  | | |  | |\ \ _| |_| \__/\
# \_| |_/\_|    \___/ \____\____/\_| \_/ \_/  \_| \_|\___/ \____/
#                                                                
# High-performance API Simulator & Digital Twin Framework

.PHONY: all help build build-backend build-frontend run dev test lint format clean docker-build package doctor demo install

# Variables
BINARY=target/debug/apicentric
RELEASE_BINARY=target/release/apicentric
SERVICES_DIR=examples
DB_PATH=data/apicentric.db

# Default target
all: build

help:
	@printf '                                                                \n'
	@printf '  \033[1;36m___  ______ _____ _____ _____ _   _ _____ ______ _____ _____ \033[0m\n'
	@printf '  \033[1;36m/ _ \\ | ___ \\_   _/  __ \\  ___| \\ | |_   _|| ___ \\_   _/  __ \\\\ \033[0m\n'
	@printf '  \033[1;36m/ /_\\ \\| |_/ / | | | /  \\/ |__ |  \\| | | |  | |_/ / | | | /  \\/\033[0m\n'
	@printf '  \033[1;36m|  _  ||  __/  | | | |   |  __|| . ` | | |  |    /  | | | |    \033[0m\n'
	@printf '  \033[1;36m| | | || |    _| |_| \\__/\\ |___| |\\  | | |  | |\\ \\ _| |_| \\__/\033[0m\n'
	@printf '  \033[1;36m\\_| |_/\\_|    \\___/ \\____\\____/\\_| \\_/ \\_/  \\_| \\_|\\___/ \\____/\033[0m\n'
	@printf '                                                                \n'
	@printf '  \033[1;33m⚡ Core Commands:\033[0m\n'
	@printf '    \033[1;32mmake build\033[0m          - 🏗️  Build complete project (Backend & Frontend)\n'
	@printf '    \033[1;32mmake run\033[0m            - 🏃 Start simulator with examples\n'
	@printf '    \033[1;32mmake dev\033[0m            - 🚀 Hot-reload development environment\n'
	@printf '    \033[1;32mmake wizard\033[0m         - 🧙 Interactive project management wizard\n'
	@printf '  \n'
	@printf '  \033[1;33m🎮 Interfaces:\033[0m\n'
	@printf '    \033[1;32mmake tui\033[0m            - 🖥️  Launch Terminal User Interface\n'
	@printf '    \033[1;32mmake gui\033[0m            - 🎨 Launch Desktop GUI Administration\n'
	@printf '  \n'
	@printf '  \033[1;33m🔬 Simulation & Testing:\033[0m\n'
	@printf '    \033[1;32mmake twin\033[0m           - 🤖 Run IoT Digital Twin simulator\n'
	@printf '    \033[1;32mmake contract\033[0m       - 📝 Execute contract-driven API tests\n'
	@printf '    \033[1;32mmake cloud\033[0m          - ☁️  Start server in cloud-ready mode\n'
	@printf '  \n'
	@printf '  \033[1;33m🛠️  Development & QA:\033[0m\n'
	@printf '    \033[1;32mmake test\033[0m           - 🧪 Run full test suite\n'
	@printf '    \033[1;32mmake lint\033[0m           - 🧹 Static analysis & linting\n'
	@printf '    \033[1;32mmake format\033[0m         - 🎨 Auto-format all source code\n'
	@printf '    \033[1;32mmake doctor\033[0m         - 🩺 Run project diagnostics\n'
	@printf '  \n'
	@printf '  \033[1;33m📦 Distribution:\033[0m\n'
	@printf '    \033[1;32mmake package\033[0m        - 📦 Create production tarball\n'
	@printf '    \033[1;32mmake docker-build\033[0m   - 🐳 Build release Docker image\n'
	@printf '    \033[1;32mmake demo\033[0m           - 🎬 Run visual demonstration\n'
	@printf '  \n'
	@printf '  \033[1;33m🧹 Utility:\033[0m\n'
	@printf '    \033[1;32mmake clean\033[0m          - 🗑️  Deep clean build artifacts\n'
	@printf '    \033[1;32mmake install\033[0m        - 📥 Bootstrap dependencies\n'
	@printf '  \n'
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

tui: build-backend
	@echo "🖥️  Launching TUI..."
	$(BINARY) tui

gui: build-backend
	@echo "🎨 Launching GUI..."
	$(BINARY) gui

twin: build-backend
	@echo "🤖 Starting IoT Digital Twin..."
	$(BINARY) twin run --device examples/iot/smarthome/thermostat-nest.yaml

contract: build-backend
	@echo "📝 Running contract tests..."
	$(BINARY) simulator test --path examples/iot/smarthome/thermostat-nest.yaml --url http://localhost:9005

cloud: build-backend
	@echo "☁️  Starting cloud mode..."
	$(BINARY) cloud --port 8080

wizard:
	@chmod +x scripts/wizard.sh
	@./scripts/wizard.sh

docker-build:
	@echo "🐳 Building Docker images..."
	docker build -t apicentric:latest .
