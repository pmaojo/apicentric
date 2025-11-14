#!/bin/bash

# 🚀 Apicentric E2E Testing Suite
# This script starts the backend, frontend, and runs the complete E2E test suite

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BACKEND_PORT=8080
FRONTEND_PORT=9002
BACKEND_URL="http://localhost:${BACKEND_PORT}"
FRONTEND_URL="http://localhost:${FRONTEND_PORT}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
WEBUI_DIR="${PROJECT_ROOT}/webui"

# PID files for cleanup
BACKEND_PID_FILE="/tmp/apicentric-backend.pid"
FRONTEND_PID_FILE="/tmp/apicentric-frontend.pid"

# Default values
RUN_BACKEND=true
RUN_FRONTEND=true
RUN_TESTS=true
HEADLESS=true
BROWSER="chromium"
CLEANUP_ON_EXIT=true
VERBOSE=false
TIMEOUT=120

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    cat << EOF
🚀 Apicentric E2E Test Runner

Usage: $0 [OPTIONS]

OPTIONS:
    -h, --help              Show this help message
    -b, --backend-only      Only start backend (skip frontend and tests)
    -f, --frontend-only     Only start frontend (skip backend and tests)
    -t, --tests-only        Only run tests (assume backend/frontend running)
    --no-backend           Skip starting backend
    --no-frontend          Skip starting frontend  
    --no-tests             Skip running tests
    --headed               Run tests in headed mode (visible browser)
    --browser BROWSER      Browser to use (chromium, firefox, webkit)
    --no-cleanup           Don't cleanup processes on exit
    --verbose              Verbose output
    --timeout SECONDS      Timeout for services to start (default: 120)
    --backend-port PORT    Backend port (default: 8080)
    --frontend-port PORT   Frontend port (default: 9002)

EXAMPLES:
    $0                     # Run complete E2E suite
    $0 --headed            # Run with visible browser
    $0 --browser firefox   # Run tests in Firefox
    $0 --tests-only        # Only run tests (services already running)
    $0 --no-cleanup        # Keep services running after tests
    $0 --verbose           # Show detailed output

EOF
}

# Function to parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -b|--backend-only)
                RUN_FRONTEND=false
                RUN_TESTS=false
                shift
                ;;
            -f|--frontend-only)
                RUN_BACKEND=false
                RUN_TESTS=false
                shift
                ;;
            -t|--tests-only)
                RUN_BACKEND=false
                RUN_FRONTEND=false
                shift
                ;;
            --no-backend)
                RUN_BACKEND=false
                shift
                ;;
            --no-frontend)
                RUN_FRONTEND=false
                shift
                ;;
            --no-tests)
                RUN_TESTS=false
                shift
                ;;
            --headed)
                HEADLESS=false
                shift
                ;;
            --browser)
                BROWSER="$2"
                shift 2
                ;;
            --no-cleanup)
                CLEANUP_ON_EXIT=false
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --timeout)
                TIMEOUT="$2"
                shift 2
                ;;
            --backend-port)
                BACKEND_PORT="$2"
                BACKEND_URL="http://localhost:${BACKEND_PORT}"
                shift 2
                ;;
            --frontend-port)
                FRONTEND_PORT="$2"
                FRONTEND_URL="http://localhost:${FRONTEND_PORT}"
                shift 2
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
}

# Function to check if a port is in use
check_port() {
    local port=$1
    if lsof -i :$port >/dev/null 2>&1; then
        return 0  # Port is in use
    else
        return 1  # Port is free
    fi
}

# Function to wait for service to be ready
wait_for_service() {
    local url=$1
    local service_name=$2
    local timeout=$3
    local count=0
    
    print_status "Waiting for $service_name to be ready at $url..."
    
    while [ $count -lt $timeout ]; do
        if curl -s "$url" >/dev/null 2>&1; then
            print_success "$service_name is ready!"
            return 0
        fi
        sleep 1
        count=$((count + 1))
        if [ $((count % 10)) -eq 0 ]; then
            print_status "Still waiting for $service_name... (${count}s/${timeout}s)"
        fi
    done
    
    print_error "$service_name failed to start within ${timeout} seconds"
    return 1
}

