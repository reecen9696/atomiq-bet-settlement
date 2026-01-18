use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::validation::CheckedMath;

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
    #[account(
        mut,
        seeds = [b"vault", casino.key().as_ref(), user.key().as_ref()],
        bump = vault.bump,
        constraint = vault.owner == user.key()
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

pub fn handler(ctx: Context<WithdrawSol>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    // Check sufficient balance
    require!(
        vault.sol_balance >= amount,
        VaultError::InsufficientBalance
    );

    // Direct lamports manipulation - required for accounts with data
    // The System Program's transfer instruction cannot be used on accounts with data
    **vault.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += amount;

    // Update vault balance
    vault.sol_balance = vault.sol_balance.safe_sub(amount)?;
    vault.last_activity = clock.unix_timestamp;

    msg!("Withdrew {} lamports from vault", amount);

    Ok(())
}
