# Solana Playground Deployment Instructions

## Updated Files for CasinoVault Architecture

The following files have been updated to implement program-owned CasinoVault:

### Modified Files

1. **`programs/vault/src/state.rs`**
   - Added `CasinoVault` struct (65 bytes)
   - PDA seeds: `[b"casino-vault", casino.key()]`

2. **`programs/vault/src/instructions/initialize_casino_vault.rs`**
   - Added `casino_vault: Account<'info, CasinoVault>` parameter
   - Initializes CasinoVault with casino ref, bump, balances, timestamps
   - Keeps `vault_authority` PDA for SPL token signing

3. **`programs/vault/src/instructions/payout.rs`**
   - Changed `casino_vault` from `UncheckedAccount` to `Account<'info, CasinoVault>`
   - Added balance tracking: `casino_vault.sol_balance`
   - Added reconciliation check before payout
   - Updates `last_activity` timestamp

4. **`programs/vault/src/instructions/spend_from_allowance.rs`**
   - Changed `casino_vault` to `Account<'info, CasinoVault>`
   - Added balance tracking when receiving bet funds
   - Added `vault_authority` parameter for SPL signing (Option A)

5. **`programs/vault/src/instructions/withdraw_casino_funds.rs`**
   - Changed `vault_authority` to `casino_vault: Account<'info, CasinoVault>`
   - Added balance check and tracking

## Deployment Steps

### Step 1: Update Code in Solana Playground

1. Go to https://beta.solpg.io
2. Open your existing vault project
3. Copy the contents of each modified file from `solana-playground-deploy/programs/vault/src/` to Solana Playground

**Files to update:**

- `src/state.rs`
- `src/instructions/initialize_casino_vault.rs`
- `src/instructions/payout.rs`
- `src/instructions/spend_from_allowance.rs`
- `src/instructions/withdraw_casino_funds.rs`

### Step 2: Build in Solana Playground

1. Click **Build** button in Solana Playground
2. Wait for compilation to complete
3. Verify no errors

Expected output:

```
Build successful
Program ID: HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP
```

### Step 3: Deploy Program

1. Click **Deploy** button in Solana Playground
2. Select **Devnet**
3. Confirm upgrade authority: `FwbvNxgJXy7bmxVYcGfGYkQDYhPvMp3ppysvW8VeckE7`
4. Wait for deployment to complete

Expected output:

```
Program deployed successfully
Transaction: <signature>
https://explorer.solana.com/tx/<signature>?cluster=devnet
```

### Step 4: Verify Deployment

```bash
solana program show HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP --url devnet
```

Check that:

- Program ID matches: `HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP`
- Authority matches: `FwbvNxgJXy7bmxVYcGfGYkQDYhPvMp3ppysvW8VeckE7`
- "Last Deployed In Slot" has updated timestamp

### Step 5: Run Migration Script

**IMPORTANT:** The existing Casino account needs to be closed and re-initialized to create the new CasinoVault account.

```bash
cd /Users/reece/code/projects/atomik-wallet
node initialize-casino-vault.js
```

The script will:

1. Check if casino exists
2. Initialize new CasinoVault account
3. Attempt to transfer funds from old vault_authority PDA to new casino_vault

**Expected output:**

```
🏗️  Initialize CasinoVault and Migrate Funds
===========================================

Authority: FwbvNxgJXy7bmxVYcGfGYkQDYhPvMp3ppysvW8VeckE7
Program ID: HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP
Casino PDA: FhTXCNZFUZwKzhYBdWsCbmQ6Uv3WLmn9fsst9wHtwZks
Casino Vault (NEW): <derived_address>
Vault Authority (OLD): DyfEu5bTTS4vC6eWfNZ1KkYDKgkv6KEQqiJdsELhS8ky

📊 Balances Before:
  Authority: X.XXXX SOL
  Old Vault Authority: 10.3100 SOL

✅ Casino Vault initialized!
Signature: <tx_sig>
https://explorer.solana.com/tx/<tx_sig>?cluster=devnet
```

### Step 6: Manual Fund Transfer (If Needed)

