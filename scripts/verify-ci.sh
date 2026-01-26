#!/bin/bash
# CI Verification Script
# This script runs the same checks that CI runs to verify everything works locally

set -e

echo "🔍 Apicentric CI Verification Script"
echo "===================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILURES=0

# Function to run a check
run_check() {
    local name="$1"
    local command="$2"
    
    echo -e "${YELLOW}Running: ${name}${NC}"
    if eval "$command"; then
        echo -e "${GREEN}✓ ${name} passed${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}✗ ${name} failed${NC}"
        echo ""
        FAILURES=$((FAILURES + 1))
        return 1
    fi
}

# 1. Format Check
echo "📝 Step 1: Format Check"
echo "----------------------"
run_check "cargo fmt --check" "cargo fmt --all -- --check"

# 2. Lint Check
echo "🔎 Step 2: Lint Check"
echo "--------------------"
run_check "cargo clippy" "cargo clippy --all-targets --all-features -- -D warnings"

# 3. Build Tests
echo "🔨 Step 3: Build Tests"
echo "---------------------"

run_check "Minimal build" "cargo build --no-default-features --features minimal"
run_check "Default build" "cargo build"
run_check "CLI tools build" "cargo build --features cli-tools"
run_check "Full build" "cargo build --all-features"

# 4. Test Suites
echo "🧪 Step 4: Test Suites"
echo "---------------------"

run_check "Tests (minimal)" "cargo test --no-default-features --features minimal"
run_check "Tests (default)" "cargo test"
run_check "Tests (full)" "cargo test --all-features"

# 5. Security Audit
echo "🔒 Step 5: Security Audit"
echo "------------------------"

if command -v cargo-audit &> /dev/null; then
    run_check "cargo audit" "cargo audit"
else
    echo -e "${YELLOW}⚠ cargo-audit not installed, skipping${NC}"
    echo "  Install with: cargo install cargo-audit"
    echo ""
fi

# 6. Documentation Check
echo "📚 Step 6: Documentation Check"
echo "------------------------------"

run_check "cargo doc" "cargo doc --all-features --no-deps"

# Summary
echo "=================================="
echo "📊 Verification Summary"
echo "=================================="

if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo ""
    echo "Your code is ready for CI. All checks that run in GitHub Actions"
    echo "have passed locally."
    exit 0
else
    echo -e "${RED}✗ ${FAILURES} check(s) failed${NC}"
    echo ""
    echo "Please fix the failing checks before pushing to CI."
    exit 1
fi
