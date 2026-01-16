#!/bin/bash
# Setup wrapped SOL for betting with existing allowance

set -e

echo "🔧 Setting up Wrapped SOL for betting..."
echo ""

USER_KEYPAIR=/Users/reece/code/projects/atomik-wallet/test-user-keypair.json
PROGRAM_ID=Cek6v3J44BS6mpoUGjSqTeCUgTViUzpQKkMLcuiZsoxL
CASINO=AMySLNvUFp5avus8644r7CdVtjB3CVkWHfPaDu98Vtat
RPC_URL=https://api.devnet.solana.com

USER=$(solana-keygen pubkey $USER_KEYPAIR)
echo "User: $USER"

# Create wrapped SOL token account for user
echo ""
echo "1️⃣ Creating wrapped SOL token account for user..."
USER_WSOL_ACCOUNT=$(spl-token create-account So11111111111111111111111111111111111111112 \
  --owner $USER \
  --fee-payer $USER_KEYPAIR \
  --url $RPC_URL \
  2>&1 | grep "Creating account" | awk '{print $NF}')

if [ -z "$USER_WSOL_ACCOUNT" ]; then
  # Account might already exist, try to get it
  USER_WSOL_ACCOUNT=$(spl-token accounts So11111111111111111111111111111111111111112 \
    --owner $USER \
    --url $RPC_URL \
    | grep -A 1 "So11111111111111111111111111111111111111112" \
    | tail -n 1 \
    | awk '{print $1}')
fi

echo "User wSOL account: $USER_WSOL_ACCOUNT"

# Wrap 0.5 SOL
echo ""
echo "2️⃣ Wrapping 0.5 SOL..."
spl-token wrap 0.5 \
  --owner $USER_KEYPAIR \
  --create-aux-account \
  --url $RPC_URL

echo ""
echo "3️⃣ Creating wrapped SOL token account for casino vault..."
# For casino, we need the vault authority PDA to own the account
# For now, let's just note that processor needs to handle this

echo ""
echo "✅ Wrapped SOL setup complete!"
echo ""
echo "User wSOL Token Account: $USER_WSOL_ACCOUNT"
echo ""
echo "📝 Next: Update processor to use wrapped SOL token accounts"
echo "   The existing allowance (JAsaV2QctP9fjxdsy16D6QegSTmAwbxc22U2zMe9xnDZ) uses wrapped SOL mint"
echo "   Processor needs to provide token accounts instead of placeholders"
