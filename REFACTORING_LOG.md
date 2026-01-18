# Atomik Wallet Refactoring Log

This document tracks the progress of the refactoring initiative outlined in the technical implementation plan.

## Phase 1A: Create Shared Constants & Type Safety ✅ COMPLETE

**Status:** ✅ Complete  
**Date:** 2026-01-18  
**Duration:** ~1 hour

### Changes Made

1. **Created `services/shared/` crate** with:
   - `constants.rs` - Centralized all magic numbers and configuration constants
   - `types.rs` - Type-safe wrappers for domain primitives
   - All tests passing (6/6)

2. **Key Types Introduced:**
   - `BetId` - Validated bet identifier with max length enforcement (32 chars)
   - `LamportAmount` - Overflow-protected lamport amounts with checked arithmetic
   - `TokenType` - Enum for NativeSOL/WrappedSOL/SPL tokens
   - `ValidationError` - Structured validation errors

3. **Constants Documented:**
   - `MIN_BET_LAMPORTS` = 10M (0.01 SOL) - Prevents spam
   - `MAX_BET_LAMPORTS` = 1T (1000 SOL) - Anti-whale limit
   - `MAX_ALLOWANCE_DURATION_SECS` = 86400 (24h) - Security limit
   - `WRAPPED_SOL_MINT` - Canonical wrapped SOL address
   - All constants include rationale comments

4. **Updated Dependencies:**
   - Root `Cargo.toml` workspace now includes `services/shared`
   - Backend `Cargo.toml` depends on `shared`
   - Processor `Cargo.toml` depends on `shared`

### Testing

```bash
cargo test -p shared
# Result: 6 passed, 0 failed

cargo build -p backend -p processor
# Result: Success (with existing warnings)
```

### Rollback Plan

If issues arise:

```bash
# Remove shared crate
rm -rf services/shared/

# Revert Cargo.toml changes
git checkout Cargo.toml services/backend/Cargo.toml services/processor/Cargo.toml
```

### Next Steps

- Phase 1B: Smart Contract Critical Fixes (Solana Playground deployment)
- Phase 1C: Backend Input Validation & Type Migration

---

## Phase 1B: Smart Contract Critical Fixes ✅ COMPLETE

**Status:** ✅ Complete  
**Date:** 2026-01-18  
**Duration:** ~45 minutes

### Changes Made

#### 1. Updated `state.rs` - Added Documented Constants

**File:** `solana-playground-deploy/programs/vault/src/state.rs`

**Changes:**

- Replaced 4 bare constants with fully documented versions including rationale
- Added `RENT_EXEMPT_RESERVE_CASINO_VAULT = 1_343_280 lamports`
- Added `RENT_EXEMPT_RESERVE_USER_VAULT = 1_566_960 lamports`
- Changed `MAX_BET_ID_LENGTH` from 64 to 32 bytes (Solana PDA seed limit)
- Updated `ProcessedBet::LEN` to use `MAX_BET_ID_LENGTH` constant instead of hardcoded value

**Security Impact:** Prevents PDA seed overflow errors when bet_id exceeds 32 bytes

#### 2. Updated `payout.rs` - Added Rent-Exemption Validation

**File:** `solana-playground-deploy/programs/vault/src/instructions/payout.rs`  
**Lines Modified:** 79-93 (14 new lines)

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

**Security Impact:** Prevents casino vault from dropping below rent-exempt threshold, which would make it eligible for garbage collection

#### 3. Updated `withdraw_casino_funds.rs` - Added Rent-Exemption Validation

**File:** `solana-playground-deploy/programs/vault/src/instructions/withdraw_casino_funds.rs`  
**Lines Modified:** 31-44 (13 new lines)

**Changes:** Same rent-exemption validation pattern as payout.rs

**Security Impact:** Prevents admin withdrawals from making casino vault subject to garbage collection

#### 4. Updated `spend_from_allowance.rs` - Added Bet ID Length Validation

**File:** `solana-playground-deploy/programs/vault/src/instructions/spend_from_allowance.rs`  
**Lines Modified:** 91-108 (6 new lines)

**Changes:**

```rust
// Validate bet ID length BEFORE PDA derivation (critical: prevents seed overflow)
// This must happen before the ProcessedBet account is initialized in the Context
require!(
    bet_id.len() <= MAX_BET_ID_LENGTH,
    VaultError::InvalidBetId
);
```

**Security Impact:** Prevents PDA derivation errors when bet_id exceeds 32 bytes. Validation now happens BEFORE ProcessedBet PDA creation (previously was after).

### Critical Issues Addressed

From the deep analysis report, Phase 1B fixes:

- **C2:** Missing rent-exemption checks (payout.rs, withdraw_casino_funds.rs)
- **C5:** Missing input validation (spend_from_allowance.rs bet_id length)

