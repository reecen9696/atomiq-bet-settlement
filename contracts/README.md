IMPORTANT - THIS COMPUTER CANNOT COMPILE/BUILD THE CONTRACT LOCALLY.
WE DEPLOY THE CONTRACT ON SOLANA PLAYGROUND

TELL THE USER WHEN WE NEED TO REDEPLOY THE CONTRACT

# Solana Playground Deployment (Devnet)

## What this folder is

This is a self-contained Anchor workspace intended to be uploaded/opened in Solana Playground for devnet deployment.

The Anchor program lives at:

- `programs/vault`

## Key behavior changes included

### 1) Deterministic allowance PDAs (fixes timestamp seed flakiness)

The legacy allowance PDA design used `Clock::unix_timestamp` inside the PDA seeds. In practice, slow wallet popups and RPC delays made this flaky because the client couldn’t reliably “guess” the exact timestamp the program would see at execution time.

This deploy folder includes the nonce-based replacement:

- New PDA: `AllowanceNonceRegistry` (seeds: `[b"allowance-nonce", user, casino]`) storing `next_nonce`
- New instruction: `approve_allowance_v2` creating allowances at seeds: `[b"allowance", user, casino, nonce]`

### 2) Backwards-compatible spend/revoke

`spend_from_allowance` and `revoke_allowance` validate the allowance PDA using `allowance.nonce`. This supports:

- Legacy allowances (where `nonce` contains the old timestamp)
- V2 allowances (where `nonce` is the deterministic u64 counter)

### 3) Wrapped SOL handling

The program also includes the earlier fix for wrapped SOL spending:

- Supports vault-owned token accounts OR user-owned accounts with delegation to the vault PDA
- Uses `MissingTokenDelegation` / `MissingTokenAccount` errors for clearer failures

## Deployment steps (Solana Playground)

1. Upload/open this folder as an Anchor workspace in Solana Playground.
2. Build + deploy to devnet.
3. Copy the deployed Program ID.
4. Update your clients:
   - UI: set `VITE_VAULT_PROGRAM_ID` to the new Program ID
   - Backend/processor: update the configured Program ID and restart

### Notes about Program ID

If you deploy a _new_ Program ID (most common in Playground), make sure the program ID your clients use matches the deployed program.

## Files you should see in the program

- `programs/vault/src/state.rs` includes `AllowanceNonceRegistry`
- `programs/vault/src/instructions/approve_allowance_v2.rs` exists and is exported from `programs/vault/src/instructions/mod.rs`
- `programs/vault/src/lib.rs` exposes `approve_allowance_v2`

If any of the above are missing in Playground, you likely didn’t upload the full folder.
