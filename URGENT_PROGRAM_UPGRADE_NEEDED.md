# 🚨 URGENT: Program Upgrade Required

## Current Status

**Problem:** The on-chain program at `HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP` is deployed with **BUGGY CODE** that causes:

```
Transfer: `from` must not carry data
Program 11111111111111111111111111111111 failed: invalid program argument
```

## Evidence

### Test Results

- ✅ Backend API working
- ✅ Processor working
- ✅ Database working
- ✅ Bet created successfully
- ❌ **On-chain execution FAILS** with native SOL transfer error

### Processor Logs

```
ERROR: Preflight simulation failed: InstructionError(0, InvalidArgument)
Program log: Instruction: SpendFromAllowance
Transfer: `from` must not carry data
Program 11111111111111111111111111111111 failed: invalid program argument
```

### Source Code Status

- ✅ `solana-playground-deploy/programs/vault/src/instructions/spend_from_allowance.rs` **HAS THE FIX** (line 2: `use anchor_lang::solana_program::{program::invoke_signed, system_instruction}`)
- ❌ On-chain program deployed **WITHOUT THE FIX**

## What Happened

When you created the new program `HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP`, it was deployed with the OLD buggy code that uses Anchor's `system_program::transfer` helper which cannot transfer from accounts with data (like our Vault PDA).

## Solution: Upgrade Program

You need to build and upgrade the program using the code from `solana-playground-deploy/`:

### Option A: Solana Playground (Easiest)

1. Open Solana Playground
2. Import the code from `solana-playground-deploy/`
3. Click "Build"
4. Once built, use the upgrade command:
   ```
   solana program deploy \
     --program-id HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP \
     --upgrade-authority <YOUR_KEYPAIR> \
     --url devnet
   ```

### Option B: CLI (Advanced)

```bash
cd solana-playground-deploy

# Build the program
anchor build

# Upgrade the deployed program
solana program deploy \
  --program-id HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP \
  --upgrade-authority ~/.config/solana/id.json \
  --url devnet \
  target/deploy/vault.so
```

### Verification

After upgrade, check program show:

```bash
solana program show HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP --url devnet
```

The "Last Deployed Slot" should be updated to a recent slot.

## After Upgrade: Full Test Flow

Once the program is upgraded, you can test the complete betting flow:

### Step 1: Initialize Casino (if not done)

```bash
# Use test wallet
solana-keygen pubkey test-user-keypair.json
# Output: LCsLwQ74zUfa5UDA6fNTRPyddH6akTd6S1fkdMAQQj8

# Casino PDA (deterministic)
# FhTXCNZFUZwKzhYBdWsCbmQ6Uv3WLmn9fsst9wHtwZks
```

### Step 2: Create Allowance

```bash
node scripts/approve-allowance-cli.js \
  test-user-keypair.json \
  1.0 \
  86400
```

This will output the **Allowance PDA** - save it!

### Step 3: Place Bet

```bash
node place-real-bet.js
```

Or manually:

```bash
curl -X POST http://localhost:3001/api/bets \
  -H "Content-Type: application/json" \
  -d '{
    "user_wallet": "LCsLwQ74zUfa5UDA6fNTRPyddH6akTd6S1fkdMAQQj8",
    "vault_address": "Hw4eEdRB2Z5MuMCJkCuzdvZ77JY3mrTrtAwcJEKSGQD8",
    "allowance_pda": "<ALLOWANCE_PDA_FROM_STEP2>",
    "stake_amount": 100000000,
    "stake_token": "SOL",
    "choice": "heads"
  }'
```

### Step 4: Monitor

```bash
# Check bet status
curl http://localhost:3001/api/bets/<BET_ID> | jq

# Watch processor logs
tail -f logs/processor.log | grep "confirmed"
```

### Expected Result

```json
{
  "bet_id": "...",
  "status": "completed",
  "won": true,
  "payout_amount": 200000000,
  "solana_tx_id": "5K2e..." // REAL transaction signature (no SIM_ prefix)
}
```

View transaction on Solana Explorer:

```
https://explorer.solana.com/tx/<TX_ID>?cluster=devnet
```

## Current State

- Casino PDA: `FhTXCNZFUZwKzhYBdWsCbmQ6Uv3WLmn9fsst9wHtwZks`
- User Wallet: `LCsLwQ74zUfa5UDA6fNTRPyddH6akTd6S1fkdMAQQj8`
- User Vault PDA: `Hw4eEdRB2Z5MuMCJkCuzdvZ77JY3mrTrtAwcJEKSGQD8`
- User Balance: 0.092960002 SOL (enough for testing)

All infrastructure is ready. **Just need to upgrade the program!**

## Reference: The Fix

The fix in `spend_from_allowance.rs` lines 118-133:

```rust
// OLD (buggy) - cannot transfer from accounts with data:
system_program::transfer(ctx, amount)?;

// NEW (fixed) - allows transfers from accounts with data:
invoke_signed(
    &system_instruction::transfer(
        &vault.key(),
        &ctx.accounts.casino_vault.key(),
        amount,
    ),
    &[
        vault.to_account_info(),
        ctx.accounts.casino_vault.to_account_info(),
    ],
    signer_seeds,
)?;
```

This uses the raw Solana system_instruction instead of Anchor's helper, which allows transferring SOL from PDAs that have account data (like our Vault account).
