//! Settlement worker that polls blockchain API and processes settlements

use crate::{
    blockchain_client::{BlockchainClient, GameSettlementInfo},
    config::Config,
    coordinator::{SettlementBatch, BatchType},
    solana_client::SolanaClientPool,
    solana_tx,
};
use anyhow::{Context, Result};
use solana_sdk::signature::{Keypair, Signer};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

pub struct SettlementWorker {
    blockchain_client: Arc<BlockchainClient>,
    solana_client: Arc<SolanaClientPool>,
    processor_keypair: Arc<Keypair>,
    config: Config,
    worker_id: usize,
    work_receiver: Option<mpsc::Receiver<SettlementBatch>>,
}

impl SettlementWorker {
    pub fn new(
        blockchain_client: Arc<BlockchainClient>,
        solana_client: Arc<SolanaClientPool>,
        processor_keypair: Arc<Keypair>,
        config: Config,
        worker_id: usize,
    ) -> Self {
        Self {
            blockchain_client,
            solana_client,
            processor_keypair,
            config,
            worker_id,
            work_receiver: None,
        }
    }

    pub fn with_channel(
        blockchain_client: Arc<BlockchainClient>,
        solana_client: Arc<SolanaClientPool>,
        processor_keypair: Arc<Keypair>,
        config: Config,
        worker_id: usize,
        work_receiver: mpsc::Receiver<SettlementBatch>,
    ) -> Self {
        Self {
            blockchain_client,
            solana_client,
            processor_keypair,
            config,
            worker_id,
            work_receiver: Some(work_receiver),
        }
    }

    pub async fn run(mut self) {
        if self.config.processor.coordinator_enabled {
            // New coordinator-based mode
            self.run_with_coordinator().await;
        } else {
            // Legacy polling mode
            self.run_legacy().await;
        }
    }

    /// New coordinator-based mode - receive work from channel
    async fn run_with_coordinator(&mut self) {
        info!(
            worker_id = self.worker_id,
            "Settlement worker starting (coordinator mode)"
        );

        let Some(mut receiver) = self.work_receiver.take() else {
            error!(worker_id = self.worker_id, "Worker started in coordinator mode but has no channel");
            return;
        };

        while let Some(batch) = receiver.recv().await {
            info!(
                worker_id = self.worker_id,
                batch_id = %batch.batch_id,
                batch_type = ?batch.batch_type,
                settlement_count = batch.settlements.len(),
                "Received batch from coordinator"
            );

            if let Err(e) = self.process_settlement_batch(batch).await {
                error!(
                    worker_id = self.worker_id,
                    error = %e,
                    "Batch processing failed"
                );
            }
        }

        warn!(worker_id = self.worker_id, "Coordinator channel closed, worker shutting down");
    }

    /// Legacy polling mode - fetch from API directly
    async fn run_legacy(&self) {
        let poll_interval = Duration::from_secs(self.config.blockchain.poll_interval_seconds);
        
        info!(
            worker_id = self.worker_id,
            poll_interval_seconds = self.config.blockchain.poll_interval_seconds,
            batch_size = self.config.blockchain.settlement_batch_size,
            total_workers = self.config.processor.settlement_worker_count,
            "Settlement worker starting (legacy polling mode)"
        );

        loop {
            info!(worker_id = self.worker_id, "Starting settlement batch processing cycle");
            
            if let Err(e) = self.process_batch().await {
                error!(worker_id = self.worker_id, error = %e, "Settlement batch processing failed");
            }

            info!(worker_id = self.worker_id, "Completed batch processing, sleeping for {} seconds", poll_interval.as_secs());
            sleep(poll_interval).await;
        }
    }

    /// Process a batch received from coordinator
    async fn process_settlement_batch(&self, batch: SettlementBatch) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Process each settlement in the batch
        for game in batch.settlements {
            if let Err(e) = self.process_settlement(game).await {
                error!(
                    worker_id = self.worker_id,
                    batch_id = %batch.batch_id,
                    error = %e,
                    "Settlement processing failed in batch"
                );
            }
        }

