#!/usr/bin/env bash
# Load testing script for Atomik Wallet backend
# Requires: k6 (https://k6.io/)

set -e

BACKEND_URL="${BACKEND_URL:-http://localhost:3001}"
DURATION="${DURATION:-30s}"
VUS="${VUS:-10}" # Virtual users

echo "🧪 Running load tests..."
echo "Backend URL: $BACKEND_URL"
echo "Duration: $DURATION"
echo "Virtual Users: $VUS"
echo ""

# Check if k6 is installed
if ! command -v k6 &> /dev/null; then
    echo "❌ k6 is not installed"
    echo "Install with: brew install k6 (macOS) or visit https://k6.io/docs/get-started/installation/"
    exit 1
fi

# Run the load test
k6 run \
    --vus $VUS \
    --duration $DURATION \
    --env BACKEND_URL=$BACKEND_URL \
    "$(dirname "$0")/load-test.js"
