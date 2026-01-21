use anchor_lang::prelude::*;

/// User vault account - stores SOL and tracks allowances
#[account]
pub struct Vault {
    /// Owner of this vault
    pub owner: Pubkey,
    /// Casino this vault is associated with
    pub casino: Pubkey,
    /// Bump seed for PDA
    pub bump: u8,
    /// SOL balance (tracked for convenience)
    pub sol_balance: u64,
    /// Timestamp when vault was created
    pub created_at: i64,
    /// Last activity timestamp
    pub last_activity: i64,
}

impl Vault {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        32 + // casino
        1 + // bump
        8 + // sol_balance
        8 + // created_at
        8; // last_activity
}

/// Casino vault account - program-owned account holding casino funds
#[account]
pub struct CasinoVault {
    /// Casino this vault is associated with
    pub casino: Pubkey,
    /// Bump seed for PDA
    pub bump: u8,
    /// SOL balance (tracked for convenience)
    pub sol_balance: u64,
    /// Timestamp when vault was created
    pub created_at: i64,
    /// Last activity timestamp
    pub last_activity: i64,
}

impl CasinoVault {
    pub const LEN: usize = 8 + // discriminator
        32 + // casino
        1 + // bump
        8 + // sol_balance
        8 + // created_at
        8; // last_activity
}

/// Casino configuration and authority
#[account]
pub struct Casino {
    /// Casino authority (admin)
    pub authority: Pubkey,
    /// Processor authorized to execute bets
    pub processor: Pubkey,
    /// Casino treasury pubkey
    pub treasury: Pubkey,
    /// Bump seed for casino PDA
    pub bump: u8,
    /// Vault authority bump (for signing)
    pub vault_authority_bump: u8,
    /// Emergency pause flag
    pub paused: bool,
    /// Total bets processed
    pub total_bets: u64,
    /// Total volume processed
    pub total_volume: u64,
    /// Timestamp when casino was created
    pub created_at: i64,
}

impl Casino {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        32 + // processor
        32 + // treasury
        1 + // bump
        1 + // vault_authority_bump
        1 + // paused
        8 + // total_bets
        8 + // total_volume
        8; // created_at
}

/// Allowance for spending without per-transaction signatures
#[account]
pub struct Allowance {
    /// User who approved this allowance
    pub user: Pubkey,
    /// Casino this allowance is for
    pub casino: Pubkey,
    /// Token mint (System Program pubkey for SOL, or SPL mint)
    pub token_mint: Pubkey,
    /// Total approved amount
    pub amount: u64,
    /// Amount already spent
    pub spent: u64,
    /// Expiry timestamp (Unix timestamp)
    pub expires_at: i64,
    /// Created timestamp
    pub created_at: i64,
    /// Nonce for uniqueness (prevents replay attacks)
    pub nonce: u64,
    /// Revoked flag
    pub revoked: bool,
    /// Bump seed for PDA
    pub bump: u8,
    /// Last spent timestamp
    pub last_spent_at: i64,
    /// Number of times spent
    pub spend_count: u32,
}

impl Allowance {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        32 + // casino
        32 + // token_mint
        8 + // amount
        8 + // spent
        8 + // expires_at
        8 + // created_at
        8 + // nonce
        1 + // revoked
        1 + // bump
        8 + // last_spent_at
        4; // spend_count

    pub fn remaining(&self) -> u64 {
        self.amount.saturating_sub(self.spent)
    }

    pub fn is_valid(&self, clock: &Clock) -> bool {
        !self.revoked && clock.unix_timestamp <= self.expires_at
    }
}

/// Per-user-per-casino nonce registry for deterministic allowance PDA creation
#[account]
pub struct AllowanceNonceRegistry {
    /// User who owns the allowances
    pub user: Pubkey,
    /// Casino this registry is for
    pub casino: Pubkey,
    /// Next nonce to use when creating an allowance PDA
    pub next_nonce: u64,
    /// Bump seed
    pub bump: u8,
}

impl AllowanceNonceRegistry {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        32 + // casino
        8 + // next_nonce
        1; // bump
}

/// Rate limiter for allowance approvals
#[account]
pub struct RateLimiter {
    /// User being rate limited
    pub user: Pubkey,
    /// Number of approvals in current window
    pub approvals_count: u8,
    /// Start of current time window
    pub window_start: i64,
    /// Bump seed
    pub bump: u8,
}

impl RateLimiter {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        1 + // approvals_count
        8 + // window_start
        1; // bump

    pub const WINDOW_DURATION: i64 = 3600; // 1 hour
    pub const MAX_APPROVALS: u8 = 100;
}

/// Processed bet tracker (prevents duplicate processing)
#[account]
pub struct ProcessedBet {
    /// Bet ID
    pub bet_id: String,
    /// User who placed the bet
    pub user: Pubkey,
    /// Amount
    pub amount: u64,
    /// Timestamp when processed
    pub processed_at: i64,
    /// Transaction signature
    pub signature: String,
    /// Bump seed
    pub bump: u8,
}

impl ProcessedBet {
    // Max signature length (base58 encoded transaction signature)
    pub const MAX_SIGNATURE_LEN: usize = 88;
    
    pub const LEN: usize = 8 + // discriminator
        4 + MAX_BET_ID_LENGTH + // bet_id (String with length prefix)
        32 + // user
        8 + // amount
        8 + // processed_at
        4 + Self::MAX_SIGNATURE_LEN + // signature
        1; // bump
}

// Constants with rationale

/// Minimum bet amount in lamports (0.01 SOL)
/// Rationale: Prevents spam bets and ensures meaningful transactions
pub const MIN_BET_LAMPORTS: u64 = 10_000_000;

/// Maximum bet amount in lamports (1000 SOL)
/// Rationale: Anti-whale limit to prevent single bets from draining casino vault
pub const MAX_BET_LAMPORTS: u64 = 1_000_000_000_000;

/// Maximum allowance duration in seconds (24 hours)
/// Rationale: Security limit to prevent indefinite allowances
pub const MAX_ALLOWANCE_DURATION: i64 = 86400;

/// Maximum allowance amount in lamports (10,000 SOL)
/// Rationale: Caps total allowance to prevent catastrophic loss if compromised
pub const MAX_ALLOWANCE_AMOUNT: u64 = 10_000_000_000_000;

/// Rent-exempt reserve for casino vault (65-byte account)
/// Pre-calculated rent to avoid repeated Rent::get() calls
/// IMPORTANT: Must be updated if CasinoVault::LEN changes
pub const RENT_EXEMPT_RESERVE_CASINO_VAULT: u64 = 1_343_280;

/// Rent-exempt reserve for user vault (89-byte account)
/// IMPORTANT: Must be updated if Vault::LEN changes
pub const RENT_EXEMPT_RESERVE_USER_VAULT: u64 = 1_566_960;

/// Maximum bet ID length (UUID without hyphens = 32 chars)
/// Rationale: Solana PDA seeds have 32-byte limit per seed
pub const MAX_BET_ID_LENGTH: usize = 32;
