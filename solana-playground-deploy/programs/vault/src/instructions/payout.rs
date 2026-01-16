use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::*;
use crate::validation::{validate_bet_id, CheckedMath};

#[derive(Accounts)]
#[instruction(amount: u64, bet_id: String)]
pub struct Payout<'info> {
    #[account(
        mut,
        seeds = [b"vault", casino.key().as_ref(), vault.owner.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [b"casino"],
        bump = casino.bump,
        constraint = !casino.paused @ VaultError::CasinoPaused
    )]
    pub casino: Account<'info, Casino>,

    /// Casino vault (source of payout)
    #[account(mut)]
    /// CHECK: Casino vault PDA or token account
    pub casino_vault: UncheckedAccount<'info>,

    /// Vault authority PDA (for signing casino vault transfers)
    #[account(
        seeds = [b"vault-authority", casino.key().as_ref()],
        bump = casino.vault_authority_bump
    )]
    /// CHECK: This is a PDA used for signing
    pub vault_authority: UncheckedAccount<'info>,

    /// Optional: User's token account (for SPL payout)
    #[account(mut)]
    pub user_token_account: Option<Account<'info, TokenAccount>>,

    /// Optional: Casino's token account (for SPL payout)
    #[account(mut)]
    pub casino_token_account: Option<Account<'info, TokenAccount>>,

    /// Reference to processed bet
    #[account(
        seeds = [b"processed-bet", bet_id.as_bytes()],
        bump = processed_bet.bump,
        constraint = processed_bet.user == vault.owner
    )]
    pub processed_bet: Account<'info, ProcessedBet>,

    /// Processor (authorized to execute payouts)
    #[account(
        constraint = processor.key() == casino.processor @ VaultError::UnauthorizedProcessor
    )]
    pub processor: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Option<Program<'info, Token>>,
}

pub fn handler(
    ctx: Context<Payout>,
    amount: u64,
    bet_id: String,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let casino = &ctx.accounts.casino;
    let clock = Clock::get()?;

    validate_bet_id(&bet_id)?;

    // Determine if SOL or SPL payout
    let is_sol = ctx.accounts.user_token_account.is_none();

    if is_sol {
        // SOL payout: casino_vault -> user vault
        let casino_key = casino.key();
        let seeds = &[
            b"vault-authority",
            casino_key.as_ref(),
            &[casino.vault_authority_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.casino_vault.to_account_info(),
                    to: vault.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        // Update vault balance
        vault.sol_balance = vault.sol_balance.safe_add(amount)?;
    } else {
        // SPL payout: casino_token_account -> user_token_account
        let user_token = ctx.accounts.user_token_account.as_ref()
            .ok_or(VaultError::InvalidTokenAccountOwner)?;
        let casino_token = ctx.accounts.casino_token_account.as_ref()
            .ok_or(VaultError::InvalidTokenAccountOwner)?;

        let casino_key = casino.key();
        let seeds = &[
            b"vault-authority",
            casino_key.as_ref(),
            &[casino.vault_authority_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.as_ref().unwrap().to_account_info(),
                Transfer {
                    from: casino_token.to_account_info(),
                    to: user_token.to_account_info(),
                    authority: ctx.accounts.vault_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
    }

    // Update vault activity
    vault.last_activity = clock.unix_timestamp;

    msg!("Payout {} for bet {}", amount, bet_id);

    Ok(())
}
