#!/bin/bash

set -e

echo "🔗 On-Chain Integration Test"
echo "============================="
echo ""

# Configuration
USER_KEYPAIR="test-user-keypair.json"
PROCESSOR_KEYPAIR="test-keypair.json"
PROGRAM_ID="Cek6v3J44BS6mpoUGjSqTeCUgTViUzpQKkMLcuiZsoxL"
RPC_URL="https://api.devnet.solana.com"

# Get wallet addresses
USER_WALLET=$(solana-keygen pubkey $USER_KEYPAIR)
PROCESSOR_WALLET=$(solana-keygen pubkey $PROCESSOR_KEYPAIR)

echo "👤 Test User Wallet: $USER_WALLET"
echo "🔑 Processor Wallet: $PROCESSOR_WALLET"
echo "📝 Program ID: $PROGRAM_ID"
echo ""

# Check balances
echo "💰 Checking balances..."
USER_BALANCE=$(solana balance $USER_WALLET --url $RPC_URL)
PROCESSOR_BALANCE=$(solana balance $PROCESSOR_WALLET --url $RPC_URL)

echo "  User: $USER_BALANCE"
echo "  Processor: $PROCESSOR_BALANCE"
echo ""

# Check if services are running
if ! curl -s http://localhost:3001/health > /dev/null 2>&1; then
    echo "❌ Backend is NOT running!"
    echo "   Start with: ./start-services.sh"
    exit 1
fi

echo "✅ Backend is running"
echo ""

# Create bets with real wallet address
echo "📊 Creating test bets with real wallet..."

for i in 1 2 3; do
    AMOUNT=$((100000000 + i * 50000000))
    CHOICE=$([ $((i % 2)) -eq 0 ] && echo "tails" || echo "heads")
    
    RESPONSE=$(curl -s -X POST http://localhost:3001/api/bets \
      -H "Content-Type: application/json" \
      -d "{
        \"user_wallet\": \"$USER_WALLET\",
        \"stake_amount\": $AMOUNT,
        \"stake_token\": \"SOL\",
        \"choice\": \"$CHOICE\"
      }")
    
    BET_ID=$(echo $RESPONSE | jq -r '.bet.bet_id // "null"')
    
    if [ "$BET_ID" = "null" ]; then
        echo "❌ Failed to create bet $i"
        echo "   Response: $RESPONSE"
    else
        echo "✅ Created bet $i: $BET_ID ($AMOUNT lamports, $CHOICE)"
    fi
    
    sleep 1
done

echo ""
echo "⏳ Waiting for processor to pick up bets (15 seconds)..."
sleep 15

echo ""
echo "📋 Checking batch status..."
curl -s http://localhost:3001/api/external/bets/pending?limit=10 | jq '.[] | {bet_id, status, stake_amount, user_wallet}' || echo "[]"

echo ""
echo "📊 Processor logs (last 30 lines with batch info):"
tail -30 logs/processor.log | grep -E "(batch|Transaction|confirmed|failed)" || echo "No batch processing logs yet"

echo ""
echo "✅ Test complete!"
echo ""
echo "To view full logs:"
echo "  ./view-logs.sh"
echo ""
echo "To check on Solana Explorer:"
echo "  https://explorer.solana.com/address/$USER_WALLET?cluster=devnet"
