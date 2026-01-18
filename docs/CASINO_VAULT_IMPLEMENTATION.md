# Casino Vault Implementation Summary

## Problem Solved

**Error:** `ExternalAccountLamportSpend` - "instruction spent from the balance of an account it does not own"

**Root Cause:** The casino vault (vault_authority PDA) was owned by the System Program, not the vault program. Solana's security model prevents programs from modifying lamports of accounts they don't own.

**Solution:** Created a proper program-owned `CasinoVault` account to hold casino funds, enabling direct lamports manipulation.

---

## Changes Made

### 1. New State Structure: `CasinoVault` (state.rs)

```rust
#[account]
pub struct CasinoVault {
    pub casino: Pubkey,        // Casino reference
    pub bump: u8,              // PDA bump
    pub sol_balance: u64,      // Tracked SOL balance
    pub created_at: i64,       // Creation timestamp
    pub last_activity: i64,    // Last activity timestamp
}

impl CasinoVault {
    pub const LEN: usize = 65; // 8 + 32 + 1 + 8 + 8 + 8
}
```

**PDA Derivation:** `[b"casino-vault", casino.key()]`

---

### 2. Updated Initialization (initialize_casino_vault.rs)

**Before:**

- Only created Casino account
- Derived vault_authority PDA for signing

**After:**

- Creates Casino account (unchanged)
- **Creates CasinoVault account (NEW)** - program-owned, holds casino funds
- Derives vault_authority PDA for SPL token signing only

```rust
#[account(
    init,
    payer = authority,
    space = CasinoVault::LEN,
    seeds = [b"casino-vault", casino.key().as_ref()],
    bump
)]
pub casino_vault: Account<'info, CasinoVault>,
```

---

### 3. Updated Payout (payout.rs)

**Before:**

```rust
/// CHECK: This is the vault authority PDA that holds casino funds
pub casino_vault: UncheckedAccount<'info>,
```

**After:**

```rust
pub casino_vault: Account<'info, CasinoVault>,
```

**Key Changes:**

- Proper account type with PDA seeds constraint
- Added balance reconciliation check before payout
- Updates `casino_vault.sol_balance` and `last_activity`
- Direct lamports manipulation now works (program-owned account)

---

### 4. Updated Bet Processing (spend_from_allowance.rs)

**Before:**

```rust
/// CHECK: Either casino vault PDA for SOL or vault authority for SPL
pub casino_vault: UncheckedAccount<'info>,
```

**After:**

```rust
pub casino_vault: Account<'info, CasinoVault>,
```

**Key Changes:**

- Proper account type with PDA seeds constraint
- Updates `casino_vault.sol_balance` when receiving bet funds
- Updates `last_activity` timestamp
- Keeps `vault_authority` as separate parameter for SPL token signing (Option A)

---

### 5. Updated Admin Withdrawal (withdraw_casino_funds.rs)

**Before:**

```rust
/// CHECK: This is a PDA owned by the program
pub vault_authority: UncheckedAccount<'info>,
```

**After:**

```rust
pub casino_vault: Account<'info, CasinoVault>,
```

**Key Changes:**

- Uses proper CasinoVault account
- Added balance check with `InsufficientBalance` error
- Updates `sol_balance` tracking
- Updates `last_activity` timestamp

---

## Architecture: Option A (Dual Account Design)

### Accounts Used:

1. **CasinoVault** (NEW)
   - **Owner:** Vault Program (HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP)
   - **Purpose:** Holds casino SOL funds
   - **Seeds:** `[b"casino-vault", casino.key()]`
   - **Used in:** Payout, SpendFromAllowance, WithdrawCasinoFunds

2. **vault_authority** (EXISTING - kept)
   - **Owner:** System Program
   - **Purpose:** Signing authority for SPL token transfers only
   - **Seeds:** `[b"vault-authority", casino.key()]`
   - **Used in:** SPL token CPI calls

### Why Dual Design?

- **SOL transfers:** Use direct lamports manipulation on program-owned CasinoVault (~100 CU)
- **SPL transfers:** Use CPI with vault_authority signing (~5k CU)
- **Benefits:** Minimal refactoring, preserves SPL logic, cheap SOL operations

---

## Migration Required

### Old Architecture:

```
vault_authority PDA (System Program-owned)
  └─ Funded with SOL via regular transfer
  └─ Used as signing key
  └─ Cannot subtract lamports (ExternalAccountLamportSpend error)
```

### New Architecture:

```
casino_vault (Program-owned CasinoVault account)
  └─ Initialized via init constraint
  └─ Holds SOL funds
  └─ Can subtract lamports (program owns it)

vault_authority PDA (System Program-owned)
  └─ Used only for SPL token signing
  └─ No longer holds funds
```

### Migration Steps:

