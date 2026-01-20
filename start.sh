#!/bin/bash
# Atomik Wallet - Production Start
# Simple, single-command startup for production-ready system

set -e

echo "🚀 Starting Atomik Wallet (Production Mode)"
echo "=============================================="
echo ""
echo "📝 System Requirements:"
echo "  ✅ Redis running on localhost:6379"
echo "  ✅ Anchor program deployed to devnet"
echo "  ✅ Casino vault initialized with funds"
echo ""

# Check if Redis is running
if ! redis-cli ping > /dev/null 2>&1; then
    echo "❌ Redis is not running. Please start Redis first:"
    echo "   brew services start redis"
    exit 1
fi

echo "✅ Redis is running"
echo ""

# Create logs directory
mkdir -p logs

echo "🔧 Starting services..."

# Start backend
echo "  🔙 Starting backend..."
cd services/backend
RUST_LOG=backend=info cargo run --release >> ../../logs/backend.log 2>&1 &
BACKEND_PID=$!
cd ../..
echo "     ✅ Backend started (PID: $BACKEND_PID)"

# Wait for backend to initialize
sleep 3

# Start processor 
echo "  ⚙️  Starting processor..."
cd services/processor
RUST_LOG=processor=info cargo run --release >> ../../logs/processor.log 2>&1 &
PROCESSOR_PID=$!
cd ../..
echo "     ✅ Processor started (PID: $PROCESSOR_PID)"

# Save PIDs for cleanup
echo $BACKEND_PID > logs/backend.pid
echo $PROCESSOR_PID > logs/processor.pid

echo ""
echo "✅ Atomik Wallet is running!"
echo ""
echo "📊 Services:"
echo "  Backend API:  http://localhost:3001"
echo "  Metrics:      http://localhost:9090 (backend), http://localhost:9091 (processor)"
echo ""
echo "📋 Useful commands:"
echo "  View logs:    tail -f logs/backend.log logs/processor.log"
echo "  Health check: curl http://localhost:3001/health"
echo "  Stop system:  ./stop.sh"
echo ""
echo "🎮 To test betting:"
echo "  1. Start frontend: cd test-ui && npm run dev"
echo "  2. Visit: http://localhost:3000"
echo "  3. Connect wallet and place bets"