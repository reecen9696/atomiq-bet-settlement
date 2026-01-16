use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::validation::validate_allowance_params;

#[derive(Accounts)]
#[instruction(amount: u64, duration_seconds: i64, token_mint: Pubkey)]
pub struct ApproveAllowance<'info> {
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

    #[account(
        init,
        payer = user,
        space = Allowance::LEN,
        seeds = [
            b"allowance",
            user.key().as_ref(),
            casino.key().as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
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
    ctx: Context<ApproveAllowance>,
    amount: u64,
    duration_seconds: i64,
    token_mint: Pubkey,
) -> Result<()> {
    let allowance = &mut ctx.accounts.allowance;
    let rate_limiter = &mut ctx.accounts.rate_limiter;
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    // Validate parameters
    validate_allowance_params(amount, duration_seconds)?;

    // Check rate limiting
    if rate_limiter.window_start == 0 {
        // First time initialization
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

    // Check rate limit
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
    allowance.nonce = clock.unix_timestamp as u64;
    allowance.revoked = false;
    allowance.bump = ctx.bumps.allowance;
    allowance.last_spent_at = 0;
    allowance.spend_count = 0;

    // Increment rate limiter
    rate_limiter.approvals_count += 1;

    // Update vault activity
    vault.last_activity = clock.unix_timestamp;

    msg!(
        "Allowance approved: {} tokens until {}",
        amount,
        allowance.expires_at
    );

    Ok(())
}
