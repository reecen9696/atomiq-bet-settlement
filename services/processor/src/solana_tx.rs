use spl_associated_token_account::get_associated_token_address;
use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    sysvar,
    transaction::Transaction,
};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::Bet;

// Program IDs
const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

fn parse_allowance_nonce_registry_next_nonce(data: &[u8]) -> Option<u64> {
    // Anchor accounts have an 8-byte discriminator prefix.
    // Layout: discriminator (8) | user (32) | casino (32) | next_nonce (8) | bump (1)
    let min_len = 8 + 32 + 32 + 8;
    if data.len() < min_len {
        return None;
    }

    let next_nonce_offset = 8 + 32 + 32;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[next_nonce_offset..next_nonce_offset + 8]);
    Some(u64::from_le_bytes(buf))
}

fn parse_allowance_token_mint(data: &[u8]) -> Option<Pubkey> {
    // Anchor accounts have an 8-byte discriminator prefix.
    // Layout (prefix only): discriminator (8) | user (32) | casino (32) | token_mint (32) | ...
    let min_len = 8 + 32 + 32 + 32;
    if data.len() < min_len {
        return None;
    }

    let token_mint_offset = 8 + 32 + 32;
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&data[token_mint_offset..token_mint_offset + 32]);
    Some(Pubkey::new_from_array(buf))
}

fn allowance_account_exists(client: &RpcClient, allowance: &Pubkey) -> bool {
    client.get_account(allowance).is_ok()
}

/// Build and submit a batch of bets to Solana
pub async fn submit_batch_transaction(
    client: &RpcClient,
    bets: &[Bet],
    processor_keypair: &Keypair,
    vault_program_id: &Pubkey,
    max_bets_per_tx: usize,
) -> Result<(String, Vec<(Uuid, bool, i64)>)> {
    // Limit batch size to avoid transaction size / compute limits.
    if bets.len() > max_bets_per_tx {
        anyhow::bail!(
            "Batch too large: {} bets (max {})",
            bets.len(),
            max_bets_per_tx
        );
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

        // Derive casino vault PDA (program-owned account holding SOL)
        let (casino_vault, _) = Pubkey::find_program_address(
            &[b"casino-vault", casino_pda.as_ref()],
            vault_program_id,
        );

        // Derive vault authority PDA (used for SPL token signing)
        let (vault_authority, _) = Pubkey::find_program_address(
            &[b"vault-authority", casino_pda.as_ref()],
            vault_program_id,
        );

        // Allowance PDA must match the on-chain nonce-based PDA.
        // Prefer the PDA provided by the backend/UI; otherwise derive the most recent allowance
        // from the on-chain nonce registry.
        let allowance = if let Some(pda_str) = bet.allowance_pda.as_ref().filter(|s| !s.is_empty()) {
            let pda = Pubkey::from_str(pda_str).context("Invalid allowance_pda pubkey")?;
            if allowance_account_exists(client, &pda) {
                pda
            } else {
                tracing::warn!(
                    "Bet {} allowance_pda {} missing on-chain; attempting nonce-registry fallback",
                    bet.bet_id,
                    pda
                );
                derive_latest_allowance_pda_from_nonce_registry(client, vault_program_id, &user_pubkey, &casino_pda)
                    .with_context(|| {
                        format!(
                            "Allowance account not initialized (provided {}, no nonce-registry fallback) for bet {}",
                            pda, bet.bet_id
                        )
                    })?
            }
        } else {
            derive_latest_allowance_pda_from_nonce_registry(client, vault_program_id, &user_pubkey, &casino_pda)
                .with_context(|| {
                    format!(
                        "Bet {} missing allowance_pda and no initialized allowance could be derived from nonce registry",
                        bet.bet_id
                    )
                })?
        };

        // Determine whether this allowance is native SOL (no SPL token accounts) or SPL.
        // If we include token accounts for a native SOL allowance, Anchor will attempt to
        // deserialize them and fail with AccountNotInitialized.
        let allowance_acct = client
            .get_account(&allowance)
            .with_context(|| format!("Failed to fetch allowance account {}", allowance))?;
        let allowance_token_mint = parse_allowance_token_mint(&allowance_acct.data)
            .with_context(|| format!("Failed to parse allowance token_mint for {}", allowance))?;
        let is_native_sol = allowance_token_mint == system_program::ID || allowance_token_mint == Pubkey::default();

        let mut user_token_account: Option<Pubkey> = None;
        let mut casino_token_account: Option<Pubkey> = None;

        if !is_native_sol {
            let user_ata = get_associated_token_address(&user_pubkey, &allowance_token_mint);
            let casino_ata = get_associated_token_address(&casino_pda, &allowance_token_mint);

            // User ATA must exist if spending SPL tokens.
            if client.get_account(&user_ata).is_err() {
                anyhow::bail!(
                    "User token account {} not initialized for mint {} (bet {})",
                    user_ata,
                    allowance_token_mint,
                    bet.bet_id
                );
            }

            // Casino ATA can be created by the processor if missing.
            if client.get_account(&casino_ata).is_err() {
                let create_ata_ix = build_create_ata_instruction(
                    &processor_keypair.pubkey(),
                    &casino_pda,
                    &allowance_token_mint,
                )?;
                instructions.push(create_ata_ix);
            }

            user_token_account = Some(user_ata);
            casino_token_account = Some(casino_ata);
        }

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
            &casino_vault,
            &vault_authority,
            user_token_account.as_ref(),
            casino_token_account.as_ref(),
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
                &casino_vault,
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

    // Preflight simulation to capture full program logs on failure.
    // This makes diagnosing Anchor constraint failures and CPI errors much easier.
    let sim = client.simulate_transaction_with_config(
        &transaction,
        RpcSimulateTransactionConfig {
            sig_verify: false,
            replace_recent_blockhash: true,
            commitment: None,
            ..Default::default()
        },
    );
    match sim {
        Ok(resp) => {
            if let Some(err) = resp.value.err {
                if let Some(logs) = resp.value.logs {
                    let trimmed: Vec<String> = logs.into_iter().take(25).collect();
                    tracing::error!(
                        "Preflight simulation failed ({} bets). Logs:\n{}",
                        bets.len(),
                        trimmed.join("\n")
                    );
                    anyhow::bail!(
                        "Preflight simulation failed: {:?}\nLogs:\n{}",
                        err,
                        trimmed.join("\n")
                    );
                }
                anyhow::bail!("Preflight simulation failed: {:?}", err);
            }
        }
        Err(e) => {
            tracing::warn!("Preflight simulation RPC error: {:#}", e);
        }
    }

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

fn derive_latest_allowance_pda_from_nonce_registry(
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
fn build_payout_instruction(
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
            AccountMeta::new_readonly(sysvar::rent::ID, false), // rent
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
