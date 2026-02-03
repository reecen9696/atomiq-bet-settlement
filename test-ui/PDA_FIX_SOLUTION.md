# Dynamic Allowance PDA Fix: Complete Solution Documentation

## Problem Overview

The core issue was a **PDA (Program Derived Address) mismatch** between what the UI displayed and what the betting interface was using:

- **BettingInterface**: Used `localStorage.getItem("lastAllowancePda")` - a **stale/outdated** allowance PDA
- **VaultManager UI**: Displayed status of the **current active** allowance PDA
- **Result**: UI showed healthy allowance (4.26 SOL remaining, expires 03/02/2026) but bets failed with `AllowanceExpired` error

This caused immediate bet failures despite showing sufficient balance and valid expiration time.

## Root Cause Analysis

The Solana program uses a **nonce-based allowance system**:
- Each user can have multiple allowance PDAs, identified by incrementing nonce values
- PDAs are derived using seeds: `[b"allowance", user_pubkey, casino_pubkey, nonce_bytes]`
- The **AllowanceNonceRegistry** PDA tracks the next available nonce
- The **active allowance** is always at `(current_nonce - 1)`

The bug occurred because:
1. User created a new allowance (nonce = N)
2. localStorage stored this allowance PDA
3. User later created another allowance (nonce = N+1)
4. UI correctly showed the new allowance status
5. But BettingInterface still used the old localStorage PDA (nonce = N)
6. Bets failed because they used the wrong allowance

## Complete Solution Implementation

### 1. Dynamic Allowance PDA Derivation Service

Added `getCurrentAllowancePDA()` method to `src/services/solana.ts` (lines 1031-1060):

```typescript
async getCurrentAllowancePDA(params: {
  user: PublicKey;
  connection?: Connection;
  casinoPda?: string;
}): Promise<string | null> {
  try {
    const conn = params.connection ?? this.connection;
    const casinoPda = params.casinoPda ?? (await this.deriveCasinoPDA());
    
    // Get the current nonce from AllowanceNonceRegistry
    const currentNonce = await this.getNextAllowanceNonce({
      user: params.user,
      casinoPda,
      connection: conn,
    });
    
    // If no nonce (no allowances created yet), return null
    if (currentNonce === 0n) {
      return null;
    }
    
    // Derive the allowance PDA for the previous nonce (current active allowance)
    const activeNonce = currentNonce - 1n;
    const allowancePda = this.pdaDerivation.deriveAllowancePDA(
      params.user,
      activeNonce,
      new PublicKey(casinoPda)
    );
    
    return allowancePda.toBase58();
  } catch (error) {
    console.error('Error getting current allowance PDA:', error);
    return null;
  }
}
```

**How it works**:
1. **Fetches current nonce** from on-chain AllowanceNonceRegistry PDA
2. **Calculates active allowance** as `(currentNonce - 1)` 
3. **Derives correct PDA** using the active nonce
4. **Returns base58 string** for API calls

### 2. Updated Betting Interface

Modified `src/components/BettingInterface.tsx` (lines 47-58) to use dynamic lookup:

```typescript
// OLD CODE (caused the bug):
const allowancePda = localStorage.getItem("lastAllowancePda") || undefined;

// NEW CODE (fixes the bug):
const allowancePda = await solanaService.getCurrentAllowancePDA({
  user: publicKey,
  connection,
});

if (!allowancePda) {
  setError("No active allowance found. Please create an allowance first.");
  return;
}
```

**Key changes**:
- **Removed localStorage dependency** completely
- **Added real-time on-chain lookup** of current allowance
- **Added proper error handling** for missing allowances
- **Added required imports**: `useConnection` hook and `solanaService`

### 3. Technical Flow

The complete flow now works as follows:

```
1. User clicks "Place Bet"
2. BettingInterface.getCurrentAllowancePDA():
   a. Queries AllowanceNonceRegistry PDA on-chain
   b. Reads nextNonce value (e.g., 5n)
   c. Calculates activeNonce = nextNonce - 1n (e.g., 4n)
   d. Derives PDA with seeds [b"allowance", user, casino, 4n]
3. Uses this current PDA for bet transaction
4. UI and betting logic now use identical allowance PDA
5. No more PDA mismatch → No more AllowanceExpired errors
```

## What This Fixes

### Before (Broken):
- **BettingInterface**: Used stale localStorage PDA (nonce = N)
- **VaultManager**: Showed current PDA status (nonce = N+1) 
- **Result**: PDA mismatch → `AllowanceExpired` despite healthy balance

### After (Fixed):
- **BettingInterface**: Dynamically derives current PDA (nonce = N+1)
- **VaultManager**: Shows same PDA status (nonce = N+1)
- **Result**: PDA match → Successful bets with proper allowance

## Additional Benefits

1. **Eliminates race conditions**: No more 6-second gaps between successful/failed transactions
2. **Removes localStorage dependency**: No stale data issues
3. **Real-time accuracy**: Always uses the most current allowance
4. **Better error handling**: Clear messaging when no allowance exists
5. **Future-proof**: Works correctly as users create new allowances

## Files Modified

1. **src/services/solana.ts** - Added `getCurrentAllowancePDA()` method
2. **src/components/BettingInterface.tsx** - Replaced localStorage with dynamic lookup
3. **No compilation errors** - All imports and dependencies properly handled

## Testing Validation

The fix ensures:
- ✅ **BettingInterface** and **VaultManager** use identical allowance PDAs
- ✅ **No localStorage staleness** - always fetches current state
- ✅ **Proper error handling** for edge cases
- ✅ **Real-time accuracy** - reflects latest allowance creation/updates

This completely resolves the `AllowanceExpired` error issue while maintaining system reliability and user experience.

## Casino Vault Information

For reference, the correct casino vault addresses are:

- **Casino PDA**: `CC7X4n2uC3fahBxGt68zbWbGD6tYBBbKLZJeMTkmsE4B` (5 SOL)
- **Casino Vault PDA**: `6V7D1qieU9sHyjmj1Tb5N1kqBJTMs6Xjcgr7M4jcYTB6` (6.21 SOL) - **USE THIS FOR DEPOSITS**
- **Vault Authority PDA**: `GaeGjJKt9wroE5koU4aVyK8rJNq1Df7oiYu7RuzEaxbv` (4 SOL)

Deposit funds to the Casino Vault PDA for testing purposes.
