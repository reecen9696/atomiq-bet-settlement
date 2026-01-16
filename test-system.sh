#!/bin/bash

set -euo pipefail

BACKEND_URL=${BACKEND_URL:-http://localhost:3001}
PROCESSOR_METRICS_URL=${PROCESSOR_METRICS_URL:-http://localhost:9091}
USER_WALLET=${USER_WALLET:-TEST_WALLET_2}

echo "🧪 Testing Atomik Wallet System (Redis-backed POC)"
echo "=================================================="
echo ""

echo "Checking Redis..."
if command -v redis-cli >/dev/null 2>&1; then
    if redis-cli ping >/dev/null 2>&1; then
        echo "✅ Redis is reachable"
    else
        echo "❌ Redis is not reachable (redis-cli ping failed)"
        echo "Start Redis and re-run this script."
        exit 1
    fi
else
    if nc -z 127.0.0.1 6379 >/dev/null 2>&1; then
        echo "✅ Redis port 6379 is open"
    else
        echo "❌ Redis is not reachable (no redis-cli, port 6379 closed)"
        echo "Install redis-cli or start Redis on localhost:6379."
        exit 1
    fi
fi

echo ""
echo "Checking services..."
if curl -fsS "$BACKEND_URL/health" >/dev/null 2>&1; then
    echo "✅ Backend is running at $BACKEND_URL"
else
    echo "❌ Backend is NOT running"
    echo "Start with: ./start-services.sh"
    exit 1
fi

echo ""
echo "Creating test bets..."

create_bet() {
    local stake_amount=$1
    local choice=$2
    curl -fsS -X POST "$BACKEND_URL/api/bets" \
        -H "Content-Type: application/json" \
        -d "{\"user_wallet\":\"$USER_WALLET\",\"stake_amount\":$stake_amount,\"stake_token\":\"SOL\",\"choice\":\"$choice\"}"
}

extract_bet_id() {
    if command -v jq >/dev/null 2>&1; then
        jq -r '.bet.bet_id'
    else
        python3 -c 'import sys, json; print(json.load(sys.stdin)["bet"]["bet_id"])'
    fi
}

BET_IDS=()
for i in 1 2 3; do
    STAKE=$((100000000 + i * 10000000))
    CHOICE="heads"
    RESPONSE=$(create_bet "$STAKE" "$CHOICE")
    BET_ID=$(echo "$RESPONSE" | extract_bet_id)
    BET_IDS+=("$BET_ID")
    echo "✅ Created bet: $BET_ID"
    sleep 0.5
done

echo ""
echo "Waiting for processor to claim and settle bets..."

wait_for_completed() {
    local bet_id=$1
    local attempts=30
    local delay=1

    for ((i=1; i<=attempts; i++)); do
        STATUS_JSON=$(curl -fsS "$BACKEND_URL/api/bets/$bet_id")
        STATUS=$(echo "$STATUS_JSON" | (command -v jq >/dev/null 2>&1 && jq -r '.bet.status // .status' || python3 -c 'import sys, json; j=json.load(sys.stdin); print((j.get("bet") or j).get("status"))'))
        if [[ "$STATUS" == "completed" ]]; then
            echo "✅ Bet $bet_id completed"
            return 0
        fi
        if [[ "$STATUS" == "failed" || "$STATUS" == "failed_retryable" ]]; then
            echo "❌ Bet $bet_id failed:"
            echo "$STATUS_JSON" | (command -v jq >/dev/null 2>&1 && jq '.' || cat)
            return 1
        fi
        sleep "$delay"
    done

    echo "❌ Timed out waiting for bet $bet_id to complete"
    curl -fsS "$BACKEND_URL/api/bets/$bet_id" | (command -v jq >/dev/null 2>&1 && jq '.' || cat)
    return 1
}

for bet_id in "${BET_IDS[@]}"; do
    wait_for_completed "$bet_id"
done

echo ""
echo "Checking pending bets endpoint (should typically be empty)..."
curl -fsS "$BACKEND_URL/api/external/bets/pending?limit=10&processor_id=test-script" | (command -v jq >/dev/null 2>&1 && jq '.' || cat)

echo ""
echo "Checking processor metrics (best-effort)..."
if METRICS_OUT=$(curl -fsS "$PROCESSOR_METRICS_URL/metrics" 2>/dev/null); then
    echo "$METRICS_OUT" | grep -E "pending_bets_fetched|batch_processing_duration_seconds|worker_errors_total|worker_circuit_breaker_open_total" | head -20 || echo "(metrics reachable but no matching lines yet)"
else
    echo "(metrics not reachable on $PROCESSOR_METRICS_URL)"
fi

echo ""
echo "✅ End-to-end test complete"
echo "- Logs: ./view-logs.sh"
