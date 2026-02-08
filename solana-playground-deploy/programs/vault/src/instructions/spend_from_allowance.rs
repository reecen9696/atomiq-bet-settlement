use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::*;
use crate::validation::{validate_bet_amount, validate_bet_id, CheckedMath};

// Wrapped SOL mint address
const WRAPPED_SOL_MINT: Pubkey = solana_program::pubkey!("So11111111111111111111111111111111111111112");

#[derive(Accounts)]
#[instruction(amount: u64, bet_id: String)]
pub struct SpendFromAllowance<'info> {
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

    #[account(
        mut,
        seeds = [
            b"allowance",
            allowance.user.as_ref(),
            casino.key().as_ref(),
            &allowance.nonce.to_le_bytes()
        ],
        bump = allowance.bump,
        constraint = allowance.user == vault.owner @ VaultError::InvalidAllowancePDA
    )]
    pub allowance: Account<'info, Allowance>,

    /// Processed bet tracker (prevents double-spend)
    #[account(
        init,
        payer = processor,
        space = ProcessedBet::LEN,
        seeds = [b"processed-bet", bet_id.as_bytes()],
        bump
    )]
    pub processed_bet: Account<'info, ProcessedBet>,

    /// Casino vault (for SOL) - program-owned account holding casino funds
    #[account(
        mut,
        seeds = [b"casino-vault", casino.key().as_ref()],
        bump = casino_vault.bump
    )]
    pub casino_vault: Account<'info, CasinoVault>,

    /// Vault authority PDA (for signing SPL token transfers)
    #[account(
        seeds = [b"vault-authority", casino.key().as_ref()],
        bump = casino.vault_authority_bump
    )]
    /// CHECK: This is a PDA used only for signing SPL transfers
    pub vault_authority: UncheckedAccount<'info>,

    /// Optional: User's token account (for SPL)
    #[account(mut)]
    pub user_token_account: Option<Account<'info, TokenAccount>>,

    /// Optional: Casino's token account (for SPL)
    #[account(mut)]
    pub casino_token_account: Option<Account<'info, TokenAccount>>,

    /// Processor (authorized to execute spends)
    #[account(
        mut,
        constraint = processor.key() == casino.processor @ VaultError::UnauthorizedProcessor
    )]
    pub processor: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Option<Program<'info, Token>>,
}

