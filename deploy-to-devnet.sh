#!/bin/bash

# Quick Deploy Script for Atomik Wallet

set -e

echo "🚀 Atomik Wallet - Blockchain Deployment"
echo "========================================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if Anchor is installed
if ! command -v anchor &> /dev/null; then
    echo -e "${YELLOW}⚠️  Anchor CLI not found${NC}"
    echo "Installing Anchor..."
    cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
    avm install latest
    avm use latest
    echo -e "${GREEN}✅ Anchor installed${NC}"
else
    echo -e "${GREEN}✅ Anchor CLI found: $(anchor --version)${NC}"
fi

echo ""
echo "${YELLOW}📦 Step 1: Build Anchor Program${NC}"
cd programs/vault
anchor build
echo -e "${GREEN}✅ Program built${NC}"
echo ""

echo "${YELLOW}🔑 Step 2: Get Program ID${NC}"
PROGRAM_ID=$(anchor keys list | grep vault | awk '{print $2}')
echo "Program ID: $PROGRAM_ID"
echo ""

echo "${YELLOW}🌐 Step 3: Deploy to Devnet${NC}"
echo "This will deploy the program to Solana devnet..."
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Deployment cancelled"
    exit 1
fi

anchor deploy --provider.cluster devnet
echo -e "${GREEN}✅ Program deployed to devnet${NC}"
echo ""

echo "${YELLOW}💰 Step 4: Fund Processor Keypair${NC}"
cd ../..
PROCESSOR_PUBKEY=$(solana-keygen pubkey test-keypair.json)
echo "Processor pubkey: $PROCESSOR_PUBKEY"

echo "Requesting airdrop..."
solana airdrop 2 $PROCESSOR_PUBKEY --url devnet || echo "Airdrop may have failed (rate limit), try manually"
echo ""

echo "${YELLOW}📝 Step 5: Update Configuration Files${NC}"
echo "Updating .env files with program ID: $PROGRAM_ID"

# Update backend .env
if [ -f "services/backend/.env" ]; then
    sed -i.bak "s/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=$PROGRAM_ID/" services/backend/.env
    echo "✅ Updated services/backend/.env"
fi

# Update processor .env
if [ -f "services/processor/.env" ]; then
    sed -i.bak "s/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=$PROGRAM_ID/" services/processor/.env
    echo "✅ Updated services/processor/.env"
fi

# Update frontend .env
if [ -f "apps/frontend/.env.local" ]; then
    sed -i.bak "s/NEXT_PUBLIC_VAULT_PROGRAM_ID=.*/NEXT_PUBLIC_VAULT_PROGRAM_ID=$PROGRAM_ID/" apps/frontend/.env.local
    echo "✅ Updated apps/frontend/.env.local"
else
    echo "NEXT_PUBLIC_VAULT_PROGRAM_ID=$PROGRAM_ID" > apps/frontend/.env.local
    echo "✅ Created apps/frontend/.env.local"
fi

echo ""
echo -e "${GREEN}=========================================="
echo "🎉 Deployment Complete!"
echo "==========================================${NC}"
echo ""
echo "Program ID: $PROGRAM_ID"
echo "Processor: $PROCESSOR_PUBKEY"
echo "Cluster: Devnet"
echo ""
echo "View on Solana Explorer:"
echo "https://explorer.solana.com/address/$PROGRAM_ID?cluster=devnet"
echo ""
echo "${YELLOW}Next Steps:${NC}"
echo "1. Update processor code to use real transactions"
echo "2. Generate TypeScript IDL for frontend:"
echo "   cd programs/vault && anchor idl parse -f src/lib.rs -o ../../apps/frontend/src/idl/vault.json"
echo "3. Implement VaultSDK in frontend"
echo "4. Test full flow: deposit → approve → bet → process"
echo ""
echo "See BLOCKCHAIN_INTEGRATION.md for detailed instructions"
