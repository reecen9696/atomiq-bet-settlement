#!/bin/bash

set -e

echo "🧪 Atomik Wallet - Running Backend Tests"
echo "========================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Backend Unit Tests
echo "${YELLOW}📦 Test 1/2: Backend Unit Tests${NC}"
echo "--------------------------------"
cd services/backend

if cargo test -- --test-threads=1 --nocapture; then
    echo ""
    echo "${GREEN}✅ Backend tests passed${NC}"
else
    echo ""
    echo "${RED}❌ Backend tests failed${NC}"
    cd ../..
    exit 1
fi

cd ../..
echo ""

# Test 2: Processor Unit Tests
echo "${YELLOW}📦 Test 2/2: Processor Unit Tests${NC}"
echo "----------------------------------"
cd services/processor

if cargo test -- --nocapture; then
    echo ""
    echo "${GREEN}✅ Processor tests passed${NC}"
else
    echo ""
    echo "${RED}❌ Processor tests failed${NC}"
    cd ../..
    exit 1
fi

cd ../..
echo ""

echo "${GREEN}=========================================="
echo "🎉 All tests passed successfully!"
echo "==========================================${NC}"
echo ""
echo "Test Summary:"
echo "  ✅ Backend Unit Tests (Repository & Handlers)"
echo "  ✅ Processor Unit Tests (Batch Processing Logic)"
echo ""
echo "Note: This repo now uses Redis for persistence in the POC."
echo "      Anchor program tests require 'anchor test' in programs/vault."
echo ""
echo "Next steps:"
echo "  1. Start services: ./start-services.sh"
echo "  2. Run end-to-end smoke: ./test-system.sh"