pub fn handler(
    ctx: Context<SpendFromAllowance>,
    amount: u64,
    bet_id: String,
) -> Result<()> {
    let allowance = &mut ctx.accounts.allowance;
    let vault = &mut ctx.accounts.vault;
    let casino = &mut ctx.accounts.casino;
    let processed_bet = &mut ctx.accounts.processed_bet;
    let clock = Clock::get()?;

    // Validate bet amount
    validate_bet_amount(amount)?;

    // Validate bet ID length BEFORE PDA derivation (critical: prevents seed overflow)
    // This must happen before the ProcessedBet account is initialized in the Context
    require!(
        bet_id.len() <= MAX_BET_ID_LENGTH,
        VaultError::InvalidBetId
    );

    // Validate bet ID format
    validate_bet_id(&bet_id)?;

    // Check allowance is valid
    require!(allowance.is_valid(&clock), VaultError::AllowanceExpired);

    // Check sufficient allowance remaining
    let new_spent = allowance.spent.safe_add(amount)?;
    require!(
        new_spent <= allowance.amount,
        VaultError::InsufficientAllowance
    );

    // Handle different token types with clear separation
    if allowance.token_mint == System::id() {
        // NATIVE SOL: vault -> casino_vault
        require!(vault.sol_balance >= amount, VaultError::InsufficientBalance);

        // Direct lamports manipulation - required for accounts with data
        // The System Program's transfer instruction cannot be used on accounts with data,
        // so we manipulate the lamports directly using Anchor's account methods
        **vault.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.casino_vault.to_account_info().try_borrow_mut_lamports()? += amount;

        vault.sol_balance = vault.sol_balance.safe_sub(amount)?;
        ctx.accounts.casino_vault.sol_balance = ctx.accounts.casino_vault.sol_balance.safe_add(amount)?;
        ctx.accounts.casino_vault.last_activity = clock.unix_timestamp;
        msg!("Native SOL transfer: {} lamports from vault to casino", amount);
    } else if allowance.token_mint == WRAPPED_SOL_MINT {
        // WRAPPED SOL: user_token_account -> casino_token_account (SPL transfer)
        let user_token = ctx
            .accounts
            .user_token_account
            .as_ref()
            .ok_or(VaultError::MissingTokenAccount)?;
        let casino_token = ctx
            .accounts
            .casino_token_account
            .as_ref()
            .ok_or(VaultError::MissingTokenAccount)?;

        require!(user_token.mint == WRAPPED_SOL_MINT, VaultError::TokenMintMismatch);
        require!(casino_token.mint == WRAPPED_SOL_MINT, VaultError::TokenMintMismatch);

        let has_delegation = user_token.delegate.is_some()
            && user_token.delegate.unwrap() == vault.key()
            && user_token.delegated_amount >= amount;

        let vault_owned = user_token.owner == vault.key();
        let user_owned = user_token.owner == vault.owner;

        require!(has_delegation || vault_owned, VaultError::InvalidTokenAccountOwner);
        if user_owned && !has_delegation {
            return Err(VaultError::MissingTokenDelegation.into());
        }

        let casino_key = casino.key();
        let seeds = &[
            b"vault",
            casino_key.as_ref(),
            vault.owner.as_ref(),
            &[vault.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts
                    .token_program
                    .as_ref()
                    .ok_or(VaultError::MissingTokenProgram)?
                    .to_account_info(),
                Transfer {
                    from: user_token.to_account_info(),
                    to: casino_token.to_account_info(),
                    authority: vault.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        msg!(
            "Wrapped SOL transfer: {} lamports from user token account to casino",
            amount
        );
    } else {
        // OTHER SPL TOKENS: user_token_account -> casino_token_account
        let user_token = ctx
            .accounts
            .user_token_account
            .as_ref()
            .ok_or(VaultError::MissingTokenAccount)?;
        let casino_token = ctx
            .accounts
            .casino_token_account
            .as_ref()
            .ok_or(VaultError::MissingTokenAccount)?;

        require!(user_token.mint == allowance.token_mint, VaultError::TokenMintMismatch);
        require!(casino_token.mint == allowance.token_mint, VaultError::TokenMintMismatch);

        let has_delegation = user_token.delegate.is_some()
            && user_token.delegate.unwrap() == vault.key()
            && user_token.delegated_amount >= amount;
        let vault_owned = user_token.owner == vault.key();

        require!(has_delegation || vault_owned, VaultError::InvalidTokenAccountOwner);

        let casino_key = casino.key();
        let seeds = &[
            b"vault",
            casino_key.as_ref(),
            vault.owner.as_ref(),
            &[vault.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts
                    .token_program
                    .as_ref()
                    .ok_or(VaultError::MissingTokenProgram)?
                    .to_account_info(),
                Transfer {
                    from: user_token.to_account_info(),
                    to: casino_token.to_account_info(),
                    authority: vault.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        msg!(
            "SPL token transfer: {} units of {} from user to casino",
            amount,
            allowance.token_mint
        );
    }

    // Update allowance
    allowance.spent = new_spent;
    allowance.last_spent_at = clock.unix_timestamp;
    allowance.spend_count = allowance.spend_count.saturating_add(1);

    // Update vault activity
    vault.last_activity = clock.unix_timestamp;

    // Update casino stats
    casino.total_bets = casino.total_bets.safe_add(1)?;
    casino.total_volume = casino.total_volume.safe_add(amount)?;

    // Record processed bet
    processed_bet.bet_id = bet_id.clone();
    processed_bet.user = vault.owner;
    processed_bet.amount = amount;
    processed_bet.processed_at = clock.unix_timestamp;
    processed_bet.signature = String::new(); // Will be filled by backend
    processed_bet.bump = ctx.bumps.processed_bet;

    msg!("Bet {} processed: {} spent from allowance", bet_id, amount);

    Ok(())
}
