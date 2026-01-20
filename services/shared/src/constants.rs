/// Shared constants for Atomik Wallet betting system
/// 
/// This module centralizes all magic numbers and configuration constants
/// to prevent inconsistencies across backend, processor, and smart contracts.

use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

/// Minimum bet amount in lamports (0.01 SOL)
/// 
/// Rationale: Prevents spam bets and ensures meaningful transactions.
/// Below this amount, transaction fees would consume significant percentage.
pub const MIN_BET_LAMPORTS: u64 = 10_000_000;

/// Maximum bet amount in lamports (1000 SOL)
/// 
/// Rationale: Anti-whale limit to prevent single bets from draining casino vault.
/// Protects against accidental large transfers and malicious attacks.
pub const MAX_BET_LAMPORTS: u64 = 1_000_000_000_000;

/// Maximum allowance duration in seconds (24 hours)
/// 
/// Rationale: Security limit to prevent indefinite allowances.
/// Users must re-approve after this period to maintain security.
pub const MAX_ALLOWANCE_DURATION_SECS: i64 = 86400;

/// Maximum allowance amount in lamports (10,000 SOL)
/// 
/// Rationale: Caps total allowance to prevent catastrophic loss if compromised.
pub const MAX_ALLOWANCE_AMOUNT_LAMPORTS: u64 = 10_000_000_000_000;

/// Wrapped SOL mint address (native SOL represented as SPL token)
/// 
/// This is the official Solana native mint address used for wrapped SOL.
pub const WRAPPED_SOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

/// Rent-exempt reserve for casino vault (65-byte account)
/// 
/// Pre-calculated rent for CasinoVault to avoid repeated Rent::get() calls.
/// Must be updated if CasinoVault::LEN changes.
pub const RENT_EXEMPT_RESERVE_CASINO_VAULT: u64 = 1_343_280;

/// Rent-exempt reserve for user vault (89-byte account)
pub const RENT_EXEMPT_RESERVE_USER_VAULT: u64 = 1_566_960;

/// Maximum bet ID length (UUID without hyphens = 32 chars)
/// 
/// Rationale: Solana PDA seeds have 32-byte limit per seed.
/// UUIDs are 36 chars with hyphens, 32 without.
pub const MAX_BET_ID_LENGTH: usize = 32;

/// Rate limiter window duration (1 hour)
/// 
/// Allowance approval rate limiting resets every hour.
pub const RATE_LIMITER_WINDOW_SECS: i64 = 3600;

/// Maximum allowance approvals per rate limit window (100)
/// 
/// Prevents allowance approval spam attacks.
pub const RATE_LIMITER_MAX_APPROVALS: u8 = 100;

/// Processor batch size (how many bets to claim at once)
pub const PROCESSOR_BATCH_SIZE: usize = 10;

/// Maximum retry attempts for failed bets
pub const MAX_BET_RETRIES: i32 = 5;

/// Base backoff delay in milliseconds for retry logic
pub const RETRY_BACKOFF_BASE_MS: i64 = 2_000;

/// Maximum backoff delay in milliseconds for retry logic
pub const RETRY_BACKOFF_MAX_MS: i64 = 60_000;
