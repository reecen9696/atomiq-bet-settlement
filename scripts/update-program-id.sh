#!/bin/bash
set -e

NEW_PROGRAM_ID=$1

if [ -z "$NEW_PROGRAM_ID" ]; then
    echo "❌ Error: Program ID required"
    echo ""
    echo "Usage: ./scripts/update-program-id.sh <NEW_PROGRAM_ID>"
    echo ""
    echo "Example:"
    echo "  ./scripts/update-program-id.sh BtZT2B1NkEGZwNT5CS326HbdbXzggiTYSUiYmSDyhTDJ"
    exit 1
fi

echo "🔄 Updating program ID to: $NEW_PROGRAM_ID"
echo ""

# Update all env files
echo "📝 Updating environment files..."
sed -i '' "s/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=$NEW_PROGRAM_ID/" .env
sed -i '' "s/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=$NEW_PROGRAM_ID/" services/backend/.env
sed -i '' "s/VITE_VAULT_PROGRAM_ID=.*/VITE_VAULT_PROGRAM_ID=$NEW_PROGRAM_ID/" test-ui/.env
sed -i '' "s/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=$NEW_PROGRAM_ID/" .env.example

echo "   ✅ .env"
echo "   ✅ services/backend/.env"
echo "   ✅ test-ui/.env"
echo "   ✅ .env.example"
echo ""

# Update source code
echo "📝 Updating source code..."
sed -i '' "s/declare_id!(\"[A-Za-z0-9]*\")/declare_id!(\"$NEW_PROGRAM_ID\")/" solana-playground-deploy/programs/vault/src/lib.rs
echo "   ✅ solana-playground-deploy/programs/vault/src/lib.rs"
echo ""

# Update CLI script - more careful pattern matching
echo "📝 Updating CLI scripts..."
sed -i '' "s/\"[A-Za-z0-9]*\",$/\"$NEW_PROGRAM_ID\",/" scripts/approve-allowance-cli.js
echo "   ✅ scripts/approve-allowance-cli.js"
echo ""

echo "✅ Program ID updated in all files!"
echo ""
echo "📋 Next steps:"
echo "   1. Restart backend services:"
echo "      cd services/backend && pm2 restart backend processor"
echo ""
echo "   2. Rebuild frontend:"
echo "      cd test-ui && npm run build"
echo ""
echo "   3. Verify with:"
echo "      grep VAULT_PROGRAM_ID .env services/backend/.env test-ui/.env"
echo ""
echo "   4. Test allowance creation:"
echo "      node scripts/approve-allowance-cli.js phantom.json 1 3600"
echo ""
