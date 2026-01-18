#!/bin/bash
# Real CLI Bet Test - Places actual bet through backend/processor

set -e

BACKEND_URL="http://localhost:3001"
PROGRAM_ID="HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP"

echo "🎲 Real Bet Test - CLI"
echo "======================================"
echo "Program ID: $PROGRAM_ID"
echo "Backend: $BACKEND_URL"
echo ""

# Step 1: Check backend health
echo "1️⃣ Checking backend health..."
HEALTH=$(curl -s $BACKEND_URL/health)
echo "   $HEALTH"
STATUS=$(echo $HEALTH | jq -r '.status')
if [ "$STATUS" != "healthy" ]; then
    echo "   ❌ Backend not healthy!"
    exit 1
fi
echo "   ✅ Backend is healthy"
echo ""

# Step 2: Place bet
echo "2️⃣ Placing bet..."
echo "   Amount: 0.1 SOL (100,000,000 lamports)"
echo "   Choice: HEADS"
echo ""

BET_RESPONSE=$(curl -s -X POST $BACKEND_URL/api/bets \
  -H "Content-Type: application/json" \
  -d '{
    "stake_amount": 100000000,
    "stake_token": "SOL",
    "choice": "heads"
  }')

echo "   Response:"
echo "$BET_RESPONSE" | jq '.'
echo ""

# Extract bet ID
BET_ID=$(echo $BET_RESPONSE | jq -r '.bet.bet_id // .bet_id // empty')

if [ -z "$BET_ID" ] || [ "$BET_ID" == "null" ]; then
    echo "   ❌ Failed to create bet!"
    echo "   Response: $BET_RESPONSE"
    exit 1
fi

echo "   ✅ Bet created: $BET_ID"
echo ""

# Step 3: Monitor bet status
echo "3️⃣ Monitoring bet status..."
echo "   (Processor runs every 30 seconds)"
echo ""

MAX_ATTEMPTS=20
ATTEMPT=0

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    ATTEMPT=$((ATTEMPT + 1))
    sleep 3
    
    BET_STATUS=$(curl -s $BACKEND_URL/api/bets/$BET_ID)
    STATUS=$(echo $BET_STATUS | jq -r '.status // .bet.status // "unknown"')
    
    printf "   [%2d/%2d] Status: %-12s" $ATTEMPT $MAX_ATTEMPTS "$STATUS"
    
    if [ "$STATUS" == "completed" ] || [ "$STATUS" == "settled" ]; then
        echo " ✅"
        echo ""
        echo "🎉 Bet completed!"
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "$BET_STATUS" | jq '{
          bet_id: .bet_id // .bet.bet_id,
          status: .status // .bet.status,
          stake_amount: .stake_amount // .bet.stake_amount,
          choice: .choice // .bet.choice,
          result: .result // .bet.result,
          won: .won // .bet.won,
          payout_amount: .payout_amount // .bet.payout_amount,
          solana_tx_id: .solana_tx_id // .bet.solana_tx_id,
          payout_tx_id: .payout_tx_id // .bet.payout_tx_id,
          created_at: .created_at // .bet.created_at,
          completed_at: .completed_at // .bet.completed_at
        }'
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""
        
        # Check for Solana transaction
        TX_ID=$(echo $BET_STATUS | jq -r '.solana_tx_id // .bet.solana_tx_id // empty')
        if [ ! -z "$TX_ID" ] && [ "$TX_ID" != "null" ]; then
            echo "🔗 View on Solana Explorer:"
            echo "   https://explorer.solana.com/tx/$TX_ID?cluster=devnet"
            echo ""
        fi
        
        # Check if won
        WON=$(echo $BET_STATUS | jq -r '.won // .bet.won // false')
        PAYOUT=$(echo $BET_STATUS | jq -r '.payout_amount // .bet.payout_amount // 0')
        
        if [ "$WON" == "true" ]; then
            echo "🏆 YOU WON! Payout: $PAYOUT lamports"
        else
            echo "😔 You lost. Better luck next time!"
        fi
        echo ""
        
        exit 0
    elif [ "$STATUS" == "failed" ]; then
        echo " ❌"
        echo ""
        echo "❌ Bet failed!"
        echo "$BET_STATUS" | jq '.'
        exit 1
    else
        echo ""
    fi
done

echo ""
echo "⏳ Bet still processing after $MAX_ATTEMPTS attempts."
echo "   Check status manually with:"
echo "   curl $BACKEND_URL/api/bets/$BET_ID | jq"
echo ""
