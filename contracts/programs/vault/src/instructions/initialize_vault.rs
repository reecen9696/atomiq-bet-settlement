use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = user,
        space = Vault::LEN,
        seeds = [b"vault", casino.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        seeds = [b"casino"],
        bump = casino.bump
    )]
    pub casino: Account<'info, Casino>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeVault>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    vault.owner = ctx.accounts.user.key();
    vault.casino = ctx.accounts.casino.key();
    vault.bump = ctx.bumps.vault;
    vault.sol_balance = 0;
    vault.created_at = clock.unix_timestamp;
    vault.last_activity = clock.unix_timestamp;

    msg!("Vault initialized for user: {}", ctx.accounts.user.key());

    Ok(())
}
