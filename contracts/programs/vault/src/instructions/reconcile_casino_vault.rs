use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct ReconcileCasinoVault<'info> {
    #[account(
        seeds = [b"casino"],
        bump = casino.bump,
        constraint = authority.key() == casino.authority @ VaultError::UnauthorizedAuthority
    )]
    pub casino: Account<'info, Casino>,

    /// Casino vault - program-owned account holding casino funds
    #[account(
        mut,
        seeds = [b"casino-vault", casino.key().as_ref()],
        bump = casino_vault.bump
    )]
    pub casino_vault: Account<'info, CasinoVault>,

    /// Casino authority (admin)
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<ReconcileCasinoVault>) -> Result<()> {
    let casino_vault = &mut ctx.accounts.casino_vault;
    let clock = Clock::get()?;

    // Get actual lamports in the account (minus rent-exempt reserve)
    let account_lamports = casino_vault.to_account_info().lamports();
    
    // Calculate rent-exempt reserve (should be ~0.00134328 SOL for 65-byte account)
    let rent = Rent::get()?;
    let rent_exempt_reserve = rent.minimum_balance(CasinoVault::LEN);
    
    // Available balance = total lamports - rent reserve
    let available_balance = account_lamports.saturating_sub(rent_exempt_reserve);

    msg!(
        "Reconciling casino vault balance: tracked={}, actual_lamports={}, rent_reserve={}, available={}",
        casino_vault.sol_balance,
        account_lamports,
        rent_exempt_reserve,
        available_balance
    );

    // Update tracked balance to match actual available balance
    casino_vault.sol_balance = available_balance;
    casino_vault.last_activity = clock.unix_timestamp;

    msg!("Casino vault balance reconciled to {} lamports", available_balance);

    Ok(())
}
