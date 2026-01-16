#!/bin/bash
set -e

echo "🔨 Building Solana Vault Program with Anchor 0.29.0"
echo "=================================================="

cd /Users/reece/code/projects/atomik-wallet

# Use Anchor 0.29.0
echo "📦 Switching to Anchor 0.29.0..."
avm use 0.29.0 2>/dev/null || (avm install 0.29.0 && avm use 0.29.0)
echo "✅ Anchor version: $(anchor --version)"

# Clean everything
echo "🧹 Cleaning build artifacts..."
cargo clean
rm -rf programs/vault/target
rm -f programs/vault/Cargo.lock

# Build the program
echo "🔨 Building Anchor program (this takes 2-3 minutes)..."
anchor build

# Check result
if [ -f "target/deploy/vault.so" ]; then
    echo ""
    echo "✅ BUILD SUCCESSFUL!"
    echo "=================="
    ls -lh target/deploy/vault.so
    
    PROGRAM_ID=$(solana address -k target/deploy/vault-keypair.json 2>/dev/null || echo "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS")
    
    echo ""
    echo "📋 Next Steps:"
    echo "1. Deploy: solana program deploy target/deploy/vault.so"
    echo "2. Update VAULT_PROGRAM_ID in .env files with: $PROGRAM_ID"
    echo "3. Run: ./deploy-to-devnet.sh"
else
    echo "❌ Build failed - no .so file"
    exit 1
fi
