# Phase 1B Deployment Guide

## Summary

Phase 1B addresses **Critical** severity security issues in the Atomik Wallet smart contracts:

- **C2:** Missing rent-exemption checks
- **C5:** Missing input validation (bet ID length)

## Files Modified

### 1. state.rs

**Path:** `solana-playground-deploy/programs/vault/src/state.rs`

**Changes:**

- Added fully documented constants with security rationale
- Added `RENT_EXEMPT_RESERVE_CASINO_VAULT = 1_343_280 lamports`
- Added `RENT_EXEMPT_RESERVE_USER_VAULT = 1_566_960 lamports`
- Changed `MAX_BET_ID_LENGTH` from 64 to 32 bytes (Solana PDA seed limit)
- Updated `ProcessedBet::LEN` calculation to use the constant

**Security Impact:** Prevents PDA derivation errors from oversized bet IDs

---

### 2. payout.rs

**Path:** `solana-playground-deploy/programs/vault/src/instructions/payout.rs`  
**Lines:** 79-93 (14 new lines added)

**Changes:**

```rust
// Get rent sysvar
let rent = Rent::get()?;

// Calculate minimum balance for rent exemption
let min_balance = rent.minimum_balance(casino_vault.to_account_info().data_len());

// Verify casino vault will remain rent-exempt after payout
let current_lamports = casino_vault.to_account_info().lamports();
require!(
    current_lamports.checked_sub(amount).unwrap_or(0) >= min_balance,
    VaultError::InsufficientBalance
);
```

**Security Impact:** Prevents casino vault from dropping below rent-exempt threshold, avoiding garbage collection

---

### 3. withdraw_casino_funds.rs

**Path:** `solana-playground-deploy/programs/vault/src/instructions/withdraw_casino_funds.rs`  
**Lines:** 31-44 (13 new lines added)

**Changes:** Same rent-exemption validation as payout.rs

**Security Impact:** Prevents admin withdrawals from violating rent-exemption

---

### 4. spend_from_allowance.rs

**Path:** `solana-playground-deploy/programs/vault/src/instructions/spend_from_allowance.rs`  
**Lines:** 91-108 (6 new lines added)

**Changes:**

```rust
// Validate bet ID length BEFORE PDA derivation (critical: prevents seed overflow)
require!(
    bet_id.len() <= MAX_BET_ID_LENGTH,
    VaultError::InvalidBetId
);
```

**Security Impact:** Validates bet_id length BEFORE ProcessedBet PDA creation (was previously after), prevents transaction failures from PDA seed overflow

---

## Deployment Steps

### 1. Open Solana Playground

Navigate to: https://beta.solpg.io/

### 2. Import Updated Code

- Upload all files from `solana-playground-deploy/programs/vault/`
- Ensure Anchor.toml is configured correctly
- Verify all dependencies in Cargo.toml

### 3. Build the Program

```bash
build
```

Verify successful compilation with no errors.

### 4. Deploy to New Program ID

**IMPORTANT:** Deploy to a NEW program ID for parallel testing. This allows the current production program to continue running while you test the updated version.

```bash
deploy
```

Note the new Program ID displayed after deployment.

### 5. Initialize Test Environment

Create new accounts with the new program ID:

```bash
# Initialize new Casino
# Initialize new CasinoVault with test funds
# Initialize test user Vault
```

---

## Testing Checklist

### Test 1: Rent-Exemption Validation (Payout)

**Objective:** Verify casino vault cannot be drained below rent-exempt threshold

**Steps:**

1. Note current casino_vault lamports balance
2. Calculate rent-exempt minimum: ~1,343,280 lamports
3. Attempt to payout an amount that would drop balance below minimum
4. **Expected Result:** Transaction fails with `InsufficientBalance` error

**Pass Criteria:** ✅ Transaction rejected before lamports transfer

---

### Test 2: Rent-Exemption Validation (Withdrawal)

**Objective:** Verify admin cannot withdraw below rent-exempt threshold

**Steps:**

