#!/bin/bash

echo "🛑 Stopping Atomik Wallet Services..."

# Stop backend
if [ -f logs/backend.pid ]; then
    BACKEND_PID=$(cat logs/backend.pid)
    if ps -p $BACKEND_PID > /dev/null 2>&1; then
        kill $BACKEND_PID
        echo "✅ Backend stopped (PID: $BACKEND_PID)"
    else
        echo "⚠️  Backend not running"
    fi
    rm logs/backend.pid
fi

# Stop processor
if [ -f logs/processor.pid ]; then
    PROCESSOR_PID=$(cat logs/processor.pid)
    if ps -p $PROCESSOR_PID > /dev/null 2>&1; then
        kill $PROCESSOR_PID
        echo "✅ Processor stopped (PID: $PROCESSOR_PID)"
    else
        echo "⚠️  Processor not running"
    fi
    rm logs/processor.pid
fi

# Also kill any remaining cargo run processes for these services
pkill -f "cargo run.*backend" 2>/dev/null && echo "✅ Cleaned up backend processes"
pkill -f "cargo run.*processor" 2>/dev/null && echo "✅ Cleaned up processor processes"

echo ""
echo "✅ All services stopped"
