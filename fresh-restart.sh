#!/bin/bash
# Fresh restart - clean database and restart all services

set -e

echo "🔄 Fresh Restart - Cleaning and Restarting All Services"
echo "========================================================"

# 1. Stop processor
echo ""
echo "1️⃣  Stopping processor..."
pkill -9 -f "target/release/processor" 2>/dev/null && echo "  ✅ Processor stopped" || echo "  ℹ️  No processor running"

# 2. Stop blockchain API
echo ""
echo "2️⃣  Stopping blockchain API..."
cd /Users/reece/code/projects/atomik/blockchain
pkill -9 -f "api-finalized" 2>/dev/null && echo "  ✅ API stopped" || echo "  ℹ️  No API running"

sleep 2

# 3. Clear blockchain database
echo ""
echo "3️⃣  Clearing blockchain database..."
rm -rf /Users/reece/code/projects/atomik/blockchain/DB/blockchain_data/* 2>/dev/null || true
echo "  ✅ Database cleared"

# 4. Start blockchain API
echo ""
echo "4️⃣  Starting blockchain API..."
cd /Users/reece/code/projects/atomik/blockchain
nohup cargo run --release --bin api-finalized -- --host 0.0.0.0 --port 8080 --db-path ./DB/blockchain_data > logs/blockchain-api.log 2>&1 &
API_PID=$!
echo "  ✅ Blockchain API started (PID: $API_PID)"

# Wait for API to initialize
echo "  ⏳ Waiting for API to initialize..."
sleep 5

# 5. Verify API is running
curl -s http://localhost:8080/health > /dev/null 2>&1 && echo "  ✅ API is responding" || echo "  ⚠️  API not responding yet"

# 6. Start processor with new config
echo ""
echo "5️⃣  Starting processor with PROCESSOR_MAX_BETS_PER_TX=6..."
cd /Users/reece/code/projects/atomik/transaction-processor

# Clear old logs
> logs/processor.log

# Start processor
./start-processor.sh > logs/processor.log 2>&1 &
PROCESSOR_PID=$!
echo "  ✅ Processor started (PID: $PROCESSOR_PID)"

# Wait for startup
sleep 3

# Verify processor is running
if ps -p $PROCESSOR_PID > /dev/null 2>&1; then
    echo "  ✅ Processor is running"
else
    echo "  ❌ Processor failed to start"
    exit 1
fi

# 7. Verify configuration
echo ""
echo "6️⃣  Verifying configuration..."
sleep 2
if grep -q "max_bets_per_tx.*:6" logs/processor.log 2>/dev/null; then
    echo "  ✅ Processor using max_bets_per_tx=6"
else
    echo "  ⚠️  Check processor logs to verify config"
fi

echo ""
echo "=============================================="
echo "✅ Fresh restart complete!"
echo ""
echo "📊 Services:"
echo "  Blockchain API: http://localhost:8080 (PID: $API_PID)"
echo "  Processor:      PID: $PROCESSOR_PID"
echo ""
echo "📋 Useful commands:"
echo "  View API logs:       tail -f /Users/reece/code/projects/atomik/blockchain/logs/blockchain-api.log"
echo "  View processor logs: tail -f /Users/reece/code/projects/atomik/transaction-processor/logs/processor.log"
echo "  Check API health:    curl http://localhost:8080/health"
echo ""
echo "⚙️  Configuration:"
echo "  PROCESSOR_MAX_BETS_PER_TX=6 (prevents transaction size errors)"
echo ""
