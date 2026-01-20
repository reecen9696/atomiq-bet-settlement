//! Solana program IDs and public keys used across services
//!
//! Centralizes all program ID constants to ensure consistency
//! and make it easier to update when needed.

use anyhow::{Context, Result};
use std::env;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// SPL Token Program ID
pub const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// SPL Associated Token Account Program ID
pub const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// Get the Vault Program ID from environment variable
///
/// # Errors
/// Returns error if VAULT_PROGRAM_ID is not set or cannot be parsed
pub fn vault_program_id_str() -> Result<String> {
    env::var("VAULT_PROGRAM_ID")
        .context("VAULT_PROGRAM_ID environment variable not set")
}

/// Parse the Vault Program ID as a Pubkey
///
/// # Errors
/// Returns error if VAULT_PROGRAM_ID is not set or cannot be parsed as a valid Pubkey
pub fn vault_program_id() -> Result<Pubkey> {
    let id_str = vault_program_id_str()?;
    Pubkey::from_str(&id_str)
        .context("Failed to parse VAULT_PROGRAM_ID as a valid Pubkey")
}

/// Get SPL Token Program as Pubkey
pub fn spl_token_program_id() -> Pubkey {
    Pubkey::from_str(SPL_TOKEN_PROGRAM_ID)
        .expect("SPL_TOKEN_PROGRAM_ID is a valid constant")
}

/// Get SPL Associated Token Account Program as Pubkey
pub fn spl_ata_program_id() -> Pubkey {
    Pubkey::from_str(SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID)
        .expect("SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID is a valid constant")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spl_program_id_constants_are_valid() {
        // Ensure the constants are valid Pubkey strings
        assert!(SPL_TOKEN_PROGRAM_ID.len() > 32);
        assert!(SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID.len() > 32);
    }

    #[test]
    fn test_parse_spl_program_ids() {
        // Should not panic
        let _ = spl_token_program_id();
        let _ = spl_ata_program_id();
        
        // Should parse correctly
        assert!(Pubkey::from_str(SPL_TOKEN_PROGRAM_ID).is_ok());
        assert!(Pubkey::from_str(SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID).is_ok());
    }
}