# Function to start backend
start_backend() {
    if [ "$RUN_BACKEND" = false ]; then
        return 0
    fi
    
    print_status "🔧 Starting Apicentric backend..."
    
    # Check if backend is already running
    if check_port $BACKEND_PORT; then
        print_warning "Backend port $BACKEND_PORT is already in use"
        read -p "Kill existing process and continue? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_status "Killing process on port $BACKEND_PORT..."
            lsof -ti:$BACKEND_PORT | xargs kill -9 2>/dev/null || true
            sleep 2
        else
            print_status "Using existing backend service"
            return 0
        fi
    fi
    
    # Navigate to project root
    cd "$PROJECT_ROOT"
    
    # Build and start backend
    if [ "$VERBOSE" = true ]; then
        print_status "Building backend..."
        cargo build --release
    else
        print_status "Building backend (this may take a while)..."
        cargo build --release >/dev/null 2>&1
    fi
    
    # Start backend server
    print_status "Starting backend server on port $BACKEND_PORT..."
    if [ "$VERBOSE" = true ]; then
        nohup cargo run --release -- cloud --port $BACKEND_PORT > /tmp/apicentric-backend.log 2>&1 &
    else
        nohup cargo run --release -- cloud --port $BACKEND_PORT >/dev/null 2>&1 &
    fi
    
    BACKEND_PID=$!
    echo $BACKEND_PID > "$BACKEND_PID_FILE"
    
    # Wait for backend to be ready
    if ! wait_for_service "${BACKEND_URL}/health" "Backend" $TIMEOUT; then
        print_error "Failed to start backend"
        cleanup_processes
        exit 1
    fi
    
    print_success "Backend started successfully (PID: $BACKEND_PID)"
}

# Function to start frontend
start_frontend() {
    if [ "$RUN_FRONTEND" = false ]; then
        return 0
    fi
    
    print_status "🌐 Starting frontend..."
    
    # Check if frontend is already running
    if check_port $FRONTEND_PORT; then
        print_warning "Frontend port $FRONTEND_PORT is already in use"
        read -p "Kill existing process and continue? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_status "Killing process on port $FRONTEND_PORT..."
            lsof -ti:$FRONTEND_PORT | xargs kill -9 2>/dev/null || true
            sleep 2
        else
            print_status "Using existing frontend service"
            return 0
        fi
    fi
    
    # Navigate to webui directory
    cd "$WEBUI_DIR"
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ] || [ "package.json" -nt "node_modules" ]; then
        print_status "Installing frontend dependencies..."
        if [ "$VERBOSE" = true ]; then
            npm install
        else
            npm install >/dev/null 2>&1
        fi
    fi
    
    # Set environment variables
    export NEXT_PUBLIC_API_URL="$BACKEND_URL"
    export PORT="$FRONTEND_PORT"
    
    # Start frontend development server
    print_status "Starting frontend server on port $FRONTEND_PORT..."
    if [ "$VERBOSE" = true ]; then
        nohup npm run dev > /tmp/apicentric-frontend.log 2>&1 &
    else
        nohup npm run dev >/dev/null 2>&1 &
    fi
    
    FRONTEND_PID=$!
    echo $FRONTEND_PID > "$FRONTEND_PID_FILE"
    
    # Wait for frontend to be ready
    if ! wait_for_service "$FRONTEND_URL" "Frontend" $TIMEOUT; then
        print_error "Failed to start frontend"
        cleanup_processes
        exit 1
    fi
    
    print_success "Frontend started successfully (PID: $FRONTEND_PID)"
}

# Function to run E2E tests
run_tests() {
    if [ "$RUN_TESTS" = false ]; then
        return 0
    fi
    
    print_status "🧪 Running E2E tests..."
    
    # Navigate to webui directory
    cd "$WEBUI_DIR"
    
    # Set environment variables for tests
    export BACKEND_URL="$BACKEND_URL"
    export FRONTEND_URL="$FRONTEND_URL"
    
    # Prepare test command
    local test_cmd="npx playwright test"
    
    if [ "$HEADLESS" = false ]; then
        test_cmd="$test_cmd --headed"
    fi
    
    if [ "$BROWSER" != "chromium" ]; then
        test_cmd="$test_cmd --project=$BROWSER"
    fi
    
    if [ "$VERBOSE" = true ]; then
        test_cmd="$test_cmd --reporter=line"
    fi
    
    print_status "Running command: $test_cmd"
    
    # Run tests
    if $test_cmd; then
        print_success "All E2E tests passed! 🎉"
        
        # Generate and show report
        print_status "Generating test report..."
        npx playwright show-report --host 0.0.0.0 &
        REPORT_PID=$!
        sleep 2
        print_success "Test report available at: http://localhost:9323"
        kill $REPORT_PID 2>/dev/null || true
        
        return 0
    else
        print_error "Some E2E tests failed 😞"
        
        # Show failed test results
        print_status "Generating failure report..."
        npx playwright show-report --host 0.0.0.0 &
        REPORT_PID=$!
        sleep 2
        print_warning "Failure report available at: http://localhost:9323"
        print_warning "Press Ctrl+C to stop the report server"
        wait $REPORT_PID 2>/dev/null || true
        
        return 1
    fi
}

