#!/bin/bash

echo "🧪 Testing Atomik Wallet System"
echo "================================"
echo ""

# Check if services are running
echo "Checking services..."
if curl -s http://localhost:3001/health > /dev/null 2>&1; then
    echo "✅ Backend is running"
else
    echo "❌ Backend is NOT running - start with: cd services/backend && cargo run"
    exit 1
fi

echo ""
echo "Creating test bets..."

# Create 3 test bets
for i in {1..3}; do
    RESPONSE=$(curl -s -X POST http://localhost:3001/api/bets \
      -H "Content-Type: application/json" \
      -d "{\"stake_amount\": $((1000000 + i * 100000)), \"stake_token\": \"SOL\", \"choice\": \"heads\"}")
    
    BET_ID=$(echo $RESPONSE | jq -r '.bet.bet_id')
    echo "✅ Created bet: $BET_ID"
    sleep 1
done

echo ""
echo "Waiting for processor to pick up bets..."
sleep 3

echo ""
echo "Checking pending bets..."
curl -s http://localhost:3001/api/external/bets/pending?limit=10 | jq '.'

echo ""
echo "Checking processor logs..."
echo "(Look for 'Batch created' or 'Processing batch' messages in processor terminal)"

echo ""
echo "✅ Test complete!"
echo ""
echo "To check results:"
echo "  - Backend logs: Check terminal where 'cargo run' is running"
echo "  - Database: psql atomik_wallet_dev -c 'SELECT bet_id, status, solana_tx_id FROM bets ORDER BY created_at DESC LIMIT 5;'"
echo "  - Processor metrics: curl http://localhost:9091/metrics | grep bets"
