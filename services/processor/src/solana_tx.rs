use spl_associated_token_account::get_associated_token_address;
use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::Bet;

// Program IDs
const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// Build and submit a batch of bets to Solana
pub async fn submit_batch_transaction(
    client: &RpcClient,
    bets: &[Bet],
    processor_keypair: &Keypair,
    vault_program_id: &Pubkey,
) -> Result<(String, Vec<(Uuid, bool, i64)>)> {
    // For now, limit batch size to avoid compute limits
    if bets.len() > 5 {
        anyhow::bail!("Batch too large: {} bets (max 5)", bets.len());
    }

    // Simulate coinflip outcomes first
    let mut results = Vec::new();
    let mut instructions = Vec::new();

    for bet in bets {
        // Determine bet result
        let won = simulate_coinflip();
        let payout = if won { bet.stake_amount * 2 } else { 0 };
        results.push((bet.bet_id, won, payout));

        // Parse user wallet pubkey
        let user_pubkey = Pubkey::from_str(&bet.user_wallet)
            .context("Invalid user wallet pubkey")?;

        // Derive casino PDA
        let (casino_pda, _) = derive_casino_pda(vault_program_id);

        // Derive user vault PDA
        let (user_vault_pda, _) = derive_user_vault_pda(&user_pubkey, &casino_pda, vault_program_id);

        // Derive vault authority PDA (used for casino vault)
        let (vault_authority, _) = Pubkey::find_program_address(
            &[b"vault-authority", casino_pda.as_ref()],
            vault_program_id,
        );

        // Query for active allowance - derive dynamically instead of hardcoded
        // For now we'll derive it based on user and current timestamp
        // In production, this should be queried from database or chain
        let allowance_timestamp = bet.created_at.timestamp();
        let (allowance, _) = Pubkey::find_program_address(
            &[
                b"allowance",
                user_pubkey.as_ref(),
                casino_pda.as_ref(),
                &allowance_timestamp.to_le_bytes(),
            ],
            vault_program_id,
        );
        
        // Derive token accounts dynamically
        let wrapped_sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")
            .expect("Valid wrapped SOL mint");
        let user_token_account = get_associated_token_address(&user_pubkey, &wrapped_sol_mint);
        let casino_token_account = get_associated_token_address(&casino_pda, &wrapped_sol_mint);

        // Temporarily disable ATA creation to isolate the issue
        // if client.get_account(&casino_token_account).is_err() {
        //     let wrapped_sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")
        //         .expect("Valid wrapped SOL mint");
        //     
        //     let create_ata_ix = build_create_ata_instruction(
        //         &processor_keypair.pubkey(),
        //         &casino_pda,
        //         &wrapped_sol_mint,
        //     )?;
        //     instructions.push(create_ata_ix);
        // }

        // Derive processed_bet PDA (use UUID string without hyphens to stay under 32 byte limit)
        let bet_id_no_hyphens = bet.bet_id.to_string().replace("-", "");
        let (processed_bet, _) = Pubkey::find_program_address(
            &[b"processed-bet", bet_id_no_hyphens.as_bytes()],
            vault_program_id,
        );

        // Build spend_from_allowance instruction
        let spend_ix = build_spend_from_allowance_instruction(
            vault_program_id,
            &user_vault_pda,
            &casino_pda,
            &allowance,
            &processed_bet,
            &vault_authority,
            &user_token_account,
            &casino_token_account,
            &processor_keypair.pubkey(),
            bet.stake_amount as u64,
            &bet_id_no_hyphens, // Pass without hyphens to match PDA derivation
        );
        instructions.push(spend_ix);

        // If user won, add payout instruction
        if won {
            // Use same UUID format (no hyphens) for payout processed-bet PDA
            let payout_bet_id = format!("payout{}", bet.bet_id.to_string().replace("-", "").chars().take(24).collect::<String>());
            let (processed_bet_payout, _) = Pubkey::find_program_address(
                &[b"payout", payout_bet_id.as_bytes()],
                vault_program_id,
            );
            
            let payout_ix = build_payout_instruction(
                vault_program_id,
                &casino_pda,
                &vault_authority,
                &user_vault_pda,
                &processed_bet_payout,
                &processor_keypair.pubkey(),
                payout as u64,
                &payout_bet_id,
            );
            instructions.push(payout_ix);
        }
    }

    // Get recent blockhash
    let recent_blockhash = client
        .get_latest_blockhash()
        .context("Failed to get recent blockhash")?;

    // Build and sign transaction
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&processor_keypair.pubkey()),
        &[processor_keypair],
        recent_blockhash,
    );

    // Send and confirm transaction
    let signature = client
        .send_and_confirm_transaction(&transaction)
        .context("Failed to send and confirm transaction")?;

    tracing::info!(
        "Solana transaction confirmed: {} ({} bets)",
        signature,
        bets.len()
    );

    Ok((signature.to_string(), results))
}

/// Derive casino PDA
fn derive_casino_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"casino"], program_id)
}

/// Derive user vault PDA (requires casino PDA)
fn derive_user_vault_pda(user_pubkey: &Pubkey, casino_pubkey: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"vault", casino_pubkey.as_ref(), user_pubkey.as_ref()],
        program_id,
    )
}

/// Build spend_from_allowance instruction
fn build_spend_from_allowance_instruction(
    program_id: &Pubkey,
    user_vault: &Pubkey,
    casino: &Pubkey,
    allowance: &Pubkey,
    processed_bet: &Pubkey,
    casino_vault: &Pubkey,
    user_token_account: &Pubkey,
    casino_token_account: &Pubkey,
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

    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*user_vault, false),
            AccountMeta::new_readonly(*casino, false),
            AccountMeta::new(*allowance, false),
            AccountMeta::new(*processed_bet, false),
            AccountMeta::new(*casino_vault, false),
            // Token accounts for wrapped SOL
            AccountMeta::new(*user_token_account, false),
            AccountMeta::new(*casino_token_account, false),
            AccountMeta::new(*processor, true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(
                Pubkey::from_str(SPL_TOKEN_PROGRAM_ID).expect("Valid SPL token program ID"), 
                false
            ), // token_program
            // token_program (optional) - omit for SOL
        ],
        data,
    }
}

/// Build payout instruction
fn build_payout_instruction(
    program_id: &Pubkey,
    casino: &Pubkey,
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
            AccountMeta::new(*vault_authority, false),          // casino_vault (using vault_authority which holds SOL)
            AccountMeta::new_readonly(*vault_authority, false), // vault_authority (PDA for signing)
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

/// Simulate coinflip outcome
fn simulate_coinflip() -> bool {
    use rand::Rng;
    rand::thread_rng().gen_bool(0.5)
}

/// Build create associated token account instruction manually
fn build_create_ata_instruction(
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
        ],
        data: vec![], // No data needed for ATA creation
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_user_vault_pda() {
        let user_pubkey = Pubkey::new_unique();
        let casino_pubkey = Pubkey::new_unique();
        let program_id = Pubkey::new_unique();

        let (pda1, bump1) = derive_user_vault_pda(&user_pubkey, &casino_pubkey, &program_id);
        let (pda2, bump2) = derive_user_vault_pda(&user_pubkey, &casino_pubkey, &program_id);

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn test_derive_casino_pda() {
        let program_id = Pubkey::new_unique();

        let (pda1, bump1) = derive_casino_pda(&program_id);
        let (pda2, bump2) = derive_casino_pda(&program_id);

        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }
}
