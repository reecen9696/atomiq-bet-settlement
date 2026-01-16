#!/bin/bash

echo "🚀 Atomik Bet Test UI Setup"
echo ""

# Check if .env exists
if [ ! -f .env ]; then
    echo "⚠️  No .env file found"
    echo "Creating .env from template..."
    cat > .env << EOF
# Privy Configuration
# Get your App ID from https://console.privy.io
VITE_PRIVY_APP_ID=your_privy_app_id_here

# API Configuration
VITE_API_BASE_URL=http://localhost:3001

# Solana Configuration
VITE_SOLANA_RPC_URL=https://api.devnet.solana.com
VITE_VAULT_PROGRAM_ID=HoWjrEKiWKjEvqtdMDAHS9PEwkHQbVp2t6vYuDv3mdi4
VITE_SOLANA_NETWORK=devnet
EOF
    echo "✅ .env file created"
    echo ""
    echo "⚠️  IMPORTANT: Update .env with your Privy App ID!"
    echo "   Get one at: https://console.privy.io"
    echo ""
else
    echo "✅ .env file exists"
    echo ""
fi

# Check if node_modules exists
if [ ! -d node_modules ]; then
    echo "📦 Installing dependencies..."
    
    # Check for pnpm
    if command -v pnpm &> /dev/null; then
        echo "Using pnpm..."
        pnpm install
    # Check for npm
    elif command -v npm &> /dev/null; then
        echo "Using npm..."
        npm install
    else
        echo "❌ Neither pnpm nor npm found. Please install Node.js first."
        exit 1
    fi
    
    echo "✅ Dependencies installed"
else
    echo "✅ Dependencies already installed"
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Next steps:"
echo "1. Update .env with your Privy App ID from https://console.privy.io"
echo "2. Make sure your backend is running on localhost:3001"
echo "3. Run: pnpm dev (or npm run dev)"
echo ""
echo "The app will be available at http://localhost:3000"
echo ""