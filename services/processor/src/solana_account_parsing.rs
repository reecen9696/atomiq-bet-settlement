//! Account data parsing utilities for Solana accounts

use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;

/// Parse the next_nonce from allowance nonce registry account data
pub fn parse_allowance_nonce_registry_next_nonce(data: &[u8]) -> Result<u64> {
    // Anchor accounts have an 8-byte discriminator prefix.
    // Layout: discriminator (8) | user (32) | casino (32) | next_nonce (8) | bump (1)
    let min_len = 8 + 32 + 32 + 8;
    if data.len() < min_len {
        anyhow::bail!("Account data too short: {} bytes (expected at least {})", data.len(), min_len);
    }

    let next_nonce_offset = 8 + 32 + 32;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[next_nonce_offset..next_nonce_offset + 8]);
    Ok(u64::from_le_bytes(buf))
}

/// Parse the token_mint from allowance account data
pub fn parse_allowance_token_mint(data: &[u8]) -> Result<Pubkey> {
    // Anchor accounts have an 8-byte discriminator prefix.
    // Layout (prefix only): discriminator (8) | user (32) | casino (32) | token_mint (32) | ...
    let min_len = 8 + 32 + 32 + 32;
    if data.len() < min_len {
        anyhow::bail!("Account data too short: {} bytes (expected at least {})", data.len(), min_len);
    }

    let token_mint_offset = 8 + 32 + 32;
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&data[token_mint_offset..token_mint_offset + 32]);
    Ok(Pubkey::new_from_array(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_parse_allowance_nonce_registry_next_nonce() {
        // Create test data with correct layout
        let mut data = vec![0u8; 81]; // discriminator + user + casino + next_nonce + bump
        
        // Set next_nonce to 42 at offset 72 (8+32+32)
        let next_nonce_bytes = 42u64.to_le_bytes();
        data[72..80].copy_from_slice(&next_nonce_bytes);
        
        let result = parse_allowance_nonce_registry_next_nonce(&data).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_parse_allowance_nonce_registry_next_nonce_short_data() {
        let short_data = vec![0u8; 50]; // Too short
        let result = parse_allowance_nonce_registry_next_nonce(&short_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_allowance_token_mint() {
        // Create test data with correct layout
        let mut data = vec![0u8; 105]; // discriminator + user + casino + token_mint + extra
        
        // Set a test pubkey at token_mint offset 72 (8+32+32)
        let test_pubkey = Pubkey::new_unique();
        data[72..104].copy_from_slice(test_pubkey.as_ref());
        
        let result = parse_allowance_token_mint(&data).unwrap();
        assert_eq!(result, test_pubkey);
    }

    #[test]
    fn test_parse_allowance_token_mint_short_data() {
        let short_data = vec![0u8; 50]; // Too short
        let result = parse_allowance_token_mint(&short_data);
        assert!(result.is_err());
    }
}