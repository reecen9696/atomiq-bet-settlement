#!/bin/bash

set -e

echo "🧪 Atomik Wallet - Running Full Test Suite"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if test database exists
echo "📊 Setting up test database..."
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/atomik_wallet_test"

# Drop and recreate test database
dropdb atomik_wallet_test 2>/dev/null || true
createdb atomik_wallet_test

echo "✅ Test database created"
echo ""

# Run backend database migrations on test DB
echo "🔄 Running database migrations..."
cd services/backend
cargo sqlx migrate run --database-url $DATABASE_URL
cd ../..
echo "✅ Migrations completed"
echo ""

# Test 1: Anchor Program Tests
echo "${YELLOW}📦 Test 1/3: Anchor Program Tests${NC}"
echo "-----------------------------------"
cd programs/vault

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing Anchor test dependencies..."
    npm install
fi

# Build program
echo "Building Anchor program..."
anchor build

# Run tests
if anchor test --skip-local-validator; then
    echo "${GREEN}✅ Anchor tests passed${NC}"
else
    echo "${RED}❌ Anchor tests failed${NC}"
    exit 1
fi

cd ../..
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

# Cleanup
echo "🧹 Cleaning up test database..."
dropdb atomik_wallet_test 2>/dev/null || true

echo ""
echo "${GREEN}=========================================="
echo "🎉 All tests passed successfully!"
echo "==========================================${NC}"
echo ""
echo "Test Summary:"
echo "  ✅ Anchor Program Tests"
echo "  ✅ Backend Unit Tests"
echo "  ✅ Processor Unit Tests"
echo ""
echo "Next steps:"
echo "  1. Deploy to Solana devnet: cd programs/vault && anchor deploy"
echo "  2. Start backend: cd services/backend && cargo run"
echo "  3. Start processor: cd services/processor && cargo run"
echo "  4. Start frontend: cd apps/frontend && pnpm dev"