1. Deploy updated program
2. Run `initialize-casino-vault.js`
3. Transfer 10.31 SOL from old vault_authority to new casino_vault
4. Restart processor
5. Test bet flow

---

## Files Modified

1. **state.rs** - Added CasinoVault struct
2. **initialize_casino_vault.rs** - Creates CasinoVault account
3. **payout.rs** - Uses CasinoVault, adds balance tracking
4. **spend_from_allowance.rs** - Uses CasinoVault, tracks incoming funds
5. **withdraw_casino_funds.rs** - Uses CasinoVault, adds balance checks

---

## Files Created

1. **initialize-casino-vault.js** - Migration script
2. **SOLANA_PLAYGROUND_DEPLOYMENT.md** - Deployment guide
3. **CASINO_VAULT_IMPLEMENTATION.md** - This summary

---

## Expected Results

### Before Fix:

```json
{"level":"ERROR","message":"ExternalAccountLamportSpend"}
{"level":"ERROR","message":"instruction spent from the balance of an account it does not own"}
```

- Retries: 1-4 per bet
- First-attempt success: ~0%

### After Fix:

```json
{"level":"INFO","message":"Solana transaction confirmed: <sig>"}
{"level":"INFO","message":"Worker X: Batch completed in 2.3s"}
```

- Retries: 0
- First-attempt success: >95%

---

## Balance Tracking

CasinoVault maintains an on-chain `sol_balance` field that tracks SOL without RPC calls:

### Operations that modify balance:

- **SpendFromAllowance:** `sol_balance += bet_amount` (casino receives lost bet)
- **Payout:** `sol_balance -= payout_amount` (casino pays winning bet)
- **WithdrawCasinoFunds:** `sol_balance -= withdrawal_amount` (admin withdraws profits)

### Reconciliation:

Balance check before operations:

```rust
require!(
    casino_vault.sol_balance >= amount,
    VaultError::InsufficientBalance
);
```

Prevents overdrafts and detects balance drift.

---

## Compute Unit Impact

### Direct Lamports Manipulation:

```rust
**casino_vault.try_borrow_mut_lamports()? -= amount;  // ~100 CU
**vault.try_borrow_mut_lamports()? += amount;          // ~100 CU
```

### CPI Transfer (alternative, not used):

```rust
invoke_signed(&system_instruction::transfer(...))     // ~5,000 CU
```

**Savings per bet:** ~4,800 CU  
**Savings per batch (5 bets, 10 instructions):** ~48,000 CU  
**Critical:** Keeps batches under 1.4M CU limit

---

## Security Properties

### Enforced On-Chain:

1. Only program can modify CasinoVault lamports
2. Balance checks prevent overdrafts
3. Admin-only withdrawal via casino.authority constraint
4. PDA derivation ensures unique vault per casino

### Not Changed:

- User vault security (unchanged)
- Allowance system (unchanged)
- Processor authorization (unchanged)

---

## Multi-Casino Support

PDA derivation `[b"casino-vault", casino.key()]` already supports multiple casinos:

```
Casino 1: FhTXCNZFUZwKzhYBdWsCbmQ6Uv3WLmn9fsst9wHtwZks
  └─ CasinoVault 1: <derived from Casino 1>

Casino 2: <future casino PDA>
  └─ CasinoVault 2: <derived from Casino 2>
```

Each casino gets its own isolated vault.

---

## Testing Checklist

- [ ] Program builds in Solana Playground
- [ ] Program deploys to devnet
- [ ] Migration script initializes CasinoVault
- [ ] Funds transfer from old vault to new vault
- [ ] Casino vault account shows program ownership
- [ ] Bet placement succeeds on first attempt
- [ ] No ExternalAccountLamportSpend errors in logs
- [ ] Casino vault balance updates correctly
- [ ] Payout instruction works
- [ ] Withdraw casino funds works
- [ ] Balance tracking remains accurate over multiple bets

---

## Rollback Plan (If Needed)

If deployment fails or issues arise:

1. Keep old program deployed (HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP)
2. Revert to old code in Solana Playground
3. Re-deploy with `solana program deploy`
4. Processor continues using old architecture

**Note:** Cannot easily rollback after CasinoVault initialization (would need to close account and re-fund old vault_authority)

---

## Future Enhancements

1. **Balance reconciliation job:** Periodically check `sol_balance` matches actual lamports
2. **Multi-token support:** Extend CasinoVault to track SPL token balances
3. **Liquidity monitoring:** Alert if casino vault balance drops below threshold
4. **Admin dashboard:** Show real-time casino vault balance and activity
5. **Automated rebalancing:** Transfer excess profits to treasury

---

## References

- **Solana Account Ownership:** https://docs.solana.com/developing/programming-model/accounts#ownership
- **Direct Lamports Manipulation:** Standard pattern for escrow/vault programs
- **PDA Signing:** https://solanacookbook.com/references/programs.html#how-to-do-cross-program-invocation
