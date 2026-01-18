use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

/// Withdraw funds from casino vault (admin only)
#[derive(Accounts)]
pub struct WithdrawCasinoFunds<'info> {
    #[account(
        seeds = [b"casino"],
        bump = casino.bump,
        constraint = casino.authority == authority.key() @ VaultError::UnauthorizedAuthority
    )]
    pub casino: Account<'info, Casino>,

    /// Casino vault - program-owned account holding casino funds
    #[account(
        mut,
        seeds = [b"casino-vault", casino.key().as_ref()],
        bump = casino_vault.bump
    )]
    pub casino_vault: Account<'info, CasinoVault>,

    /// Casino authority (must sign)
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WithdrawCasinoFunds>, amount: u64) -> Result<()> {
    let casino_vault = &mut ctx.accounts.casino_vault;
    let clock = Clock::get()?;

    // Balance check with reconciliation
    require!(
        casino_vault.sol_balance >= amount,
        VaultError::InsufficientBalance
    );

    // Direct lamports manipulation - casino vault is program-owned
    **casino_vault.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += amount;

    // Update tracked balance
    casino_vault.sol_balance = casino_vault.sol_balance.saturating_sub(amount);
    casino_vault.last_activity = clock.unix_timestamp;

    msg!("Withdrew {} lamports from casino vault", amount);

    Ok(())
}