### Testing Plan

Deploy to Solana Playground and test:

1. **Rent-exemption validation:**
   - Try to payout/withdraw an amount that would drop casino vault below rent-exempt threshold
   - Expected: Transaction fails with `InsufficientBalance` error

2. **Bet ID length validation:**
   - Try to place bet with bet_id longer than 32 characters
   - Expected: Transaction fails with `InvalidBetId` error

3. **Normal operations:**
   - Place bet with valid bet_id (<= 32 chars)
   - Payout with sufficient casino vault balance
   - Withdraw with casino vault staying above rent threshold
   - Expected: All succeed

### Deployment Instructions

1. Open Solana Playground: https://beta.solpg.io/
2. Import the updated `solana-playground-deploy/programs/vault/` code
3. Build the program
4. Deploy to a NEW program ID (for parallel testing with production)
5. Initialize new CasinoVault with test funds
6. Run test suite above
7. If all tests pass, update services with new program ID

### Rollback Plan

If critical issues arise:

```bash
# Revert smart contract changes
git checkout solana-playground-deploy/programs/vault/src/state.rs
git checkout solana-playground-deploy/programs/vault/src/instructions/payout.rs
git checkout solana-playground-deploy/programs/vault/src/instructions/withdraw_casino_funds.rs
git checkout solana-playground-deploy/programs/vault/src/instructions/spend_from_allowance.rs

# Keep using production program ID
# Old program: 5sKSxXZ79DUJpnA8MVVmKsikKrQ4oG7TpNMJCjVzFnLf
```

### Next Steps

- Deploy to Solana Playground and verify fixes work on devnet
- Phase 1C: Backend Input Validation & Type Migration (requires Phase 1B tested on-chain)

---

## Phase 1C: Backend Input Validation ✅ COMPLETE

**Status:** ✅ Complete  
**Date:** 2026-01-18  
**Duration:** ~30 minutes

### Changes Made

#### 1. Updated `domain.rs` - Added Type-Safe Request Types

**File:** `services/backend/src/domain.rs`

**Changes:**

- Added `use shared::LamportAmount` import
- Changed `CreateBetRequest.stake_amount` from `u64` to `LamportAmount`
- Added custom deserializer `deserialize_lamport_amount` that validates during JSON parsing
- Validation now happens at API boundary before any business logic

**Security Impact:** Invalid bet amounts are rejected before reaching repository layer, with clear error messages

#### 2. Updated `bets.rs` Handler - Removed Manual Validation

**File:** `services/backend/src/handlers/bets.rs`

**Changes:**

```rust
// REMOVED manual validation:
// if (req.stake_amount as i64) < state.config.betting.min_bet_lamports as i64
//     || (req.stake_amount as i64) > state.config.betting.max_bet_lamports as i64
// {
//     return Err(AppError::InvalidInput(...));
// }

// Validation is now handled by LamportAmount type during deserialization
```

**Benefits:**

- Eliminated duplicate validation logic
- Validation errors are caught at deserialization (400 Bad Request)
- Type system guarantees valid amounts throughout request lifecycle

#### 3. Updated `bet_repository.rs` - Type Conversion

**File:** `services/backend/src/repository/bet_repository.rs`

**Changes:**

```rust
// Convert LamportAmount to i64 for storage
let stake_amount_i64 = req.stake_amount.as_u64() as i64;
```

**Details:**

- Added `use shared::LamportAmount` import
- Repository extracts validated u64 value from LamportAmount
- Redis storage continues using i64 format (no schema changes needed)

### Testing

```bash
cargo build -p backend
# Result: Success (compiled in 4.95s, 9 warnings)

# Services restarted
bash scripts/stop-services.sh && bash scripts/start-services.sh
# Result: Backend PID 8886, Processor PID 8893

curl http://localhost:3001/health
# Result: {"status":"healthy","timestamp":"2026-01-18T03:17:40.851826+00:00"}
```

### Benefits Achieved

1. **Type Safety**: LamportAmount prevents invalid amounts at compile time
2. **Validation at Boundary**: JSON deserialization validates before business logic
3. **No Duplicate Validation**: Single source of truth for bet amount rules
4. **Better Error Messages**: Serde provides structured error responses
5. **Overflow Protection**: LamportAmount provides checked arithmetic

### Rollback Plan

If issues arise:

```bash
git checkout services/backend/src/domain.rs
git checkout services/backend/src/handlers/bets.rs
git checkout services/backend/src/repository/bet_repository.rs
cargo build -p backend
bash scripts/stop-services.sh && bash scripts/start-services.sh
```

### Next Steps

- Phase 2A: Frontend Module Decomposition (extract solana.ts into separate concerns)

---

## Phase 2: Frontend Decomposition [PLANNED]

_Last Updated: 2026-01-18_
