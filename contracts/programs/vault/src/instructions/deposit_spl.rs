use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::validation::validate_token_account;

#[derive(Accounts)]
pub struct DepositSpl<'info> {
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

    /// User's SPL token account
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    /// Vault's SPL token account (ATA owned by vault PDA)
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<DepositSpl>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    // Validate token accounts
    validate_token_account(
        &ctx.accounts.user_token_account,
        &ctx.accounts.user.key(),
        &ctx.accounts.user_token_account.mint,
    )?;

    validate_token_account(
        &ctx.accounts.vault_token_account,
        &vault.key(),
        &ctx.accounts.user_token_account.mint,
    )?;

    // Transfer SPL tokens from user to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    vault.last_activity = clock.unix_timestamp;

    msg!("Deposited {} tokens to vault", amount);

    Ok(())
}
