//! Program Derived Address (PDA) derivation utilities

use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::solana_account_parsing::parse_allowance_nonce_registry_next_nonce;

/// Check if an allowance account exists on-chain
pub fn allowance_account_exists(client: &RpcClient, allowance: &Pubkey) -> bool {
    client.get_account(allowance).is_ok()
}

/// Derive the latest allowance PDA from the nonce registry
pub fn derive_latest_allowance_pda_from_nonce_registry(
    client: &RpcClient,
    program_id: &Pubkey,
    user: &Pubkey,
    casino: &Pubkey,
) -> Result<Pubkey> {
    let (nonce_registry, _) = Pubkey::find_program_address(
        &[b"allowance-nonce", user.as_ref(), casino.as_ref()],
        program_id,
    );

    let acct = client
        .get_account(&nonce_registry)
        .with_context(|| format!("Nonce registry account {} not found", nonce_registry))?;
    
    let next_nonce = parse_allowance_nonce_registry_next_nonce(&acct.data)
        .context("Failed to parse nonce registry next_nonce")?;
    
    if next_nonce == 0 {
        anyhow::bail!("Nonce registry next_nonce is 0 (no allowance has been approved yet)");
    }

    let nonce = next_nonce - 1;
    let (allowance, _) = Pubkey::find_program_address(
        &[b"allowance", user.as_ref(), casino.as_ref(), &nonce.to_le_bytes()],
        program_id,
    );

    if !allowance_account_exists(client, &allowance) {
        anyhow::bail!(
            "Derived allowance PDA {} for nonce {} is not initialized",
            allowance,
            nonce
        );
    }

    Ok(allowance)
}

/// Derive casino PDA
pub fn derive_casino_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"casino"], program_id)
}

/// Derive user vault PDA (requires casino PDA)
pub fn derive_user_vault_pda(user_pubkey: &Pubkey, casino_pubkey: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"vault", casino_pubkey.as_ref(), user_pubkey.as_ref()],
        program_id,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_casino_pda() {
        let program_id = Pubkey::new_unique();
        let (casino_pda, _bump) = derive_casino_pda(&program_id);
        
        // Verify it's a valid PDA by checking it matches expected derivation
        let expected = Pubkey::find_program_address(&[b"casino"], &program_id);
        assert_eq!(casino_pda, expected.0);
    }

    #[test]
    fn test_derive_user_vault_pda() {
        let program_id = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let casino = Pubkey::new_unique();
        
        let (vault_pda, _bump) = derive_user_vault_pda(&user, &casino, &program_id);
        
        // Verify it matches expected derivation
        let expected = Pubkey::find_program_address(
            &[b"vault", casino.as_ref(), user.as_ref()],
            &program_id,
        );
        assert_eq!(vault_pda, expected.0);
    }
}