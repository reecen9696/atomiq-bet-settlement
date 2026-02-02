#!/bin/bash
# Atomik Wallet - Stop Services

echo "🛑 Stopping Atomik Wallet Services..."

# Kill processes by PID if pid files exist
if [[ -f logs/backend.pid ]]; then
    BACKEND_PID=$(cat logs/backend.pid)
    if kill -0 $BACKEND_PID 2>/dev/null; then
        kill $BACKEND_PID
        echo "✅ Backend stopped (PID: $BACKEND_PID)"
    else
        echo "⚠️  Backend not running"
    fi
    rm -f logs/backend.pid
else
    echo "⚠️  Backend PID file not found"
fi

if [[ -f logs/processor.pid ]]; then
    PROCESSOR_PID=$(cat logs/processor.pid)
    if kill -0 $PROCESSOR_PID 2>/dev/null; then
        kill $PROCESSOR_PID
        echo "✅ Processor stopped (PID: $PROCESSOR_PID)"
    else
        echo "⚠️  Processor not running"
    fi
    rm -f logs/processor.pid
else
    if pgrep -f "/Users/reece/code/projects/atomik/backend/transaction-processor/target/release/processor" > /dev/null; then
        pkill -f "/Users/reece/code/projects/atomik/backend/transaction-processor/target/release/processor" 2>/dev/null && echo "✅ Processor stopped"
    else
        echo "⚠️  Processor PID file not found"
    fi
fi

# Cleanup any remaining processes
pkill -f "cargo run.*backend" 2>/dev/null && echo "✅ Cleaned up backend processes"
pkill -f "cargo run.*processor" 2>/dev/null && echo "✅ Cleaned up processor processes"
pkill -f "/Users/reece/code/projects/atomik/backend/transaction-processor/target/release/backend" 2>/dev/null && echo "✅ Cleaned up backend binaries"
pkill -f "/Users/reece/code/projects/atomik/backend/transaction-processor/target/release/processor" 2>/dev/null && echo "✅ Cleaned up processor binaries"

echo ""
echo "✅ All services stopped"