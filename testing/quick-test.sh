#!/bin/bash

# 🚀 Quick E2E Test Runner - Desarrollo rápido
# Script simplificado para desarrollo y debugging

set -e

# Colores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_status() { echo -e "${YELLOW}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Navegar al directorio webui
cd "$(dirname "$0")/../webui"

print_status "🧪 Quick E2E Test Runner"

# Verificar que el backend esté corriendo
if ! curl -s http://localhost:8080/health >/dev/null 2>&1; then
    print_error "Backend not running on port 8080"
    print_status "Start backend with: cargo run --release -- cloud --port 8080"
    exit 1
fi

# Verificar que el frontend esté corriendo
if ! curl -s http://localhost:9002 >/dev/null 2>&1; then
    print_error "Frontend not running on port 9002"
    print_status "Start frontend with: cd webui && npm run dev"
    exit 1
fi

print_success "Backend and frontend are running"

# Ejecutar tests según parámetro
case "${1:-all}" in
    "nav"|"navigation")
        print_status "Running navigation tests..."
        npx playwright test 01-basic-navigation.spec.ts --headed
        ;;
    "dash"|"dashboard")
        print_status "Running dashboard tests..."
        npx playwright test 02-dashboard.spec.ts --headed
        ;;
    "services"|"service")
        print_status "Running service management tests..."
        npx playwright test 03-service-management.spec.ts --headed
        ;;
    "backend"|"api")
        print_status "Running backend integration tests..."
        npx playwright test 04-backend-integration.spec.ts --headed
        ;;
    "headless")
        print_status "Running all tests headless..."
        npx playwright test
        ;;
    "debug")
        print_status "Running in debug mode..."
        npx playwright test --debug
        ;;
    "ui")
        print_status "Opening Playwright UI..."
        npx playwright test --ui
        ;;
    "report")
        print_status "Opening test report..."
        npx playwright show-report
        ;;
    *)
        print_status "Running all tests with browser visible..."
        npx playwright test --headed
        ;;
esac

print_success "Tests completed! 🎉"