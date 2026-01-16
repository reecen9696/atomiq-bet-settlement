use anchor_lang::prelude::*;
use crate::errors::*;
use crate::state::*;
use crate::validation::validate_allowance_params;

#[derive(Accounts)]
#[instruction(amount: u64, duration_seconds: i64, token_mint: Pubkey, nonce: u64)]
pub struct ApproveAllowanceV2<'info> {
    #[account(
        mut,
        seeds = [b"vault", casino.key().as_ref(), user.key().as_ref()],
        bump = vault.bump,
        constraint = vault.owner == user.key()
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        seeds = [b"casino"],
        bump = casino.bump,
        constraint = !casino.paused @ VaultError::CasinoPaused
    )]
    pub casino: Account<'info, Casino>,

    /// Stores the next nonce for deterministic allowance PDAs.
    #[account(
        init_if_needed,
        payer = user,
        space = AllowanceNonceRegistry::LEN,
        seeds = [b"allowance-nonce", user.key().as_ref(), casino.key().as_ref()],
        bump
    )]
    pub allowance_nonce_registry: Account<'info, AllowanceNonceRegistry>,

    #[account(
        init,
        payer = user,
        space = Allowance::LEN,
        seeds = [
            b"allowance",
            user.key().as_ref(),
            casino.key().as_ref(),
            &nonce.to_le_bytes()
        ],
        bump
    )]
    pub allowance: Account<'info, Allowance>,

    /// Rate limiter account (optional, created if doesn't exist)
    #[account(
        init_if_needed,
        payer = user,
        space = RateLimiter::LEN,
        seeds = [b"rate-limiter", user.key().as_ref()],
        bump
    )]
    pub rate_limiter: Account<'info, RateLimiter>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ApproveAllowanceV2>,
    amount: u64,
    duration_seconds: i64,
    token_mint: Pubkey,
    nonce: u64,
) -> Result<()> {
    let allowance = &mut ctx.accounts.allowance;
    let nonce_registry = &mut ctx.accounts.allowance_nonce_registry;
    let rate_limiter = &mut ctx.accounts.rate_limiter;
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    // Validate parameters
    validate_allowance_params(amount, duration_seconds)?;

    // Initialize nonce registry if first use
    if nonce_registry.user == Pubkey::default() {
        nonce_registry.user = ctx.accounts.user.key();
        nonce_registry.casino = ctx.accounts.casino.key();
        nonce_registry.next_nonce = 0;
        nonce_registry.bump = ctx.bumps.allowance_nonce_registry;
    }

    require!(nonce_registry.user == ctx.accounts.user.key(), VaultError::InvalidAllowanceNonce);
    require!(nonce_registry.casino == ctx.accounts.casino.key(), VaultError::InvalidAllowanceNonce);

    // Client must use the current next_nonce to derive the allowance PDA.
    require!(nonce == nonce_registry.next_nonce, VaultError::InvalidAllowanceNonce);

    // Rate limiter init
    if rate_limiter.window_start == 0 {
        rate_limiter.user = ctx.accounts.user.key();
        rate_limiter.window_start = clock.unix_timestamp;
        rate_limiter.approvals_count = 0;
        rate_limiter.bump = ctx.bumps.rate_limiter;
    }

    // Reset window if expired
    if clock.unix_timestamp - rate_limiter.window_start >= RateLimiter::WINDOW_DURATION {
        rate_limiter.window_start = clock.unix_timestamp;
        rate_limiter.approvals_count = 0;
    }

    require!(
        rate_limiter.approvals_count < RateLimiter::MAX_APPROVALS,
        VaultError::RateLimitExceeded
    );

    // Initialize allowance
    allowance.user = ctx.accounts.user.key();
    allowance.casino = ctx.accounts.casino.key();
    allowance.token_mint = token_mint;
    allowance.amount = amount;
    allowance.spent = 0;
    allowance.expires_at = clock.unix_timestamp + duration_seconds;
    allowance.created_at = clock.unix_timestamp;
    allowance.nonce = nonce;
    allowance.revoked = false;
    allowance.bump = ctx.bumps.allowance;
    allowance.last_spent_at = 0;
    allowance.spend_count = 0;

    // Increment nonce + rate limiter
    nonce_registry.next_nonce = nonce_registry
        .next_nonce
        .checked_add(1)
        .ok_or(VaultError::ArithmeticOverflow)?;
    rate_limiter.approvals_count = rate_limiter.approvals_count.saturating_add(1);

    // Update vault activity
    vault.last_activity = clock.unix_timestamp;

    msg!(
        "Allowance approved (nonce={}): {} tokens until {}",
        nonce,
        amount,
        allowance.expires_at
    );

    Ok(())
}
