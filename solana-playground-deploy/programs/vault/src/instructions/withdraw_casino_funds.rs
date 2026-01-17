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

    /// Casino vault authority (holds the funds)
    #[account(
        mut,
        seeds = [b"vault-authority", casino.key().as_ref()],
        bump = casino.vault_authority_bump
    )]
    /// CHECK: This is a PDA owned by the program
    pub vault_authority: UncheckedAccount<'info>,

    /// Casino authority (must sign)
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<WithdrawCasinoFunds>, amount: u64) -> Result<()> {
    // Direct lamports manipulation - vault authority is owned by our program
    **ctx.accounts.vault_authority.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += amount;

    msg!("Withdrew {} lamports from casino vault", amount);

    Ok(())
}
