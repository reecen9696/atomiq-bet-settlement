/// Type-safe wrappers for domain primitives
/// 
/// These types prevent common errors by enforcing validation at construction time
/// and providing checked arithmetic operations.

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;
use uuid::Uuid;

use crate::constants::*;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Bet ID too long: {length} chars (max {max})")]
    BetIdTooLong { length: usize, max: usize },
    
    #[error("Invalid bet ID format: {0}")]
    InvalidBetIdFormat(String),
    
    #[error("Bet amount out of range: {amount} (min: {min}, max: {max})")]
    BetAmountOutOfRange { amount: u64, min: u64, max: u64 },
    
    #[error("Bet amount overflow in operation")]
    BetAmountOverflow,
    
    #[error("Invalid token type")]
    InvalidTokenType,
}

/// Type-safe bet identifier with validation
/// 
/// Enforces maximum length and format validation to prevent PDA derivation errors.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BetId(String);

impl BetId {
    /// Create a new BetId from a UUID
    pub fn new(uuid: Uuid) -> Self {
        // Remove hyphens to save space in PDA seeds (36 -> 32 chars)
        let id = uuid.to_string().replace("-", "");
        Self(id)
    }
    
    /// Get the inner string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the inner string, consuming self
    pub fn into_string(self) -> String {
        self.0
    }
    
    /// Convert back to UUID if possible
    pub fn to_uuid(&self) -> Result<Uuid, ValidationError> {
        Uuid::parse_str(&self.0)
            .map_err(|_| ValidationError::InvalidBetIdFormat(self.0.clone()))
    }
}

impl TryFrom<String> for BetId {
    type Error = ValidationError;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Remove hyphens if present
        let normalized = value.replace("-", "");
        
        // Validate length
        if normalized.len() > MAX_BET_ID_LENGTH {
            return Err(ValidationError::BetIdTooLong {
                length: normalized.len(),
                max: MAX_BET_ID_LENGTH,
            });
        }
        
        // Validate it's a valid UUID format
        Uuid::parse_str(&normalized)
            .map_err(|_| ValidationError::InvalidBetIdFormat(value.clone()))?;
        
        Ok(Self(normalized))
    }
}

impl TryFrom<Uuid> for BetId {
    type Error = ValidationError;
    
    fn try_from(uuid: Uuid) -> Result<Self, Self::Error> {
        Ok(Self::new(uuid))
    }
}

impl std::fmt::Display for BetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe lamport amount with overflow protection
/// 
/// Provides checked arithmetic operations to prevent integer overflow vulnerabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LamportAmount(u64);

impl LamportAmount {
    /// Create a new LamportAmount with validation
    pub fn new(amount: u64) -> Result<Self, ValidationError> {
        if amount < MIN_BET_LAMPORTS || amount > MAX_BET_LAMPORTS {
            return Err(ValidationError::BetAmountOutOfRange {
                amount,
                min: MIN_BET_LAMPORTS,
                max: MAX_BET_LAMPORTS,
            });
        }
        Ok(Self(amount))
    }
    
    /// Create without validation (for internal use)
    pub fn new_unchecked(amount: u64) -> Self {
        Self(amount)
    }
    
    /// Get the raw lamport value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    
    /// Checked addition
    pub fn checked_add(&self, other: LamportAmount) -> Result<Self, ValidationError> {
        self.0
            .checked_add(other.0)
            .map(Self::new_unchecked)
            .ok_or(ValidationError::BetAmountOverflow)
    }
    
    /// Checked subtraction
    pub fn checked_sub(&self, other: LamportAmount) -> Result<Self, ValidationError> {
        self.0
            .checked_sub(other.0)
            .map(Self::new_unchecked)
            .ok_or(ValidationError::BetAmountOverflow)
    }
    
    /// Checked multiplication
    pub fn checked_mul(&self, multiplier: u64) -> Result<Self, ValidationError> {
        self.0
            .checked_mul(multiplier)
            .map(Self::new_unchecked)
            .ok_or(ValidationError::BetAmountOverflow)
    }
    
    /// Convert to SOL (as f64)
    pub fn to_sol(&self) -> f64 {
        self.0 as f64 / 1_000_000_000.0
    }
    
