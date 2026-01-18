#!/bin/bash

# Apicentric Cloud Deployment Script
# This script helps you deploy Apicentric to various environments

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

show_help() {
    cat << EOF
Apicentric Cloud Deployment Script

Usage: $0 [OPTION]

OPTIONS:
    dev         Start development environment
    build       Build the Docker image
    deploy      Deploy to production (requires docker-compose)
    stop        Stop all running containers
    logs        Show application logs
    test        Run health check tests
    clean       Clean up containers and images
    help        Show this help message

EXAMPLES:
    $0 dev              # Start development environment
    $0 build            # Build production Docker image
    $0 deploy           # Deploy to production
    $0 logs             # View logs
    $0 test             # Run health checks

EOF
}

check_requirements() {
    log "Checking requirements..."

    if ! command -v docker &> /dev/null; then
        error "Docker is not installed. Please install Docker first."
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null && ! command -v docker &> /dev/null; then
        error "Docker Compose is not available. Please install Docker Compose."
        exit 1
    fi

    success "All requirements satisfied"
}

dev() {
    log "Starting development environment..."
    cargo run --bin apicentric-cloud -- --port 3000
}

build() {
    log "Building Docker image..."
    docker build -f packaging/Dockerfile -t apicentric-cloud:latest .
    success "Docker image built successfully"
}

deploy() {
    check_requirements
    log "Deploying Apicentric Cloud..."

    # Create necessary directories
    mkdir -p services logs

    # Start the services
    docker-compose up -d

    # Wait for health check
    log "Waiting for service to be ready..."
    sleep 10

    if docker-compose exec apicentric-cloud wget --spider -q http://localhost:8080/health; then
        success "Apicentric Cloud deployed successfully!"
        log "API available at: http://localhost:8080"
        log "Health check: http://localhost:8080/health"
    else
        error "Deployment failed - service is not responding"
        exit 1
    fi
}

stop() {
    log "Stopping Apicentric Cloud..."
    docker-compose down
    success "Services stopped"
}

logs() {
    log "Showing application logs..."
    docker-compose logs -f apicentric-cloud
}

test_health() {
    log "Running health checks..."

    # Check if container is running
    if ! docker-compose ps | grep -q "Up"; then
        error "No running containers found"
        exit 1
    fi

    # Test health endpoint
    if curl -s http://localhost:8080/health | grep -q "healthy"; then
        success "Health check passed"
    else
        error "Health check failed"
        exit 1
    fi

    # Test API endpoint
    if curl -s http://localhost:8080/api/services | grep -q "success"; then
        success "API check passed"
    else
        error "API check failed"
        exit 1
    fi
}

clean() {
    log "Cleaning up..."
    docker-compose down --rmi all --volumes --remove-orphans
    docker system prune -f
    success "Cleanup completed"
}

# Main script logic
case "${1:-}" in
    "dev")
        dev
        ;;
    "build")
        build
        ;;
    "deploy")
        deploy
        ;;
    "stop")
        stop
        ;;
    "logs")
        logs
        ;;
    "test")
        test_health
        ;;
    "clean")
        clean
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    "")
        warn "No option provided. Use 'help' to see available options."
        show_help
        exit 1
        ;;
    *)
        error "Unknown option: $1"
        show_help
        exit 1
        ;;
esac