1. As casino authority, attempt to withdraw funds that would drop balance below minimum
2. **Expected Result:** Transaction fails with `InsufficientBalance` error

**Pass Criteria:** ✅ Transaction rejected before withdrawal

---

### Test 3: Bet ID Length Validation

**Objective:** Verify bet_id length is enforced before PDA derivation

**Steps:**

1. Attempt to place bet with bet_id of 33+ characters (exceeds MAX_BET_ID_LENGTH=32)
2. **Expected Result:** Transaction fails with `InvalidBetId` error

**Pass Criteria:** ✅ Transaction rejected in handler, not PDA derivation

---

### Test 4: Normal Operations

**Objective:** Verify all valid operations still work correctly

**Steps:**

1. Place bet with valid bet_id (≤ 32 chars)
2. Payout with sufficient casino vault balance (stays above rent threshold)
3. Admin withdraw with casino vault staying above rent threshold
4. **Expected Results:** All transactions succeed

**Pass Criteria:** ✅ All operations complete successfully

---

### Test 5: Edge Cases

**Objective:** Test boundary conditions

**Steps:**

1. Bet ID exactly 32 characters → Should succeed
2. Bet ID 33 characters → Should fail
3. Payout leaving exactly rent-exempt minimum → Should succeed
4. Payout leaving 1 lamport less than minimum → Should fail

**Pass Criteria:** ✅ All boundaries enforced correctly

---

## Rollback Plan

If critical issues are discovered during testing:

### Revert Smart Contract Changes

```bash
cd /Users/reece/code/projects/atomik-wallet

git checkout solana-playground-deploy/programs/vault/src/state.rs
git checkout solana-playground-deploy/programs/vault/src/instructions/payout.rs
git checkout solana-playground-deploy/programs/vault/src/instructions/withdraw_casino_funds.rs
git checkout solana-playground-deploy/programs/vault/src/instructions/spend_from_allowance.rs
```

### Continue Using Production Program

Keep services pointed at production program ID:

```
5sKSxXZ79DUJpnA8MVVmKsikKrQ4oG7TpNMJCjVzFnLf
```

---

## Post-Deployment

### If All Tests Pass:

1. **Update Environment Variables** in services:
   - Backend: Update `PROGRAM_ID` in `.env`
   - Processor: Update `PROGRAM_ID` in `.env`

2. **Restart Services:**

   ```bash
   bash scripts/stop-services.sh
   bash scripts/start-services.sh
   ```

3. **Monitor Production:**
   - Watch for any PDA derivation errors (should be eliminated)
   - Watch for rent-exemption issues (should be prevented)
   - Monitor transaction success rates

4. **Update Documentation:**
   - Record new Program ID
   - Update [REFACTORING_LOG.md](../REFACTORING_LOG.md)
   - Mark Phase 1B as deployed

### If Tests Fail:

1. Document the specific failure
2. Execute rollback plan above
3. Analyze root cause
4. Fix issues and re-test in Solana Playground
5. Re-deploy when ready

---

## Next Phase

Once Phase 1B is successfully deployed and verified on devnet:

**Phase 1C: Backend Input Validation & Type Migration**

This will migrate the backend to use the shared types created in Phase 1A:

- Replace raw `String` with `shared::BetId`
- Replace raw `u64` with `shared::LamportAmount`
- Add validation at API boundaries

Phase 1C requires Phase 1B to be tested on-chain first to ensure the backend changes align with the smart contract changes.

---

## Questions or Issues?

If you encounter any problems during deployment or testing, check:

1. **Build Errors:** Ensure all imports are correct (MAX_BET_ID_LENGTH should be available from state.rs)
2. **PDA Derivation Errors:** Should be eliminated by bet_id length validation
3. **Rent-Exemption Errors:** Verify minimum balance calculations are correct
4. **Transaction Simulation:** Use Solana Explorer to view transaction details and error messages

---

_Created: 2026-01-18_  
_Phase: 1B - Smart Contract Critical Fixes_  
_Status: Ready for Deployment_
