use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeCasinoVault<'info> {
    #[account(
        init,
        payer = authority,
        space = Casino::LEN,
        seeds = [b"casino"],
        bump
    )]
    pub casino: Account<'info, Casino>,

    /// Vault authority PDA (used for signing transfers from casino vault)
    #[account(
        seeds = [b"vault-authority", casino.key().as_ref()],
        bump
    )]
    /// CHECK: This is a PDA used only for signing
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeCasinoVault>, authority: Pubkey) -> Result<()> {
    let casino = &mut ctx.accounts.casino;
    let clock = Clock::get()?;

    casino.authority = authority;
    casino.processor = authority; // Initially set to authority, can be updated
    casino.treasury = authority;
    casino.bump = ctx.bumps.casino;
    casino.vault_authority_bump = ctx.bumps.vault_authority;
    casino.paused = false;
    casino.total_bets = 0;
    casino.total_volume = 0;
    casino.created_at = clock.unix_timestamp;

    msg!("Casino vault initialized with authority: {}", authority);

    Ok(())
}
