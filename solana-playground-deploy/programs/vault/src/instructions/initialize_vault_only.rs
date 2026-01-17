use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeVaultOnly<'info> {
    #[account(
        seeds = [b"casino"],
        bump = casino.bump
    )]
    pub casino: Account<'info, Casino>,

    /// Casino vault - program-owned account holding casino funds
    #[account(
        init,
        payer = authority,
        space = CasinoVault::LEN,
        seeds = [b"casino-vault", casino.key().as_ref()],
        bump
    )]
    pub casino_vault: Account<'info, CasinoVault>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeVaultOnly>) -> Result<()> {
    let casino_vault = &mut ctx.accounts.casino_vault;
    let clock = Clock::get()?;

    casino_vault.casino = ctx.accounts.casino.key();
    casino_vault.bump = ctx.bumps.casino_vault;
    casino_vault.sol_balance = 0;
    casino_vault.created_at = clock.unix_timestamp;
    casino_vault.last_activity = clock.unix_timestamp;

    msg!("Casino vault initialized: {}", ctx.accounts.casino_vault.key());

    Ok(())
}
