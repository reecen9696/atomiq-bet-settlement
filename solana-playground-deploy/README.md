# Solana Playground Deployment Instructions

## Overview
This folder contains the fixed smart contract for the Atomik Wallet betting system. The main fix addresses the `InvalidTokenAccountOwner` error that was preventing wrapped SOL betting from working.

## Key Changes Made

### 1. Fixed spend_from_allowance.rs
- **Problem**: Line 145 had `require!(user_token.owner == vault.key())` which failed when users owned their own token accounts
- **Solution**: Added support for multiple ownership patterns:
  - User-owned accounts with delegation to vault
  - Vault-owned accounts (original working pattern)
  - Proper error handling for missing delegation

### 2. Clear Token Type Separation
- **Native SOL**: `System::id()` - direct vault to casino SOL transfer
- **Wrapped SOL**: `So11111111111111111111111111111111111111112` - SPL token transfer
- **Other SPL tokens**: Any other mint - SPL token transfer with delegation

### 3. New Error Types Added
- `MissingTokenDelegation`: When user owns token account but hasn't delegated to vault
- `MissingTokenAccount`: When required token accounts are missing

## Deployment Steps

1. **Upload to Solana Playground**:
   - Compress this entire folder
   - Upload to https://beta.solana.com/
   - Or copy all files manually

2. **Build and Deploy**:
   ```bash
   anchor build
   anchor deploy --provider.cluster devnet
   ```

3. **Note the New Program ID**:
   - After deployment, copy the new Program ID
   - Update backend services with the new Program ID

4. **Update Backend Configuration**:
   - Replace `HoWjrEKiWKjEvqtdMDAHS9PEwkHQbVp2t6vYuDv3mdi4` with new Program ID
   - Restart all services (backend, processor)

## Testing Priority

After deployment, test in this order:
1. **Native SOL betting** (primary focus) ✅
2. **Wrapped SOL betting** (should now work) ✅
3. **Other SPL tokens** (if needed) ✅

## Expected Behavior

- **Native SOL**: Works as before, vault transfers SOL directly to casino
- **Wrapped SOL**: Now works with user-owned token accounts (with proper delegation)
- **Delegation**: Users can delegate token spending authority to their vault PDA
- **Error Messages**: Clear distinction between missing accounts vs missing delegation

## Files Changed
- `programs/vault/src/instructions/spend_from_allowance.rs` - Main fix
- `programs/vault/src/errors.rs` - Added new error types

The smart contract now properly supports the betting flow with both native SOL and wrapped SOL tokens!