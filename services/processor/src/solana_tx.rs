//! Solana transaction building and submission
//!
//! This module handles the complete lifecycle of submitting batch bet transactions
//! to the Solana blockchain. It has been decomposed into focused modules for maintainability.

// Re-export commonly used functions from other modules in the crate
pub use crate::solana_account_parsing::{parse_allowance_nonce_registry_next_nonce, parse_allowance_token_mint};
pub use crate::solana_instructions::{build_create_ata_instruction, build_payout_instruction, build_spend_from_allowance_instruction};
pub use crate::solana_pda::{allowance_account_exists, derive_casino_pda, derive_latest_allowance_pda_from_nonce_registry, derive_user_vault_pda};
pub use crate::solana_simulation::simulate_coinflip;

use anyhow::{Context, Result};
use spl_associated_token_account::get_associated_token_address;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;
use uuid::Uuid;

use crate::domain::Bet;

/// Build and submit a batch of bets to Solana
///
/// This is the main entry point for processing bet transactions. It:
/// 1. Validates input constraints
/// 2. Simulates coinflip outcomes for all bets
/// 3. Builds spend_from_allowance instructions
/// 4. Builds payout instructions for winning bets
/// 5. Creates any missing Associated Token Accounts
/// 6. Simulates the transaction for debugging
/// 7. Sends and confirms the transaction
///
/// Returns the transaction signature and bet results
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
