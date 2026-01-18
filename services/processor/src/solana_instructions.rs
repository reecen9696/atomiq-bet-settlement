//! Solana instruction builders

use anyhow::Result;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
    sysvar,
};
use std::str::FromStr;

use shared::program_ids::{SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, SPL_TOKEN_PROGRAM_ID};

/// Build spend_from_allowance instruction
#[allow(clippy::too_many_arguments)]
pub fn build_spend_from_allowance_instruction(
    program_id: &Pubkey,
    user_vault: &Pubkey,
    casino: &Pubkey,
    allowance: &Pubkey,
    processed_bet: &Pubkey,
    casino_vault: &Pubkey,
    vault_authority: &Pubkey,
    user_token_account: Option<&Pubkey>,
    casino_token_account: Option<&Pubkey>,
    processor: &Pubkey,
    amount: u64,
    bet_id: &str,
) -> Instruction {
    // Instruction discriminator for spend_from_allowance
    // SHA256("global:spend_from_allowance")[0..8]
    let mut data = vec![143, 226, 77, 235, 46, 46, 239, 222]; // spend_from_allowance discriminator
    
    // Serialize amount (u64)
    data.extend_from_slice(&amount.to_le_bytes());
    
    // Serialize bet_id (String)
    let bet_id_bytes = bet_id.as_bytes();
    data.extend_from_slice(&(bet_id_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(bet_id_bytes);

    let mut accounts = vec![
        AccountMeta::new(*user_vault, false),
        AccountMeta::new(*casino, false),
        AccountMeta::new(*allowance, false),
        AccountMeta::new(*processed_bet, false),
        AccountMeta::new(*casino_vault, false),
        AccountMeta::new_readonly(*vault_authority, false),
    ];

    // Keep account ordering stable for Anchor optional accounts.
    // Anchor treats an optional account as None when the provided pubkey equals program_id.
    // Important: Must use 'new' (writable) to match the #[account(mut)] in Rust instruction,
    // even for placeholders, otherwise Anchor may fail to recognize them as None.
    match (user_token_account, casino_token_account) {
        (Some(user_ta), Some(casino_ta)) => {
            accounts.push(AccountMeta::new(*user_ta, false));
            accounts.push(AccountMeta::new(*casino_ta, false));
        }
        (None, None) => {
            accounts.push(AccountMeta::new(*program_id, false));
            accounts.push(AccountMeta::new(*program_id, false));
        }
        _ => {
            // Should never happen; treat as SOL-mode placeholders to avoid shifting.
            accounts.push(AccountMeta::new(*program_id, false));
            accounts.push(AccountMeta::new(*program_id, false));
        }
    }

    accounts.push(AccountMeta::new(*processor, true));
    accounts.push(AccountMeta::new_readonly(system_program::ID, false));

    // token_program is optional on-chain; use the same placeholder convention.
    if user_token_account.is_some() && casino_token_account.is_some() {
        accounts.push(AccountMeta::new_readonly(
            Pubkey::from_str(SPL_TOKEN_PROGRAM_ID).expect("Valid SPL token program ID"),
            false,
        ));
    } else {
        accounts.push(AccountMeta::new_readonly(*program_id, false));
    }

    Instruction {
        program_id: *program_id,
        accounts,
        data,
    }
}

/// Build payout instruction
pub fn build_payout_instruction(
    program_id: &Pubkey,
    casino: &Pubkey,
    casino_vault: &Pubkey,
    vault_authority: &Pubkey,
    user_vault: &Pubkey,
    processed_bet: &Pubkey,
    processor: &Pubkey,
    amount: u64,
    bet_id: &str,
) -> Instruction {
    // Instruction discriminator for payout
    // SHA256("global:payout")[0..8]
    let mut data = vec![149, 140, 194, 236, 174, 189, 6, 239]; // payout discriminator
    
    // Serialize amount (u64)
    data.extend_from_slice(&amount.to_le_bytes());
    
    // Serialize bet_id (String)
    let bet_id_bytes = bet_id.as_bytes();
    data.extend_from_slice(&(bet_id_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(bet_id_bytes);

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*user_vault, false),              // vault
            AccountMeta::new(*casino, false),                   // casino (writable for stats)
            AccountMeta::new(*casino_vault, false),             // casino_vault (program-owned, holds SOL)
            AccountMeta::new_readonly(*vault_authority, false), // vault_authority (PDA for SPL signing)
            // For SOL transfers, pass program_id as placeholder for optional token accounts
            AccountMeta::new_readonly(*program_id, false),      // user_token_account (optional)
            AccountMeta::new_readonly(*program_id, false),      // casino_token_account (optional)
            AccountMeta::new_readonly(*processed_bet, false),   // processed_bet (reference)
            AccountMeta::new(*processor, true),                 // processor (signer)
            AccountMeta::new_readonly(system_program::ID, false), // system_program
            // token_program (optional) - omit for SOL
        ],
        data,
    }
}

/// Build create associated token account instruction manually
pub fn build_create_ata_instruction(
    payer: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<Instruction> {
    let spl_token_program = Pubkey::from_str(SPL_TOKEN_PROGRAM_ID)
        .map_err(|_| anyhow::anyhow!("Invalid SPL token program ID"))?;
    let spl_ata_program = Pubkey::from_str(SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID)
        .map_err(|_| anyhow::anyhow!("Invalid ATA program ID"))?;

    // Derive the associated token account address
    let (ata_address, _) = Pubkey::find_program_address(
        &[
            owner.as_ref(),
            spl_token_program.as_ref(),
            mint.as_ref(),
        ],
        &spl_ata_program,
    );

    // Build the instruction
    Ok(Instruction {
        program_id: spl_ata_program,
        accounts: vec![
            AccountMeta::new(*payer, true),           // payer
            AccountMeta::new(ata_address, false),     // associated_token_account
            AccountMeta::new_readonly(*owner, false), // owner
            AccountMeta::new_readonly(*mint, false),  // mint
            AccountMeta::new_readonly(system_program::ID, false), // system_program
            AccountMeta::new_readonly(spl_token_program, false), // token_program
            AccountMeta::new_readonly(sysvar::rent::ID, false), // rent
        ],
        data: vec![], // No data needed for ATA creation
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_spend_from_allowance_instruction() {
        let program_id = Pubkey::new_unique();
        let user_vault = Pubkey::new_unique();
        let casino = Pubkey::new_unique();
        let allowance = Pubkey::new_unique();
        let processed_bet = Pubkey::new_unique();
        let casino_vault = Pubkey::new_unique();
        let vault_authority = Pubkey::new_unique();
        let processor = Pubkey::new_unique();

        // Test SOL mode (no token accounts)
        let instruction = build_spend_from_allowance_instruction(
            &program_id,
            &user_vault,
            &casino,
            &allowance,
            &processed_bet,
            &casino_vault,
            &vault_authority,
            None,
            None,
            &processor,
            1000,
            "test-bet-id",
        );

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 11);
        
        // Verify discriminator
        assert_eq!(&instruction.data[0..8], [143, 226, 77, 235, 46, 46, 239, 222]);
    }

    #[test]
    fn test_build_payout_instruction() {
        let program_id = Pubkey::new_unique();
        let casino = Pubkey::new_unique();
        let casino_vault = Pubkey::new_unique();
        let vault_authority = Pubkey::new_unique();
        let user_vault = Pubkey::new_unique();
        let processed_bet = Pubkey::new_unique();
        let processor = Pubkey::new_unique();

        let instruction = build_payout_instruction(
            &program_id,
            &casino,
            &casino_vault,
            &vault_authority,
            &user_vault,
            &processed_bet,
            &processor,
            2000,
            "payout-test",
        );

        assert_eq!(instruction.program_id, program_id);
        assert_eq!(instruction.accounts.len(), 9);
        
        // Verify discriminator
        assert_eq!(&instruction.data[0..8], [149, 140, 194, 236, 174, 189, 6, 239]);
    }
}