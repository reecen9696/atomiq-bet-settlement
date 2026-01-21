use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::validation::validate_token_account;

#[derive(Accounts)]
pub struct WithdrawSpl<'info> {
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

    /// Vault's SPL token account
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// User's SPL token account (destination)
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<WithdrawSpl>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    // Validate token accounts
    validate_token_account(
        &ctx.accounts.vault_token_account,
        &vault.key(),
        &ctx.accounts.vault_token_account.mint,
    )?;

    validate_token_account(
        &ctx.accounts.user_token_account,
        &ctx.accounts.user.key(),
        &ctx.accounts.vault_token_account.mint,
    )?;

    // Transfer SPL tokens from vault to user using PDA signer
    let casino_key = ctx.accounts.casino.key();
    let user_key = ctx.accounts.user.key();
    let seeds = &[
        b"vault",
        casino_key.as_ref(),
        user_key.as_ref(),
        &[vault.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: vault.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    vault.last_activity = clock.unix_timestamp;

    msg!("Withdrew {} tokens from vault", amount);

    Ok(())
}