        let duration = start_time.elapsed();
        info!(
            worker_id = self.worker_id,
            batch_id = %batch.batch_id,
            duration_ms = duration.as_millis(),
            "Batch processing completed"
        );

        Ok(())
    }

    async fn process_batch(&self) -> Result<()> {
        // Calculate per-worker batch size to reduce overlap between workers
        // Total batch size is divided among workers to minimize duplicate fetches
        let per_worker_batch_size = (self.config.blockchain.settlement_batch_size 
            / self.config.processor.settlement_worker_count).max(1);
        
        // Fetch pending settlements from blockchain API
        let games = self.blockchain_client
            .fetch_pending_settlements(per_worker_batch_size)
            .await
            .context("Failed to fetch pending settlements")?;

        if games.is_empty() {
            info!(worker_id = self.worker_id, "No pending settlements found");
            return Ok(());
        }

        info!(
            worker_id = self.worker_id,
            pending_count = games.len(),
            per_worker_batch = per_worker_batch_size,
            "Processing settlements"
        );

        // Process each settlement
        for game in games {
            if let Err(e) = self.process_settlement(game).await {
                // Log error but continue with other settlements
                error!(worker_id = self.worker_id, error = %e, "Settlement processing failed");
            }
        }

        Ok(())
    }

    async fn process_settlement(&self, game: GameSettlementInfo) -> Result<()> {
        let tx_id = game.transaction_id;
        
        debug!(
            worker_id = self.worker_id,
            tx_id,
            player = %game.player_address,
            outcome = %game.outcome,
            payout = game.payout,
            "Processing settlement"
        );

        // SAFETY: Check if settlement was already processed (has solana_tx_id)
        // This handles the case where Solana TX succeeded but DB update failed
        // We can skip the Solana step and just update the DB status
        if let Some(existing_tx_id) = &game.solana_tx_id {
            info!(
                worker_id = self.worker_id,
                tx_id,
                solana_tx = %existing_tx_id,
                "Settlement already has Solana TX, marking as complete"
            );
            
            // Retry indefinitely to update status - critical for consistency
            return self.update_settlement_complete_with_retry(
                tx_id,
                existing_tx_id.clone(),
                game.version,
            ).await;
        }

        // Update status to SubmittedToSolana
        match self.blockchain_client
            .update_settlement_status(
                tx_id,
                "SubmittedToSolana",
                None,
                None,
                game.version,
                None,
                None,
            )
            .await
        {
            Ok(_) => {
                info!(worker_id = self.worker_id, tx_id, "Status updated to SubmittedToSolana");
            }
            Err(e) => {
                let error_str = e.to_string();
                
                // Version conflict means another worker is processing this settlement - this is expected and safe
                if error_str.contains("Version conflict") || error_str.contains("409") {
                    debug!(
                        worker_id = self.worker_id,
                        tx_id,
                        "Another worker is processing this settlement (version conflict) - skipping"
                    );
                    return Ok(()); // Not an error - another worker won the race
                }
                
                error!(worker_id = self.worker_id, tx_id, error = %e, "Failed to update status to SubmittedToSolana");
                return Err(e).context("Failed to update status to SubmittedToSolana");
            }
        }

        // Process on Solana
        let solana_tx_sig = match self.settle_on_solana(&game).await {
            Ok(sig) => sig,
            Err(e) => {
                let error_msg = format!("Solana settlement failed: {}", e);
                warn!(
                    worker_id = self.worker_id,
                    tx_id,
                    error = %e,
                    "Solana settlement failed, updating status to SettlementFailed"
                );
                
                // Calculate retry logic: max 3 retries with 5s, 10s, 15s backoff
                let new_retry_count = game.retry_count + 1;
                let (status, next_retry_after) = if new_retry_count >= 3 {
                    // Exceeded max retries - mark as permanent failure
                    ("SettlementFailedPermanent", None)
                } else {
                    // Calculate backoff: 5s, 10s, 15s
                    let backoff_seconds = (new_retry_count as i64) * 5;
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as i64;
                    let retry_after = now_ms + (backoff_seconds * 1000);
                    ("SettlementFailed", Some(retry_after))
                };
                
                info!(
                    worker_id = self.worker_id,
                    tx_id,
                    retry_count = new_retry_count,
                    status = status,
                    next_retry_after,
                    "Updating settlement status with retry logic"
                );
                
                // Update status to SettlementFailed or SettlementFailedPermanent
                if let Err(update_err) = self.blockchain_client
                    .update_settlement_status(
                        tx_id,
                        status,
                        None,
                        Some(error_msg),
                        game.version + 1,
                        Some(new_retry_count),
                        next_retry_after,
                    )
                    .await
                {
                    error!(
                        worker_id = self.worker_id,
                        tx_id,
                        solana_error = %e,
                        update_error = %update_err,
                        "Failed to update settlement status to SettlementFailed"
                    );
                }
                
                return Err(e);
            }
        };

        // CRITICAL SAFETY: Update status to SettlementComplete with infinite retry
        // If Solana TX succeeded, we MUST persist this state in the blockchain DB
        // Retry indefinitely with backoff until success
        info!(
            worker_id = self.worker_id,
            tx_id,
            solana_tx = %solana_tx_sig,
            "Solana settlement succeeded, updating status to SettlementComplete"
        );

        self.update_settlement_complete_with_retry(
            tx_id,
            solana_tx_sig.clone(),
            game.version + 1,
        ).await?;

        info!(
            worker_id = self.worker_id,
            tx_id,
            solana_tx = %solana_tx_sig,
            "Settlement completed successfully"
        );

        Ok(())
    }

    /// CRITICAL SAFETY METHOD: Update settlement to SettlementComplete with infinite retry
    /// This ensures that if a Solana transaction succeeded, we ALWAYS update the blockchain DB
    /// Prevents the catastrophic scenario where SOL is transferred but settlement stays pending
    async fn update_settlement_complete_with_retry(
        &self,
        tx_id: u64,
        solana_tx_sig: String,
        expected_version: u64,
    ) -> Result<()> {
        let mut retry_count = 0;
        let mut backoff_seconds = 1;
        
        loop {
            match self.blockchain_client
                .update_settlement_status(
                    tx_id,
                    "SettlementComplete",
                    Some(solana_tx_sig.clone()),
                    None,
                    expected_version,
                    None,
                    None,
                )
                .await
            {
                Ok(_) => {
                    if retry_count > 0 {
                        info!(
                            worker_id = self.worker_id,
                            tx_id,
                            solana_tx = %solana_tx_sig,
                            retry_count,
                            "Status updated to SettlementComplete after retries"
                        );
                    } else {
                        info!(
                            worker_id = self.worker_id,
                            tx_id,
                            solana_tx = %solana_tx_sig,
                            "Status updated to SettlementComplete"
                        );
                    }
                    return Ok(());
                }
                Err(e) => {
                    let error_str = e.to_string();
                    
                    // Version conflict means another worker already updated it - success!
                    if error_str.contains("Version conflict") || error_str.contains("409") {
                        info!(
                            worker_id = self.worker_id,
                            tx_id,
                            solana_tx = %solana_tx_sig,
                            "Settlement already completed by another worker"
                        );
                        return Ok(());
                    }
                    
                    // For any other error, retry with exponential backoff
                    // NEVER give up - Solana TX succeeded so we MUST update DB
                    retry_count += 1;
                    error!(
                        worker_id = self.worker_id,
                        tx_id,
                        solana_tx = %solana_tx_sig,
                        retry_count,
                        backoff_seconds,
                        error = %e,
                        "CRITICAL: Failed to update SettlementComplete, will retry indefinitely"
                    );
                    
                    sleep(Duration::from_secs(backoff_seconds)).await;
                    
                    // Exponential backoff capped at 60 seconds
                    backoff_seconds = (backoff_seconds * 2).min(60);
                }
            }
        }
    }

    async fn settle_on_solana(&self, game: &GameSettlementInfo) -> Result<String> {
        let bet_id = format!("bet-{}", game.transaction_id);
        
        // Determine if win or loss
        let is_win = game.outcome == "Win";

        if is_win {
            // Win: payout from casino vault
            self.process_payout(game, &bet_id).await
        } else {
            // Loss: spend from user's allowance
            self.process_spend(game, &bet_id).await
        }
    }

    async fn process_payout(&self, game: &GameSettlementInfo, bet_id: &str) -> Result<String> {
        use solana_sdk::{transaction::Transaction, system_program};
        use crate::solana_pda::{derive_casino_pda, derive_user_vault_pda};
        use crate::solana_instructions::build_payout_instruction;
        
        // Parse addresses
        let player_pubkey = game.player_address.parse()
            .context("Invalid player address")?;
        let vault_program_id = self.config.solana.vault_program_id.parse()?;

        // Derive PDAs
        let (casino_pda, _) = derive_casino_pda(&vault_program_id);
        let (user_vault_pda, _) = derive_user_vault_pda(&player_pubkey, &casino_pda, &vault_program_id);
        let (casino_vault, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"casino-vault", casino_pda.as_ref()],
            &vault_program_id,
        );
        let (vault_authority, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"vault-authority", casino_pda.as_ref()],
            &vault_program_id,
        );

        // Derive PDA for processed bet
        let (processed_bet_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"processed-bet", bet_id.as_bytes()],
            &vault_program_id,
        );

        // Build payout instruction
        let payout_ix = build_payout_instruction(
            &vault_program_id,
            &casino_pda,
            &casino_vault,
            &vault_authority,
            &user_vault_pda,
            &processed_bet_pda,
            &self.processor_keypair.pubkey(),
            game.payout,
            bet_id,
        );

        // Get recent blockhash and send
        let client = self.solana_client.get_client().await;
        let recent_blockhash = client.get_latest_blockhash()?;
        
        let transaction = Transaction::new_signed_with_payer(
            &[payout_ix],
            Some(&self.processor_keypair.pubkey()),
            &[&*self.processor_keypair],
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    async fn process_spend(&self, game: &GameSettlementInfo, bet_id: &str) -> Result<String> {
        use solana_sdk::transaction::Transaction;
        use crate::solana_pda::{derive_casino_pda, derive_user_vault_pda, derive_latest_allowance_pda_from_nonce_registry};
        use crate::solana_instructions::build_spend_from_allowance_instruction;
        
        // Parse addresses
        let player_pubkey = game.player_address.parse()
            .context("Invalid player address")?;
        let vault_program_id = self.config.solana.vault_program_id.parse()?;

        // Derive PDAs
        let (casino_pda, _) = derive_casino_pda(&vault_program_id);
        let (user_vault_pda, _) = derive_user_vault_pda(&player_pubkey, &casino_pda, &vault_program_id);
        let (casino_vault, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"casino-vault", casino_pda.as_ref()],
            &vault_program_id,
        );
        let (vault_authority, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"vault-authority", casino_pda.as_ref()],
            &vault_program_id,
        );

        // Get client for allowance lookup
        let client = self.solana_client.get_client().await;
        
        // Derive allowance PDA
        let allowance = derive_latest_allowance_pda_from_nonce_registry(
            &*client,
            &vault_program_id,
            &player_pubkey,
            &casino_pda,
        ).context("Failed to derive allowance PDA")?;

        // Derive PDA for processed bet
        let (processed_bet_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"processed-bet", bet_id.as_bytes()],
            &vault_program_id,
        );

        // Build spend instruction
        let spend_ix = build_spend_from_allowance_instruction(
            &vault_program_id,
            &user_vault_pda,
            &casino_pda,
            &allowance,
            &processed_bet_pda,
            &casino_vault,
            &vault_authority,
            None, // user_token_account
            None, // casino_token_account
            &self.processor_keypair.pubkey(),
            game.bet_amount,
            bet_id,
        );

        // Get recent blockhash and send
        let recent_blockhash = client.get_latest_blockhash()?;
        
        let transaction = Transaction::new_signed_with_payer(
            &[spend_ix],
            Some(&self.processor_keypair.pubkey()),
            &[&*self.processor_keypair],
            recent_blockhash,
        );

        let signature = client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }
}
