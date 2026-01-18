#!/bin/bash

set -euo pipefail

echo "🧪 Atomik Wallet - Running Full Test Suite (Redis-backed POC)"
echo "============================================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Anchor Program Tests (optional)
echo "${YELLOW}📦 Test 1/3: Anchor Program Tests (optional)${NC}"
echo "---------------------------------------------"
if command -v anchor >/dev/null 2>&1; then
    set +e

    echo "Installing Anchor test dependencies (best-effort)..."
    if command -v pnpm >/dev/null 2>&1; then
        # Avoid full workspace install (can fail if other workspace packages are misconfigured)
        pnpm --dir programs/vault install --ignore-workspace
    else
        echo "${YELLOW}⚠️  pnpm not found; skipping JS deps install${NC}"
    fi

    cd programs/vault

    echo "Building Anchor program..."
    if anchor build; then
        echo "Running Anchor tests..."
        anchor test --skip-local-validator
    else
        echo "${YELLOW}⚠️  Anchor build failed${NC}"
    fi

    if [ $? -eq 0 ]; then
        echo "${GREEN}✅ Anchor tests/build succeeded${NC}"
    else
        echo "${YELLOW}⚠️  Anchor tests/build failed (non-blocking for backend/processor POC)${NC}"
    fi

    cd ../..
    set -e
else
    echo "${YELLOW}⚠️  Anchor CLI not found; skipping program tests${NC}"
fi

echo ""

# Test 2: Backend Unit Tests
echo "${YELLOW}📦 Test 2/3: Backend Unit Tests${NC}"
echo "--------------------------------"
cd services/backend

if cargo test -- --test-threads=1; then
    echo "${GREEN}✅ Backend tests passed${NC}"
else
    echo "${RED}❌ Backend tests failed${NC}"
    exit 1
fi

cd ../..
echo ""

# Test 3: Processor Unit Tests
echo "${YELLOW}📦 Test 3/3: Processor Unit Tests${NC}"
echo "----------------------------------"
cd services/processor

if cargo test; then
    echo "${GREEN}✅ Processor tests passed${NC}"
else
    echo "${RED}❌ Processor tests failed${NC}"
    exit 1
fi

cd ../..
echo ""

echo "${GREEN}=========================================="
echo "🎉 Test suite finished"
echo "==========================================${NC}"
echo ""
echo "Next steps:"
echo "  1. Start services: ./start-services.sh"
echo "  2. Run end-to-end smoke: ./test-system.sh"
echo "  3. (Optional) Program tests: cd programs/vault && anchor test"
