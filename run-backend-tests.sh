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

# Check if PostgreSQL is running
if ! pg_isready -q 2>/dev/null; then
    echo "${RED}❌ PostgreSQL is not running${NC}"
    echo "Please start PostgreSQL before running tests."
    echo ""
    echo "On macOS with Homebrew:"
    echo "  brew services start postgresql"
    echo ""
    exit 1
fi

echo "✅ PostgreSQL is running"
echo ""

# Setup test database
echo "📊 Setting up test database..."
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/atomik_wallet_test"

# Try with current user if postgres user doesn't exist
if ! psql -U postgres -c "SELECT 1" >/dev/null 2>&1; then
    export DATABASE_URL="postgresql://$(whoami)@localhost:5432/atomik_wallet_test"
    echo "Using current user for database connection"
fi

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
echo "Note: Test database 'atomik_wallet_test' is kept for SQLx compile-time checks."
echo "      Anchor program tests require 'anchor test' command."
echo "      Install Anchor CLI to run full Solana program tests."
echo ""
echo "Next steps:"
echo "  1. Deploy to Solana devnet: cd programs/vault && anchor deploy"
echo "  2. Start backend: cd services/backend && cargo run"
echo "  3. Start processor: cd services/processor && cargo run"
echo "  4. Start frontend: cd apps/frontend && pnpm dev"