# Function to cleanup processes
cleanup_processes() {
    if [ "$CLEANUP_ON_EXIT" = false ]; then
        print_status "Skipping cleanup (--no-cleanup flag used)"
        return 0
    fi
    
    print_status "🧹 Cleaning up processes..."
    
    # Kill backend
    if [ -f "$BACKEND_PID_FILE" ]; then
        BACKEND_PID=$(cat "$BACKEND_PID_FILE")
        if kill -0 $BACKEND_PID 2>/dev/null; then
            print_status "Stopping backend (PID: $BACKEND_PID)..."
            kill $BACKEND_PID 2>/dev/null || true
            sleep 2
            kill -9 $BACKEND_PID 2>/dev/null || true
        fi
        rm -f "$BACKEND_PID_FILE"
    fi
    
    # Kill frontend
    if [ -f "$FRONTEND_PID_FILE" ]; then
        FRONTEND_PID=$(cat "$FRONTEND_PID_FILE")
        if kill -0 $FRONTEND_PID 2>/dev/null; then
            print_status "Stopping frontend (PID: $FRONTEND_PID)..."
            kill $FRONTEND_PID 2>/dev/null || true
            sleep 2
            kill -9 $FRONTEND_PID 2>/dev/null || true
        fi
        rm -f "$FRONTEND_PID_FILE"
    fi
    
    # Kill any remaining processes on our ports
    lsof -ti:$BACKEND_PORT | xargs kill -9 2>/dev/null || true
    lsof -ti:$FRONTEND_PORT | xargs kill -9 2>/dev/null || true
    
    print_success "Cleanup completed"
}

# Function to handle script interruption
handle_interrupt() {
    print_warning "Received interrupt signal"
    cleanup_processes
    exit 130
}

# Function to check dependencies
check_dependencies() {
    print_status "🔍 Checking dependencies..."
    
    # Check if we're in the right directory
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        print_error "Cargo.toml not found. Are you in the Apicentric project root?"
        exit 1
    fi
    
    if [ ! -f "$WEBUI_DIR/package.json" ]; then
        print_error "Frontend package.json not found at $WEBUI_DIR"
        exit 1
    fi
    
    # Check required tools
    local missing_tools=()
    
    if ! command -v cargo >/dev/null 2>&1; then
        missing_tools+=("cargo (Rust)")
    fi
    
    if ! command -v node >/dev/null 2>&1; then
        missing_tools+=("node.js")
    fi
    
    if ! command -v npm >/dev/null 2>&1; then
        missing_tools+=("npm")
    fi
    
    if ! command -v curl >/dev/null 2>&1; then
        missing_tools+=("curl")
    fi
    
    if ! command -v lsof >/dev/null 2>&1; then
        missing_tools+=("lsof")
    fi
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        print_error "Missing required tools:"
        for tool in "${missing_tools[@]}"; do
            echo "  - $tool"
        done
        exit 1
    fi
    
    print_success "All dependencies found"
}

# Main function
main() {
    print_status "🚀 Apicentric E2E Test Suite Starting..."
    
    # Parse command line arguments
    parse_args "$@"
    
    # Set up signal handlers
    trap handle_interrupt SIGINT SIGTERM
    
    # Check dependencies
    check_dependencies
    
    # Print configuration
    print_status "Configuration:"
    echo "  Backend: $($RUN_BACKEND && echo "START" || echo "SKIP") ($BACKEND_URL)"
    echo "  Frontend: $($RUN_FRONTEND && echo "START" || echo "SKIP") ($FRONTEND_URL)"
    echo "  Tests: $($RUN_TESTS && echo "RUN" || echo "SKIP") ($([ "$HEADLESS" = true ] && echo "headless" || echo "headed") $BROWSER)"
    echo "  Cleanup: $($CLEANUP_ON_EXIT && echo "YES" || echo "NO")"
    echo ""
    
    # Start services and run tests
    local exit_code=0
    
    start_backend || exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        start_frontend || exit_code=$?
    fi
    
    if [ $exit_code -eq 0 ]; then
        run_tests || exit_code=$?
    fi
    
    # Show summary
    if [ $exit_code -eq 0 ]; then
        print_success "🎉 E2E Test Suite completed successfully!"
        echo ""
        echo "Services running:"
        [ "$RUN_BACKEND" = true ] && echo "  Backend:  $BACKEND_URL"
        [ "$RUN_FRONTEND" = true ] && echo "  Frontend: $FRONTEND_URL"
        echo ""
        
        if [ "$CLEANUP_ON_EXIT" = false ]; then
            print_status "Services are still running. Use the following to stop them:"
            echo "  kill \$(cat $BACKEND_PID_FILE) 2>/dev/null || true"
            echo "  kill \$(cat $FRONTEND_PID_FILE) 2>/dev/null || true"
        fi
    else
        print_error "❌ E2E Test Suite failed with exit code $exit_code"
    fi
    
    # Cleanup
    cleanup_processes
    
    exit $exit_code
}

# Run main function
main "$@"