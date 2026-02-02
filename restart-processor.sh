#!/bin/bash
# Restart processor with new configuration

set -e

echo "🔄 Restarting Transaction Processor"
echo "====================================="

# Kill any running processor instances
echo "Stopping current processor..."
pkill -9 -f "target/release/processor" 2>/dev/null && echo "  ✅ Stopped existing processor" || echo "  ℹ️  No processor running"

# Wait for process to fully terminate
sleep 1

# Start processor using the start-processor.sh script
echo ""
echo "Starting processor with updated config..."
cd "$(dirname "$0")"

# Run the processor binary directly in the background
./start-processor.sh > logs/processor.log 2>&1 &
PROCESSOR_PID=$!

echo "  ✅ Processor started (PID: $PROCESSOR_PID)"
echo ""

# Wait for startup
sleep 2

# Check if it's still running
if ps -p $PROCESSOR_PID > /dev/null 2>&1; then
    echo "✅ Processor is running"
    echo ""
    echo "📊 Logs: tail -f logs/processor.log"
    echo "🛑 Stop: pkill -f 'target/release/processor'"
else
    echo "❌ Processor failed to start. Check logs/processor.log"
    exit 1
fi
