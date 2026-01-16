use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Insufficient balance in vault")]
    InsufficientBalance,

    #[msg("Invalid bet amount: must be between MIN_BET and MAX_BET")]
    InvalidBetAmount,

    #[msg("Allowance has expired")]
    AllowanceExpired,

    #[msg("Allowance has been revoked")]
    AllowanceRevoked,

    #[msg("Insufficient allowance remaining")]
    InsufficientAllowance,

    #[msg("Allowance duration exceeds maximum allowed")]
    AllowanceDurationTooLong,

    #[msg("Allowance amount exceeds maximum allowed")]
    AllowanceAmountTooHigh,

    #[msg("Rate limit exceeded: too many allowance approvals")]
    RateLimitExceeded,

    #[msg("Invalid token account owner")]
    InvalidTokenAccountOwner,

    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Token account is frozen")]
    TokenAccountFrozen,

    #[msg("Token account not initialized")]
    TokenAccountNotInitialized,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Arithmetic underflow")]
    ArithmeticUnderflow,

    #[msg("Unauthorized: caller is not the processor")]
    UnauthorizedProcessor,

    #[msg("Unauthorized: caller is not the casino authority")]
    UnauthorizedAuthority,

    #[msg("Casino is currently paused")]
    CasinoPaused,

    #[msg("Invalid vault PDA")]
    InvalidVaultPDA,

    #[msg("Invalid casino vault PDA")]
    InvalidCasinoVaultPDA,

    #[msg("Bet ID already processed (duplicate)")]
    DuplicateBetId,

    #[msg("Bet ID is invalid or too long")]
    InvalidBetId,

    #[msg("Token mint mismatch with allowance")]
    TokenMintMismatch,

    #[msg("Invalid allowance PDA")]
    InvalidAllowancePDA,

    #[msg("Missing token delegation authority")]
    MissingTokenDelegation,

    #[msg("Missing required token account")]
    MissingTokenAccount,
}
