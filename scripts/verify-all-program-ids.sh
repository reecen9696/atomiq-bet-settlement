#!/bin/bash

echo "============================================"
echo "🔍 COMPREHENSIVE PROGRAM ID VERIFICATION"
echo "============================================"
echo ""

NEW_PROGRAM="HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP"
OLD_PROGRAM="Cek6v3J44BS6mpoUGjSqTeCUgTViUzpQKkMLcuiZsoxL"

echo "Expected (NEW): $NEW_PROGRAM"
echo "Old (deprecated): $OLD_PROGRAM"
echo ""

echo "📁 Checking Environment Files..."
echo "================================="

check_file() {
    local file=$1
    local pattern=$2
    
    if [ -f "$file" ]; then
        if grep -q "$OLD_PROGRAM" "$file" 2>/dev/null; then
            echo "❌ $file - STILL HAS OLD PROGRAM ID"
            return 1
        elif grep -q "$NEW_PROGRAM" "$file" 2>/dev/null; then
            echo "✅ $file - Using new program ID"
            return 0
        else
            echo "⚠️  $file - Program ID not found"
            return 2
        fi
    else
        echo "⚠️  $file - File not found"
        return 3
    fi
}

# Check critical env files
check_file ".env" "VAULT_PROGRAM_ID"
check_file "services/backend/.env" "VAULT_PROGRAM_ID"
check_file "test-ui/.env" "VITE_VAULT_PROGRAM_ID"

echo ""
echo "📝 Checking Source Code..."
echo "==========================="
check_file "solana-playground-deploy/programs/vault/src/lib.rs" "declare_id"
check_file "programs/vault/src/lib.rs" "declare_id"
check_file "Anchor.toml" "vault"

echo ""
echo "🖥️  Checking Running Processes..."
echo "=================================="

if pgrep -f "npm run dev" > /dev/null; then
    echo "✅ Vite dev server is running"
    if lsof -i :3000 > /dev/null 2>&1; then
        echo "✅ Port 3000 is open"
    else
        echo "⚠️  Port 3000 is not responding"
    fi
else
    echo "❌ Vite dev server is NOT running"
fi

if pgrep -f "backend" > /dev/null; then
    echo "✅ Backend service is running"
else
    echo "⚠️  Backend service is NOT running"
fi

if pgrep -f "processor" > /dev/null; then
    echo "✅ Processor service is running"
else
    echo "⚠️  Processor service is NOT running"
fi

echo ""
echo "🔐 Deriving PDAs..."
echo "==================="

node -e "
const { PublicKey } = require('@solana/web3.js');

const NEW_PROGRAM = '$NEW_PROGRAM';
const OLD_PROGRAM = '$OLD_PROGRAM';

const [casinoNew] = PublicKey.findProgramAddressSync(
  [Buffer.from('casino')],
  new PublicKey(NEW_PROGRAM)
);

const [casinoOld] = PublicKey.findProgramAddressSync(
  [Buffer.from('casino')],
  new PublicKey(OLD_PROGRAM)
);

console.log('NEW Program Casino PDA:', casinoNew.toBase58());
console.log('OLD Program Casino PDA:', casinoOld.toBase58());
" 2>/dev/null || echo "⚠️  Could not derive PDAs (Solana Web3.js not found)"

echo ""
echo "📋 Next Steps:"
echo "=============="
echo "1. Open browser to: http://localhost:3000/verify-program-id.html"
echo "2. Check the program ID shown matches: $NEW_PROGRAM"
echo "3. If still showing old program:"
echo "   - Hard refresh browser: Cmd+Shift+R (Mac) or Ctrl+Shift+R"
echo "   - Clear cache: Open console → localStorage.clear(); location.reload();"
echo "   - Check browser console for errors"
echo "4. Expected Casino PDA: FhTXCNZFUZwKzhYBdWsCbmQ6Uv3WLmn9fsst9wHtwZks"
echo ""
echo "✅ If you see the old casino PDA after refresh, check:"
echo "   - Browser is actually loading from localhost:3000 (not cached version)"
echo "   - No service worker caching old code"
echo "   - Private/incognito window as test"
echo ""