    /// Create from SOL amount
    pub fn from_sol(sol: f64) -> Result<Self, ValidationError> {
        let lamports = (sol * 1_000_000_000.0) as u64;
        Self::new(lamports)
    }
}

impl TryFrom<u64> for LamportAmount {
    type Error = ValidationError;
    
    fn try_from(amount: u64) -> Result<Self, Self::Error> {
        Self::new(amount)
    }
}

impl From<LamportAmount> for u64 {
    fn from(amount: LamportAmount) -> Self {
        amount.0
    }
}

impl std::fmt::Display for LamportAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} lamports ({:.9} SOL)", self.0, self.to_sol())
    }
}

/// Token type discriminator
/// 
/// Distinguishes between native SOL, wrapped SOL, and other SPL tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    /// Native SOL (transferred via system program)
    NativeSOL,
    
    /// Wrapped SOL (SPL token representation of native SOL)
    WrappedSOL,
    
    /// Other SPL token (USDC, USDT, etc.)
    SPL(Pubkey),
}

impl TokenType {
    /// Check if this is native SOL
    pub fn is_native_sol(&self) -> bool {
        matches!(self, TokenType::NativeSOL)
    }
    
    /// Check if this is wrapped SOL
    pub fn is_wrapped_sol(&self) -> bool {
        matches!(self, TokenType::WrappedSOL)
    }
    
    /// Get the mint address for SPL tokens
    pub fn mint(&self) -> Option<Pubkey> {
        match self {
            TokenType::NativeSOL => None,
            TokenType::WrappedSOL => Some(WRAPPED_SOL_MINT),
            TokenType::SPL(mint) => Some(*mint),
        }
    }
}

impl TryFrom<String> for TokenType {
    type Error = ValidationError;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "SOL" => Ok(TokenType::NativeSOL),
            "WSOL" => Ok(TokenType::WrappedSOL),
            mint_str => {
                // Try to parse as pubkey
                mint_str
                    .parse::<Pubkey>()
                    .map(TokenType::SPL)
                    .map_err(|_| ValidationError::InvalidTokenType)
            }
        }
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::NativeSOL => write!(f, "SOL"),
            TokenType::WrappedSOL => write!(f, "WSOL"),
            TokenType::SPL(mint) => write!(f, "{}", mint),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bet_id_validation() {
        let uuid = Uuid::new_v4();
        let bet_id = BetId::new(uuid);
        assert!(bet_id.as_str().len() <= MAX_BET_ID_LENGTH);
        assert!(!bet_id.as_str().contains('-'));
    }
    
    #[test]
    fn test_bet_id_too_long() {
        let long_string = "a".repeat(MAX_BET_ID_LENGTH + 1);
        let result = BetId::try_from(long_string);
        assert!(matches!(result, Err(ValidationError::BetIdTooLong { .. })));
    }
    
    #[test]
    fn test_lamport_amount_validation() {
        // Valid amount
        let amount = LamportAmount::new(100_000_000).unwrap();
        assert_eq!(amount.as_u64(), 100_000_000);
        
        // Too small
        assert!(LamportAmount::new(1_000).is_err());
        
        // Too large
        assert!(LamportAmount::new(MAX_BET_LAMPORTS + 1).is_err());
    }
    
    #[test]
    fn test_lamport_amount_arithmetic() {
        let a = LamportAmount::new_unchecked(100);
        let b = LamportAmount::new_unchecked(50);
        
        assert_eq!(a.checked_add(b).unwrap().as_u64(), 150);
        assert_eq!(a.checked_sub(b).unwrap().as_u64(), 50);
        assert_eq!(a.checked_mul(2).unwrap().as_u64(), 200);
    }
    
    #[test]
    fn test_lamport_amount_overflow() {
        let a = LamportAmount::new_unchecked(u64::MAX);
        let b = LamportAmount::new_unchecked(1);
        assert!(a.checked_add(b).is_err());
    }
    
    #[test]
    fn test_token_type() {
        assert!(TokenType::NativeSOL.is_native_sol());
        assert!(TokenType::WrappedSOL.is_wrapped_sol());
        assert_eq!(TokenType::WrappedSOL.mint(), Some(WRAPPED_SOL_MINT));
    }
}
