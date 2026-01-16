# Manual Deployment Guide

## Issue: Anchor Build Failing

The automated build is encountering `edition2024` errors due to dependency version conflicts between Anchor versions and Rust toolchains. Here's how to proceed:

## Option 1: Deploy Using Solana Playground (RECOMMENDED)

1. Go to https://beta.solpg.io
2. Create a new Anchor project
3. Copy all files from `programs/vault/src/` to the playground
4. Copy `programs/vault/Cargo.toml` 
5. Update Cargo.toml in playground to use:
   ```toml
   [dependencies]
   anchor-lang = "0.29.0"
   anchor-spl = "0.29.0"
   ```
6. Click "Build" in playground
7. Click "Deploy" - it will deploy to devnet
8. Copy the program ID from deployment
9. Update the program ID in:
   - `services/backend/.env` → `VAULT_PROGRAM_ID=<your-program-id>`
   - `services/processor/.env` → `VAULT_PROGRAM_ID=<your-program-id>`

## Option 2: Build Locally (if you can get it working)

```bash
# Try with fresh environment
cd /Users/reece/code/projects/atomik-wallet

# Method 1: Use cargo build-sbf directly
cargo build-sbf --manifest-path programs/vault/Cargo.toml --sbf-out-dir target/deploy

# Method 2: Use anchor build (if 0.29.0 CLI available)
anchor build

# After successful build:
solana program deploy target/deploy/vault.so --url devnet

# Update .env files with the program ID from deployment output
```

## Option 3: Continue Without Real Blockchain (FASTEST FOR TESTING)

The system is already functional in simulation mode:

1. Keep `USE_REAL_SOLANA=false` in `services/processor/.env`
2. The processor will generate fake transaction signatures
3. All backend APIs work
4. You can test the full flow without blockchain

To test:
```bash
# Start backend
cd services/backend
cargo run

# Start processor
cd services/processor  
cargo run

# Create test bets via API
curl -X POST http://localhost:3001/api/bets \
  -H "Content-Type: application/json" \
  -d '{"stake_amount": 1000000, "stake_token": "SOL", "choice": "heads"}'

# Check processing
curl http://localhost:3001/api/external/bets/pending?limit=10
```

## What's Already Completed

✅ Backend API with all endpoints
✅ Processor with batch processing
✅ Redis pub/sub for instant bet notifications
✅ Batch update endpoint for status updates
✅ Database schema and migrations
✅ Real Solana transaction builder code
✅ Feature flag system (USE_REAL_SOLANA)
✅ E2E tests passing (6/6 bets in 48ms)

## What Needs Blockchain Deployment

⏳ Deploy Anchor program to devnet
⏳ Initialize casino vault on-chain
⏳ Test real Solana transactions

## Next Steps (Choose One)

**Fast Path (No Blockchain):**
- Start services and test with simulated transactions
- Frontend can be connected without blockchain

**Full Blockchain Path:**
- Deploy program via Solana Playground
- Update program IDs
- Fund processor keypair: `solana airdrop 2 $(solana-keygen pubkey test-keypair.json) --url devnet`
- Set `USE_REAL_SOLANA=true`
- Test real transactions

## Build Issue Details

The issue is `constant_time_eq v0.4.2` requires `edition2024` (Rust 1.90.0+) but Anchor 0.32's pinned toolchain uses Rust 1.79.0 with Cargo 1.84.0. Downgrading to Anchor 0.29.0 should fix it, but AVM is failing to switch versions.
