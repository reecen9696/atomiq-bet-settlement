#!/bin/bash
set -e

echo "Building Solana program with Rust 1.92.0..."
echo "Cargo version: $(cargo --version)"
echo "Rustc version: $(rustc --version)"

cd /Users/reece/code/projects/atomik-wallet

# Build with anchor
echo "Starting anchor build..."
anchor build

# Check result
if [ -f "target/deploy/vault.so" ]; then
    echo "✅ Build successful!"
    ls -lh target/deploy/vault.so
    exit 0
else
    echo "❌ Build failed - no .so file"
    exit 1
fi
