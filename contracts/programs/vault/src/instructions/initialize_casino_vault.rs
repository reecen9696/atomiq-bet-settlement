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

    /// Casino vault - program-owned account holding casino funds
    #[account(
        init,
        payer = authority,
        space = CasinoVault::LEN,
        seeds = [b"casino-vault", casino.key().as_ref()],
        bump
    )]
    pub casino_vault: Account<'info, CasinoVault>,

    /// Vault authority PDA (used for signing SPL token transfers)
    #[account(
        seeds = [b"vault-authority", casino.key().as_ref()],
        bump
    )]
    /// CHECK: This is a PDA used only for signing SPL transfers
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeCasinoVault>, authority: Pubkey) -> Result<()> {
    let casino = &mut ctx.accounts.casino;
    let casino_vault = &mut ctx.accounts.casino_vault;
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

    casino_vault.casino = casino.key();
    casino_vault.bump = ctx.bumps.casino_vault;
    casino_vault.sol_balance = 0;
    casino_vault.created_at = clock.unix_timestamp;
    casino_vault.last_activity = clock.unix_timestamp;

    msg!("Casino initialized with authority: {}", authority);
    msg!("Casino vault initialized: {}", ctx.accounts.casino_vault.key());

    Ok(())
}
