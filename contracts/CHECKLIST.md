# Deployment Checklist

## Program source completeness (nonce-based allowances)
- [ ] `programs/vault/src/state.rs` contains `AllowanceNonceRegistry`
- [ ] `programs/vault/src/errors.rs` contains `InvalidAllowanceNonce`
- [ ] `programs/vault/src/instructions/approve_allowance_v2.rs` exists
- [ ] `programs/vault/src/instructions/mod.rs` exports `approve_allowance_v2`
- [ ] `programs/vault/src/lib.rs` exposes `approve_allowance_v2`

## Backwards compatibility checks
- [ ] `programs/vault/src/instructions/revoke_allowance.rs` validates allowance PDA using `allowance.nonce`
- [ ] `programs/vault/src/instructions/spend_from_allowance.rs` validates allowance PDA using `allowance.nonce`

## Wrapped SOL checks
- [ ] `MissingTokenDelegation` / `MissingTokenAccount` errors exist
- [ ] Wrapped SOL path supports user-owned token accounts with delegation

## Deployment steps (Playground / devnet)
- [ ] Upload/open this entire folder in Solana Playground
- [ ] Build + deploy to devnet
- [ ] Copy the Program ID
- [ ] Update `VITE_VAULT_PROGRAM_ID` in your UI
- [ ] Update backend/processor Program ID and restart

## Post-deploy sanity
- [ ] Initialize casino PDA once
- [ ] Initialize a user vault
- [ ] Deposit SOL
- [ ] Approve allowance (UI uses `approve_allowance_v2`)
- [ ] Place a bet and confirm the processor can spend from allowance