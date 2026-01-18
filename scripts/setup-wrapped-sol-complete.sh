#!/bin/bash
set -e

echo "🔧 Setting up Wrapped SOL..."

USER_KEYPAIR=../keys/test-user-keypair.json
USER=$(solana-keygen pubkey $USER_KEYPAIR)

echo "User: $USER"
echo ""

# Create/get wrapped SOL account for user
echo "1️⃣ Creating wrapped SOL token account..."
spl-token create-account So11111111111111111111111111111111111111112 \
  --owner $USER_KEYPAIR \
  --fee-payer $USER_KEYPAIR \
  --url devnet \
  || echo "Account might already exist"

# Wrap 0.3 SOL
echo ""
echo "2️⃣ Wrapping 0.3 SOL..."
spl-token wrap 0.3 \
  --owner $USER_KEYPAIR \
  --fee-payer $USER_KEYPAIR \
  --url devnet

echo ""
echo "3️⃣ Checking balance..."
spl-token balance So11111111111111111111111111111111111111112 \
  --owner $USER \
  --url devnet

echo ""
echo "✅ Wrapped SOL setup complete!"
echo "   The existing allowance uses wrapped SOL and should now work"
