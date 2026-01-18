#!/bin/bash

# Create logs directory
mkdir -p logs

echo "🚀 Starting Atomik Wallet Services..."
echo ""
echo "Logs will be written to:"
echo "  - logs/backend.log"
echo "  - logs/processor.log"
echo ""

# Start backend
echo "Starting backend..."
cd services/backend
RUST_LOG=backend=info cargo run >> ../../logs/backend.log 2>&1 &
BACKEND_PID=$!
cd ../..
echo "✅ Backend started (PID: $BACKEND_PID)"

# Wait a moment for backend to initialize
sleep 2

# Start processor
echo "Starting processor..."
cd services/processor
RUST_LOG=processor=info cargo run >> ../../logs/processor.log 2>&1 &
PROCESSOR_PID=$!
cd ../..
echo "✅ Processor started (PID: $PROCESSOR_PID)"

# Save PIDs
echo $BACKEND_PID > logs/backend.pid
echo $PROCESSOR_PID > logs/processor.pid

echo ""
echo "✅ Services started!"
echo ""
echo "To view logs in real-time:"
echo "  ./view-logs.sh"
echo ""
echo "To stop services:"
echo "  ./stop-services.sh"
echo ""
echo "To run tests:"
echo "  ./test-system.sh"
