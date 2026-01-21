use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct PauseCasino<'info> {
    #[account(
        mut,
        seeds = [b"casino"],
        bump = casino.bump,
        constraint = casino.authority == authority.key() @ VaultError::UnauthorizedAuthority
    )]
    pub casino: Account<'info, Casino>,

    pub authority: Signer<'info>,
}

pub fn pause_handler(ctx: Context<PauseCasino>) -> Result<()> {
    let casino = &mut ctx.accounts.casino;
    casino.paused = true;

    msg!("Casino paused by authority");

    Ok(())
}

#[derive(Accounts)]
pub struct UnpauseCasino<'info> {
    #[account(
        mut,
        seeds = [b"casino"],
        bump = casino.bump,
        constraint = casino.authority == authority.key() @ VaultError::UnauthorizedAuthority
    )]
    pub casino: Account<'info, Casino>,

    pub authority: Signer<'info>,
}

pub fn unpause_handler(ctx: Context<UnpauseCasino>) -> Result<()> {
    let casino = &mut ctx.accounts.casino;
    casino.paused = false;

    msg!("Casino unpaused by authority");

    Ok(())
}
