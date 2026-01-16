use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::*;
use crate::validation::CheckedMath;

#[derive(Accounts)]
pub struct DepositSol<'info> {
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

pub fn handler(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    // Transfer SOL from user to vault PDA
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: vault.to_account_info(),
            },
        ),
        amount,
    )?;

    // Update vault balance with checked arithmetic
    vault.sol_balance = vault.sol_balance.safe_add(amount)?;
    vault.last_activity = clock.unix_timestamp;

    msg!("Deposited {} lamports to vault", amount);

    Ok(())
}
