# Program ID Update Guide

## Current Program ID

**Program ID:** `HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP`  
**Upgrade Authority:** `FwbvNxgJXy7bmxVYcGfGYkQDYhPvMp3ppysvW8VeckE7`  
**Network:** Solana Devnet

## When to Update Program ID

Update the program ID when:

- Deploying a new version of the on-chain program
- Switching between devnet/mainnet
- Creating a fresh deployment (not upgrade)

## Files to Update

### 1. Environment Files (REQUIRED - affects running services)

These files control the actual program ID used by backend and frontend:

```bash
# Root environment
.env                              # VAULT_PROGRAM_ID=
.env.example                     # Template for new developers

# Backend service
services/backend/.env            # VAULT_PROGRAM_ID=

# Frontend
test-ui/.env                     # VITE_VAULT_PROGRAM_ID=
```

### 2. Source Code Files (for build/deploy)

```bash
# On-chain program declaration
solana-playground-deploy/programs/vault/src/lib.rs   # declare_id!("...")
```

### 3. Scripts (for CLI tools)

```bash
scripts/approve-allowance-cli.js    # PROGRAM_ID fallback value
scripts/transfer-sol.js             # If used, update fallback
```

## Quick Update Procedure

### Step 1: Update Environment Variables

```bash
# Update all .env files with new program ID
sed -i '' 's/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=NEW_PROGRAM_ID/' .env
sed -i '' 's/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=NEW_PROGRAM_ID/' services/backend/.env
sed -i '' 's/VITE_VAULT_PROGRAM_ID=.*/VITE_VAULT_PROGRAM_ID=NEW_PROGRAM_ID/' test-ui/.env
```

### Step 2: Restart Services

```bash
# Restart backend and processor
cd services/backend && cargo build --release
pm2 restart backend
pm2 restart processor

# Rebuild frontend
cd test-ui && npm run build
```

### Step 3: Update Source (if redeploying)

```bash
# Update on-chain program declaration
# Edit: solana-playground-deploy/programs/vault/src/lib.rs
# Change: declare_id!("NEW_PROGRAM_ID");
```

## Automated Update Script

Create a script `scripts/update-program-id.sh`:

```bash
#!/bin/bash
set -e

NEW_PROGRAM_ID=$1

if [ -z "$NEW_PROGRAM_ID" ]; then
    echo "Usage: ./scripts/update-program-id.sh <NEW_PROGRAM_ID>"
    exit 1
fi

echo "Updating program ID to: $NEW_PROGRAM_ID"

# Update all env files
sed -i '' "s/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=$NEW_PROGRAM_ID/" .env
sed -i '' "s/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=$NEW_PROGRAM_ID/" services/backend/.env
sed -i '' "s/VITE_VAULT_PROGRAM_ID=.*/VITE_VAULT_PROGRAM_ID=$NEW_PROGRAM_ID/" test-ui/.env
sed -i '' "s/VAULT_PROGRAM_ID=.*/VAULT_PROGRAM_ID=$NEW_PROGRAM_ID/" .env.example

# Update source code
sed -i '' "s/declare_id!(\".*\")/declare_id!(\"$NEW_PROGRAM_ID\")/" solana-playground-deploy/programs/vault/src/lib.rs

# Update CLI script
sed -i '' "s/\"[A-Za-z0-9]*\",$/\"$NEW_PROGRAM_ID\",/" scripts/approve-allowance-cli.js

echo "✅ Program ID updated in all files!"
echo ""
echo "Next steps:"
echo "1. Restart backend: cd services/backend && pm2 restart backend processor"
echo "2. Rebuild frontend: cd test-ui && npm run build"
echo "3. Test with: node scripts/approve-allowance-cli.js"
```

## Verification

After updating, verify the program ID is correct:

```bash
# Check environment
grep -r "VAULT_PROGRAM_ID" .env services/backend/.env test-ui/.env

# Check if services see the new ID
curl http://localhost:3001/health | jq .

# Test transaction with new program
node scripts/approve-allowance-cli.js phantom.json 1 3600
```

## Deployment History

| Date       | Program ID                                     | Network | Notes                                                      |
| ---------- | ---------------------------------------------- | ------- | ---------------------------------------------------------- |
| 2026-01-17 | `HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP` | devnet  | Fixed native SOL transfer, MIN_BET=0.01, MAX_APPROVALS=100 |
| 2026-01-16 | `Cek6v3J44BS6mpoUGjSqTeCUgTViUzpQKkMLcuiZsoxL` | devnet  | Original deployment (deprecated)                           |

## Common Issues

### Services using old program ID

- **Problem:** Backend still references old program
- **Fix:** Restart services after .env update: `pm2 restart all`

### Frontend shows wrong program

- **Problem:** .env changes not reflected in browser
- **Fix:** Hard refresh (Cmd+Shift+R) or clear localStorage and reload

### Transactions fail with "Invalid program ID"

- **Problem:** Mismatch between frontend/backend program IDs
- **Fix:** Verify all .env files have identical program ID

### Cannot find program accounts

- **Problem:** Old PDAs from previous program deployment
- **Fix:** Must reinitialize casino and create new vaults for new program

## Additional Notes

- **Environment variables** are the source of truth for running services
- **Source code** `declare_id!` is only used during program build/deploy
- Always update `.env.example` so new developers get correct defaults
- CLI scripts can read from `.env` files or use hardcoded fallbacks
- After updating program ID, existing PDAs and allowances from old program are **not compatible**
