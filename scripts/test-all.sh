#!/bin/bash
# Comprehensive test script for genesis-preflight

set -e

echo "==============================================="
echo "Genesis Preflight - Comprehensive Test Suite"
echo "==============================================="
echo ""

# Colors for output (optional, works on Unix-like systems)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        exit 1
    fi
}

# 1. Check formatting
echo "1. Checking code formatting..."
cargo fmt --all -- --check
print_status $? "Code formatting"

# 2. Run clippy
echo ""
echo "2. Running clippy lints..."
cargo clippy --all-targets -- -D warnings
print_status $? "Clippy lints"

# 3. Build debug
echo ""
echo "3. Building debug binary..."
cargo build
print_status $? "Debug build"

# 4. Build release
echo ""
echo "4. Building release binary..."
cargo build --release
print_status $? "Release build"

# 5. Run unit tests
echo ""
echo "5. Running unit tests..."
cargo test --lib
print_status $? "Unit tests"

# 6. Run integration tests
echo ""
echo "6. Running integration tests..."
cargo test --test '*'
print_status $? "Integration tests"

# 7. Run all tests
echo ""
echo "7. Running all tests..."
cargo test --all
print_status $? "All tests"

# 8. Build documentation
echo ""
echo "8. Building documentation..."
cargo doc --no-deps
print_status $? "Documentation build"

# 9. Test binary execution
echo ""
echo "9. Testing binary execution..."
./target/release/genesis-preflight --version > /dev/null
print_status $? "Binary --version flag"

./target/release/genesis-preflight --help > /dev/null
print_status $? "Binary --help flag"

# 10. Test on fixtures
echo ""
echo "10. Testing on fixture datasets..."
./target/release/genesis-preflight scan tests/fixtures/valid_dataset --no-hash --quiet
print_status $? "Scan valid dataset"

./target/release/genesis-preflight scan tests/fixtures/invalid_dataset --no-hash --quiet
if [ $? -eq 2 ]; then
    echo -e "${GREEN}✓${NC} Scan invalid dataset (correctly detected issues)"
else
    echo -e "${RED}✗${NC} Scan invalid dataset (should have exit code 2)"
    exit 1
fi

# 11. Check for warnings in release build
echo ""
echo "11. Checking for warnings..."
WARNINGS=$(cargo build --release 2>&1 | grep -i warning | wc -l)
if [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}✓${NC} No warnings in release build"
else
    echo -e "${YELLOW}!${NC} Found $WARNINGS warnings in release build"
fi

# 12. Check binary size
echo ""
echo "12. Binary size check..."
BINARY_SIZE=$(wc -c < target/release/genesis-preflight)
BINARY_SIZE_MB=$((BINARY_SIZE / 1024 / 1024))
echo "   Binary size: ${BINARY_SIZE_MB}MB"
if [ $BINARY_SIZE_MB -lt 50 ]; then
    echo -e "${GREEN}✓${NC} Binary size acceptable"
else
    echo -e "${YELLOW}!${NC} Binary size is large (>${BINARY_SIZE_MB}MB)"
fi

echo ""
echo "==============================================="
echo -e "${GREEN}All tests passed!${NC}"
echo "==============================================="
echo ""
echo "Ready for release:"
echo "  1. cargo publish --dry-run"
echo "  2. git tag v0.1.0"
echo "  3. cargo publish"
