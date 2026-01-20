# 🚀 Deployment Instructions - Fixed Program

## What Was Fixed

**Root Cause**: The "Transfer: `from` must not carry data" error occurs when trying to use Solana's System Program `transfer()` instruction on accounts that have data (like your Vault PDA with 97 bytes of account data).

**Solution**: Replaced all `system_program::transfer()` and `invoke_signed(&system_instruction::transfer(...))` calls with **direct lamports manipulation** using `try_borrow_mut_lamports()`.

### Files Modified:

1. ✅ **spend_from_allowance.rs** - Fixed native SOL bet processing
2. ✅ **withdraw_sol.rs** - Fixed SOL withdrawals from vault
3. ✅ **payout.rs** - Fixed payout transfers to vault

## Deployment Steps

### Option 1: Solana Playground (Easiest)

1. **Copy your code to Solana Playground**
   - Go to https://beta.solpg.io
   - Create a new Anchor project
   - Copy all files from `solana-playground-deploy/programs/vault/src/` to the playground

2. **Build the program**
   - Click "Build" button (or Ctrl+S)
   - Wait for successful build
   - Verify no errors

3. **Upgrade the deployed program**
   ```bash
   # In Solana Playground terminal:
   solana program deploy \
     --program-id HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP \
     --upgrade-authority <YOUR_WALLET> \
     --url devnet \
     ./target/deploy/vault.so
   ```

### Option 2: Local Build with Anchor CLI

1. **Navigate to the project**

   ```bash
   cd /Users/reece/code/projects/atomik-wallet/solana-playground-deploy
   ```

2. **Build the program**

   ```bash
   anchor build
   ```

3. **Verify the build**

   ```bash
   ls -lh target/deploy/vault.so
   # Should see a .so file around 400-600KB
   ```

4. **Deploy/Upgrade to devnet**

   ```bash
   solana program deploy \
     --program-id HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP \
     --upgrade-authority ~/.config/solana/id.json \
     --url devnet \
     target/deploy/vault.so
   ```

   Or if using a specific keypair:

   ```bash
   solana program deploy \
     --program-id HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP \
     --upgrade-authority /path/to/your/keypair.json \
     --url devnet \
     target/deploy/vault.so
   ```

### Important Notes:

- **Upgrade Authority**: The wallet with address `FwbvNxgJXy7bmxVYcGfGYkQDYhPvMp3ppysvW8VeckE7`
- **Program ID**: `HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP` (DO NOT CHANGE)
- **Network**: Solana Devnet
- **Cost**: Upgrading an existing program costs minimal SOL (~0.001 SOL)

## Verification

After deployment, verify the upgrade:

```bash
# Check program details
solana program show HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP --url devnet
```

Look for:

- ✅ "Last Deployed In Slot" should be a recent slot number
- ✅ "Authority" should be your wallet address

## Testing the Fix

Once deployed, test with the CLI script:

```bash
cd /Users/reece/code/projects/atomik-wallet

# Place a real bet
node place-real-bet.js
```

Expected output:

```
✅ Bet created: <BET_ID>
Status: completed
Won: true/false
Solana TX: <ACTUAL_TX_SIGNATURE> (NOT SIM_)
```

## What Changed (Technical Details)

### Before (BROKEN):

```rust
// This fails on accounts with data ❌
system_program::transfer(
    CpiContext::new_with_signer(...),
    amount,
)?;
```

### After (FIXED):

```rust
// Direct lamports manipulation ✅
**from_account.to_account_info().try_borrow_mut_lamports()? -= amount;
**to_account.to_account_info().try_borrow_mut_lamports()? += amount;
```

### Why This Works:

1. **Bypasses System Program** - No CPI needed
2. **Works on any account** - With or without data
3. **Standard Solana pattern** - Used in escrow, vaults, staking programs
4. **Direct memory access** - Manipulates lamports field directly

## Common Errors & Solutions

### Error: "Invalid upgrade authority"

**Solution**: Make sure you're using the correct keypair that matches the upgrade authority

### Error: "Insufficient funds"

**Solution**: Ensure your wallet has at least 0.01 SOL for transaction fees

### Error: "Program file not found"

**Solution**: Run `anchor build` first to generate the .so file

## After Successful Deployment

1. **Restart your services**

   ```bash
   ./stop-services.sh
   ./start-services.sh
   ```

2. **Test end-to-end betting**

   ```bash
   # Create allowance
   node scripts/approve-allowance-cli.js test-user-keypair.json 1.0 86400

   # Place bet
   node place-real-bet.js
   ```

3. **Monitor processor logs**
   ```bash
   tail -f logs/processor.log | grep -E "confirmed|error"
   ```

## Success Criteria

✅ Program deploys without errors
✅ Processor logs show "confirmed" instead of "Transfer: from must not carry data"
✅ Bets complete with real transaction signatures (no SIM\_ prefix)
✅ Transactions visible on Solana Explorer

## Troubleshooting

If bets still fail after deployment:

1. **Check processor is using new program**

   ```bash
   grep VAULT_PROGRAM services/processor/.env
   # Should be: HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP
   ```

2. **Verify deployment slot**

   ```bash
   solana program show HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP --url devnet
   ```

3. **Check processor logs for new errors**
   ```bash
   tail -30 logs/processor.log
   ```

## References

- Solana Docs: [Accounts and Data](https://docs.solana.com/developing/programming-model/accounts)
- Anchor Book: [Working with Lamports](https://book.anchor-lang.com/)
- This fix follows the standard pattern used in:
  - Marinade Finance (liquid staking)
  - Jet Protocol (lending)
  - Magic Eden (NFT marketplace)

---

**Ready to deploy?** Choose Option 1 (Solana Playground) if you want the easiest path, or Option 2 (Local Build) if you prefer full control.
