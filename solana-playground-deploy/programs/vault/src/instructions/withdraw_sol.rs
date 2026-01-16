use anchor_lang::prelude::*;
use anchor_lang::system_program;
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

    // Transfer SOL from vault to user using PDA signer
    let casino_key = ctx.accounts.casino.key();
    let user_key = ctx.accounts.user.key();
    let seeds = &[
        b"vault",
        casino_key.as_ref(),
        user_key.as_ref(),
        &[vault.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: vault.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    // Update vault balance
    vault.sol_balance = vault.sol_balance.safe_sub(amount)?;
    vault.last_activity = clock.unix_timestamp;

    msg!("Withdrew {} lamports from vault", amount);

    Ok(())
}
