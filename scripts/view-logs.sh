#!/bin/bash

# Check if logs exist
if [ ! -f logs/backend.log ] && [ ! -f logs/processor.log ]; then
    echo "❌ No log files found. Start services first with: ./start-services.sh"
    exit 1
fi

echo "📋 Viewing Atomik Wallet Logs"
echo "================================"
echo "Press Ctrl+C to exit"
echo ""

# Use multitail if available, otherwise use tail with labels
if command -v multitail &> /dev/null; then
    multitail -l "tail -f logs/backend.log" -l "tail -f logs/processor.log"
else
    # Fallback: Use tail with grep coloring
    tail -f logs/backend.log logs/processor.log | while read line; do
        if [[ $line == ==>* ]]; then
            # File separator from tail -f
            echo -e "\033[1;36m$line\033[0m"
        elif [[ $line == *ERROR* ]] || [[ $line == *error* ]]; then
            echo -e "\033[1;31m$line\033[0m"
        elif [[ $line == *WARN* ]] || [[ $line == *warn* ]]; then
            echo -e "\033[1;33m$line\033[0m"
        elif [[ $line == *INFO* ]] || [[ $line == *info* ]]; then
            echo -e "\033[1;32m$line\033[0m"
        elif [[ $line == *Transaction\ confirmed* ]] || [[ $line == *confirmed* ]]; then
            echo -e "\033[1;35m$line\033[0m"
        else
            echo "$line"
        fi
    done
fi
