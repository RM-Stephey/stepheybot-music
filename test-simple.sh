#!/bin/bash

# Simple test script for StepheyBot Music Multi-User API
# This script performs basic compilation and functionality checks

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper functions
log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED_TESTS++))
}

error() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED_TESTS++))
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

run_test() {
    ((TOTAL_TESTS++))
    echo -e "\n${PURPLE}[TEST $TOTAL_TESTS]${NC} $1"
}

# Test 1: Check Rust toolchain
test_rust_toolchain() {
    run_test "Checking Rust toolchain"

    if cargo --version >/dev/null 2>&1; then
        success "Cargo is available: $(cargo --version)"
    else
        error "Cargo is not available"
        return 1
    fi

    if rustc --version >/dev/null 2>&1; then
        success "Rust compiler is available: $(rustc --version)"
    else
        error "Rust compiler is not available"
        return 1
    fi
}

# Test 2: Check dependencies
test_dependencies() {
    run_test "Checking project dependencies"

    if [ -f "Cargo.toml" ]; then
        success "Cargo.toml found"
    else
        error "Cargo.toml not found"
        return 1
    fi

    if cargo fetch >/dev/null 2>&1; then
        success "Dependencies can be fetched"
    else
        error "Failed to fetch dependencies"
        return 1
    fi
}

# Test 3: Compilation test
test_compilation() {
    run_test "Testing compilation"

    log "Running cargo check..."
    check_output=$(cargo check 2>&1)
    if echo "$check_output" | grep -q "^error\[E[0-9]*\]:"; then
        error "Compilation failed with errors"
        echo "$check_output"
        return 1
    elif echo "$check_output" | grep -q "could not compile"; then
        error "Compilation failed"
        echo "$check_output"
        return 1
    else
        success "Project compiles without errors"
        warning_count=$(echo "$check_output" | grep -c "^warning:" || echo "0")
        if [ "$warning_count" -gt 0 ]; then
            info "Found $warning_count warnings (this is normal during development)"
        fi
    fi

    log "Running cargo build..."
    if cargo build >/dev/null 2>&1; then
        success "Project builds successfully"
    else
        error "Build failed"
        return 1
    fi
}

# Test 4: Unit tests
test_unit_tests() {
    run_test "Running unit tests"

    log "Running cargo test..."
    if cargo test --lib 2>&1 | tee test_output.tmp; then
        test_count=$(grep -c "test result:" test_output.tmp || echo "0")
        if [ "$test_count" -gt 0 ]; then
            success "Unit tests completed"
            grep "test result:" test_output.tmp | while read line; do
                info "$line"
            done
        else
            warning "No unit tests found to run"
        fi
    else
        error "Unit tests failed"
        return 1
    fi

    rm -f test_output.tmp
}

# Test 5: Documentation test
test_documentation() {
    run_test "Testing documentation"

    if cargo doc --no-deps >/dev/null 2>&1; then
        success "Documentation builds successfully"
    else
        warning "Documentation build had issues (this is usually okay)"
    fi
}

# Test 6: Code formatting
test_formatting() {
    run_test "Checking code formatting"

    if command -v rustfmt >/dev/null 2>&1; then
        if cargo fmt --check >/dev/null 2>&1; then
            success "Code is properly formatted"
        else
            warning "Code formatting issues found (run 'cargo fmt' to fix)"
        fi
    else
        info "rustfmt not available, skipping formatting check"
    fi
}

# Test 7: Clippy lints
test_lints() {
    run_test "Running Clippy lints"

    if command -v cargo-clippy >/dev/null 2>&1; then
        if cargo clippy --all-targets --all-features -- -D warnings >/dev/null 2>&1; then
            success "No clippy warnings found"
        else
            warning "Clippy found some issues (this is normal during development)"
        fi
    else
        info "Clippy not available, skipping lint check"
    fi
}

# Test 8: Feature compilation
test_features() {
    run_test "Testing feature compilation"

    # Test default features
    if cargo check --no-default-features >/dev/null 2>&1; then
        success "Compiles without default features"
    else
        warning "Issues compiling without default features"
    fi

    # Test individual features if they exist
    if grep -q "\[features\]" Cargo.toml; then
        info "Features found in Cargo.toml:"
        grep -A 10 "\[features\]" Cargo.toml | grep "=" | head -5 | while read line; do
            info "  $line"
        done
        success "Feature configuration is present"
    else
        info "No custom features defined"
    fi
}

# Test 9: Binary creation
test_binary() {
    run_test "Testing binary creation"

    if cargo build --release >/dev/null 2>&1; then
        success "Release binary builds successfully"

        binary_path="target/release/stepheybot-music"
        if [ -f "$binary_path" ]; then
            success "Binary created at $binary_path"
            file_size=$(ls -lh "$binary_path" | awk '{print $5}')
            info "Binary size: $file_size"
        else
            warning "Binary not found at expected location"
        fi
    else
        error "Release build failed"
        return 1
    fi
}

# Test 10: API structure validation
test_api_structure() {
    run_test "Validating API structure"

    # Check if API modules exist
    if [ -f "src/api/mod.rs" ]; then
        success "API module structure exists"
    else
        warning "API module not found"
    fi

    if [ -f "src/api/user_api.rs" ]; then
        success "User API module exists"
    else
        warning "User API module not found"
    fi

    if [ -f "src/models/user.rs" ]; then
        success "User model exists"
    else
        warning "User model not found"
    fi

    if [ -f "src/services/user_service.rs" ]; then
        success "User service exists"
    else
        warning "User service not found"
    fi
}

# Generate test report
generate_report() {
    echo -e "\n${PURPLE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${PURPLE}                        TEST REPORT                            ${NC}"
    echo -e "${PURPLE}═══════════════════════════════════════════════════════════════${NC}"

    echo -e "\n${BLUE}Summary:${NC}"
    echo -e "  Total tests: ${TOTAL_TESTS}"
    echo -e "  ${GREEN}Passed: ${PASSED_TESTS}${NC}"
    echo -e "  ${RED}Failed: ${FAILED_TESTS}${NC}"

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "\n${GREEN}🎉 All tests passed! Your multi-user system is ready for development.${NC}"
        OVERALL_STATUS=0
    else
        echo -e "\n${RED}❌ Some tests failed. Please review the errors above.${NC}"
        OVERALL_STATUS=1
    fi

    echo -e "\n${BLUE}Next steps:${NC}"
    echo -e "  1. Run the full end-to-end tests with: ./test-multiuser.sh"
    echo -e "  2. Start the development server with: cargo run"
    echo -e "  3. Access the API at: http://localhost:8080/api/health"

    echo -e "\n${PURPLE}═══════════════════════════════════════════════════════════════${NC}"
}

# Cleanup function
cleanup() {
    rm -f test_output.tmp
}

# Signal handlers
trap cleanup EXIT

# Main execution
main() {
    echo -e "${PURPLE}StepheyBot Music - Simple Multi-User API Test${NC}"
    echo -e "${PURPLE}Testing compilation and basic functionality...${NC}\n"

    # Run all tests
    test_rust_toolchain || exit 1
    test_dependencies || exit 1
    test_compilation || exit 1
    test_unit_tests
    test_documentation
    test_formatting
    test_lints
    test_features
    test_binary || exit 1
    test_api_structure

    # Generate report
    generate_report

    exit $OVERALL_STATUS
}

# Run main function
main "$@"
