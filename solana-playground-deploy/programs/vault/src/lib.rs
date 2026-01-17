use anchor_lang::prelude::*;

declare_id!("HTg6Cs11FNiRXjQ2wFiQodKrVuTQdEJYk8j4RtfX56rP");

pub mod state;
pub mod instructions;
pub mod errors;
pub mod validation;

// Solana Playground/Anchor macro compatibility:
// Anchor's #[program] macro expects certain generated `__client_accounts_*` items
// to be resolvable from the crate root. Our Accounts structs live under
// `crate::instructions::*`, so re-export them here.
pub use crate::instructions::*;

use crate::instructions::approve_allowance::ApproveAllowance;
use crate::instructions::approve_allowance_v2::ApproveAllowanceV2;
use crate::instructions::deposit_sol::DepositSol;
use crate::instructions::deposit_spl::DepositSpl;
use crate::instructions::initialize_casino_vault::InitializeCasinoVault;
use crate::instructions::initialize_vault::InitializeVault;
use crate::instructions::initialize_vault_only::InitializeVaultOnly;
use crate::instructions::reconcile_casino_vault::ReconcileCasinoVault;
use crate::instructions::pause_casino::{PauseCasino, UnpauseCasino};
use crate::instructions::payout::Payout;
use crate::instructions::revoke_allowance::RevokeAllowance;
use crate::instructions::spend_from_allowance::SpendFromAllowance;
use crate::instructions::withdraw_sol::WithdrawSol;
use crate::instructions::withdraw_spl::WithdrawSpl;
use crate::instructions::withdraw_casino_funds::WithdrawCasinoFunds;

#[program]
pub mod vault {
    use super::*;

    /// Initialize a user vault (PDA derived from user pubkey)
    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        instructions::initialize_vault::handler(ctx)
    }

    /// Initialize the casino vault (admin only, one-time setup)
    pub fn initialize_casino_vault(
        ctx: Context<InitializeCasinoVault>,
        authority: Pubkey,
    ) -> Result<()> {
        instructions::initialize_casino_vault::handler(ctx, authority)
    }

    /// Initialize just the casino vault for an existing casino
    pub fn initialize_vault_only(ctx: Context<InitializeVaultOnly>) -> Result<()> {
        instructions::initialize_vault_only::handler(ctx)
    }

    /// Reconcile casino vault balance (admin only - syncs tracked balance with actual lamports)
    pub fn reconcile_casino_vault(ctx: Context<ReconcileCasinoVault>) -> Result<()> {
        instructions::reconcile_casino_vault::handler(ctx)
    }

    /// Deposit SOL into vault
    pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
        instructions::deposit_sol::handler(ctx, amount)
    }

    /// Deposit SPL tokens (USDC) into vault
    pub fn deposit_spl(ctx: Context<DepositSpl>, amount: u64) -> Result<()> {
        instructions::deposit_spl::handler(ctx, amount)
    }

    /// Approve spending allowance (one-time approval for multiple bets)
    pub fn approve_allowance(
        ctx: Context<ApproveAllowance>,
        amount: u64,
        duration_seconds: i64,
        token_mint: Pubkey,
    ) -> Result<()> {
        instructions::approve_allowance::handler(ctx, amount, duration_seconds, token_mint)
    }

    /// Approve spending allowance (nonce-based PDA; deterministic for clients)
    pub fn approve_allowance_v2(
        ctx: Context<ApproveAllowanceV2>,
        amount: u64,
        duration_seconds: i64,
        token_mint: Pubkey,
        nonce: u64,
    ) -> Result<()> {
        instructions::approve_allowance_v2::handler(ctx, amount, duration_seconds, token_mint, nonce)
    }

    /// Revoke an active allowance
    pub fn revoke_allowance(ctx: Context<RevokeAllowance>) -> Result<()> {
        instructions::revoke_allowance::handler(ctx)
    }

    /// Spend from allowance (called by processor, no user signature needed)
    pub fn spend_from_allowance(
        ctx: Context<SpendFromAllowance>,
        amount: u64,
        bet_id: String,
    ) -> Result<()> {
        instructions::spend_from_allowance::handler(ctx, amount, bet_id)
    }

    /// Payout winnings from casino vault to user vault
    pub fn payout(
        ctx: Context<Payout>,
        amount: u64,
        bet_id: String,
    ) -> Result<()> {
        instructions::payout::handler(ctx, amount, bet_id)
    }

    /// Withdraw SOL from vault to user wallet (user only, always available)
    pub fn withdraw_sol(ctx: Context<WithdrawSol>, amount: u64) -> Result<()> {
        instructions::withdraw_sol::handler(ctx, amount)
    }

    /// Withdraw SPL tokens from vault to user wallet
    pub fn withdraw_spl(ctx: Context<WithdrawSpl>, amount: u64) -> Result<()> {
        instructions::withdraw_spl::handler(ctx, amount)
    }

    /// Emergency pause (admin only)
    pub fn pause_casino(ctx: Context<PauseCasino>) -> Result<()> {
        instructions::pause_casino::pause_handler(ctx)
    }

    /// Unpause (admin only)
    pub fn unpause_casino(ctx: Context<UnpauseCasino>) -> Result<()> {
        instructions::pause_casino::unpause_handler(ctx)
    }

    /// Withdraw funds from casino vault (admin only)
    pub fn withdraw_casino_funds(ctx: Context<WithdrawCasinoFunds>, amount: u64) -> Result<()> {
        instructions::withdraw_casino_funds::handler(ctx, amount)
    }
}