If the automatic transfer fails (vault_authority is a PDA and can't sign), manually transfer funds:

```bash
# Get the new casino_vault address from the script output
CASINO_VAULT=<address_from_script>

# Transfer funds manually
solana transfer $CASINO_VAULT 10.31 --url devnet --allow-unfunded-recipient
```

### Step 7: Verify Casino Vault

```bash
# Check casino vault account
solana account <casino_vault_address> --url devnet

# Should show:
# - Owner: HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP (program owned!)
# - Balance: 10.31 SOL (or whatever was transferred)
# - Data Length: 65 bytes
```

### Step 8: Restart Services

```bash
cd /Users/reece/code/projects/atomik-wallet

# Kill existing services
killall backend processor

# Restart backend (picks up new program)
cd services/backend
cargo build --release
./target/release/backend &

# Restart processor (picks up new program)
cd ../processor
cargo build --release
./target/release/processor &

# Check processor logs
tail -f logs/processor.log
```

### Step 9: Test End-to-End

1. Open UI: http://localhost:3000
2. Place a bet (0.01 SOL)
3. Monitor processor logs:

**Expected (SUCCESS):**

```json
{"level":"INFO","message":"Worker X: Processing 1 pending bets"}
{"level":"INFO","message":"Submitting 1 bets to Solana"}
{"level":"INFO","message":"Solana transaction confirmed: <sig>"}
{"level":"INFO","message":"Worker X: Batch <id> completed"}
```

**Should NOT see:**

```json
{"level":"ERROR","message":"ExternalAccountLamportSpend"}
{"level":"ERROR","message":"instruction spent from the balance of an account it does not own"}
```

4. Verify transaction on Solana Explorer
5. Check casino vault balance decreased (for loss) or increased (for win)

## Troubleshooting

### Error: "Casino already exists"

The script checks if casino exists. If re-initializing, you need to close the old casino account first (requires casino authority signature).

### Error: "Transfer failed"

The old vault_authority PDA cannot directly transfer (it's a PDA). Use manual transfer command provided by the script.

### Error: "Account discriminator mismatch"

Old account data doesn't match new structure. Need to close and re-initialize.

### Bets still failing with ExternalAccountLamportSpend

1. Verify program deployed: `solana program show HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP --url devnet`
2. Check processor is using correct program ID in `.env`
3. Restart processor to pick up new deployment
4. Check processor logs for correct account addresses

## Architecture Notes

### Option A: Dual Account Design (Implemented)

- **CasinoVault**: Program-owned account holding SOL (for bet payouts/receipts)
- **vault_authority**: PDA used only for signing SPL token transfers
- **Benefits**:
  - Direct lamports manipulation for SOL (cheap: ~100 CU)
  - Preserves existing SPL token logic
  - Clear separation of concerns

### Why This Fixes ExternalAccountLamportSpend

**Before:**

- `vault_authority` PDA was just a signing key (System Program owned)
- Program tried to subtract lamports from System Program-owned account
- Solana rejected: "instruction spent from the balance of an account it does not own"

**After:**

- `casino_vault` is a proper Account<CasinoVault> (program owned)
- Program CAN subtract lamports from its own accounts
- Direct lamports manipulation now works

### PDA Derivations

```rust
// Casino (unchanged)
seeds: [b"casino"]
address: FhTXCNZFUZwKzhYBdWsCbmQ6Uv3WLmn9fsst9wHtwZks

// Casino Vault (NEW - program owned)
seeds: [b"casino-vault", casino.key()]
owner: HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP

// Vault Authority (OLD - kept for SPL signing)
seeds: [b"vault-authority", casino.key()]
owner: 11111111111111111111111111111111 (System Program)
```

## Success Criteria

✅ Program deploys successfully on Solana Playground  
✅ Casino vault initializes with proper program ownership  
✅ Bets complete on first attempt (no retries)  
✅ No ExternalAccountLamportSpend errors in processor logs  
✅ Casino vault balance tracking matches actual lamports  
✅ Withdrawals work via withdraw_casino_funds instruction

## Next Steps After Deployment

1. Update `fund-casino-vault.js` to use new casino_vault PDA derivation
2. Monitor first-attempt success rate (should be >95%)
3. Test multiple concurrent bets (batch processing)
4. Verify balance reconciliation over time
5. Consider adding balance drift monitoring alert
