# Program Update Complete - 2026-01-17

## ✅ New Program Deployed

**Program ID:** `HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP`  
**Network:** Solana Devnet  
**Upgrade Authority:** `FwbvNxgJXy7bmxVYcGfGYkQDYhPvMp3ppysvW8VeckE7`

## Changes in This Deployment

### On-Chain Program Updates

- ✅ Fixed native SOL transfer (uses `invoke_signed` with raw `system_instruction::transfer`)
- ✅ MIN_BET reduced to 0.01 SOL (10M lamports)
- ✅ MAX_APPROVALS increased to 100 (from 10)
- ✅ Proper casino account mutability

### Files Updated

#### Environment Files

- [x] `.env` - VAULT_PROGRAM_ID=HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP
- [x] `services/backend/.env` - VAULT_PROGRAM_ID=HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP
- [x] `test-ui/.env` - VITE_VAULT_PROGRAM_ID=HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP
- [x] `.env.example` - Updated template

#### Source Code

- [x] `solana-playground-deploy/programs/vault/src/lib.rs` - declare_id!
- [x] `scripts/approve-allowance-cli.js` - Program ID fallback

#### Services

- [x] Backend rebuilt (release mode)
- [x] Processor rebuilt (release mode)
- [x] Frontend rebuilt (Vite production build)

## Quick Reference Commands

### Verify Program ID

```bash
# Check all environment files
grep -r "VAULT_PROGRAM_ID" .env services/backend/.env test-ui/.env

# View on Solana Explorer
open "https://explorer.solana.com/address/HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP?cluster=devnet"

# Check program details
solana program show HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP --url devnet
```

### Test Allowance Creation

```bash
# Create 5 SOL allowance for 24 hours
node scripts/approve-allowance-cli.js phantom.json 5 86400

# Check allowance PDA
solana account <ALLOWANCE_PDA> --url devnet
```

### Future Updates

Use the automated script:

```bash
./scripts/update-program-id.sh <NEW_PROGRAM_ID>
```

See [PROGRAM_ID_UPDATE_GUIDE.md](./PROGRAM_ID_UPDATE_GUIDE.md) for detailed instructions.

## Next Steps

### 1. Initialize New Casino (REQUIRED)

The new program needs a fresh casino initialization:

```bash
# Will need to initialize casino with new program
# Use Solana Playground or CLI script
```

### 2. Create Test Allowance

```bash
node scripts/approve-allowance-cli.js phantom.json 1 3600
```

### 3. Place Test Bet

- Use test-ui at http://localhost:3000
- Or use backend API: POST /api/bets/place

### 4. Monitor Processor

```bash
# Check logs (adjust based on how services run)
tail -f services/processor/logs/processor.log

# Or check with your process manager
```

## Known State

### Wallets

- **Processor:** `8JQCVcxGMN2kQKXDzgCEJN8AawnQskWU4ha6NqZ83uDm` (~1 SOL)
- **Test Wallet:** `FwbvNxgJXy7bmxVYcGfGYkQDYhPvMp3ppysvW8VeckE7` (~7 SOL)
- **Test User:** `3XJ7ZZWbJHp727k1Afnk8P9xevcGGGH9mjAWZT6Pr33A`

### Settings

- Minimum bet: 0.01 SOL (10,000,000 lamports)
- Maximum bet: 1000 SOL
- Max allowance duration: 24 hours (86400 seconds)
- Max approvals per hour: 100

### Old Program (Deprecated)

- Old Program ID: `Cek6v3J44BS6mpoUGjSqTeCUgTViUzpQKkMLcuiZsoxL`
- Do not use - has old transfer logic and incorrect constants

## Testing Checklist

- [ ] Initialize casino with new program
- [ ] Create vault for test user
- [ ] Deposit SOL to vault
- [ ] Create allowance (CLI or UI)
- [ ] Place bet below 0.01 SOL (should fail)
- [ ] Place bet at 0.01 SOL (should succeed)
- [ ] Verify processor executes transaction
- [ ] Check on-chain transaction in Explorer
- [ ] Verify payout received

## Troubleshooting

### Services not picking up new program ID

```bash
# Verify .env files
grep VAULT_PROGRAM_ID .env services/backend/.env test-ui/.env

# Restart services (adjust command based on your setup)
pm2 restart all
# or
killall backend processor && cd services/backend && cargo run --release &
```

### Frontend shows old program

```bash
# Rebuild and hard refresh
cd test-ui && npm run build
# Then in browser: Cmd+Shift+R (Mac) or Ctrl+Shift+R (Windows/Linux)
```

### Transaction fails

- Check you're using correct program ID in all services
- Verify casino has been initialized on new program
- Check wallet has sufficient funds
- Verify allowance is not expired

---

**Date:** 2026-01-17  
**Updated By:** GitHub Copilot  
**Status:** ✅ Complete - Ready for Testing
