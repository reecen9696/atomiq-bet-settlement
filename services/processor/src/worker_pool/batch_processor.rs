//! Batch processing orchestration
//!
//! Handles the full lifecycle of batch processing: fetch from blockchain, execute on Solana, update blockchain.

use anyhow::{Context, Result};
use reqwest::Client;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use std::str::FromStr;
use uuid::Uuid;

use crate::circuit_breaker::CircuitBreaker;
use crate::config::Config;
use crate::domain::Bet;
use crate::retry_strategy::RetryStrategy;
use crate::solana_client::SolanaClientPool;
use crate::blockchain_client::{BlockchainClient, GameSettlementInfo};

/// Orchestrates batch processing for a worker
#[derive(Clone)]
pub struct BatchProcessor {
    pub solana_client: Arc<SolanaClientPool>,
    pub processor_keypair: Arc<Keypair>,
    pub http: Client,
    pub retry_strategy: RetryStrategy,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub config: Config,
}

impl BatchProcessor {
    /// Process a single batch of settlements from blockchain
    pub async fn process_batch(&self, worker_id: usize) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Create span for entire batch processing lifecycle
        let span = tracing::info_span!(
            "process_batch",
            worker_id,
            processor_id = format!("worker-{}", worker_id)
        );
        let _enter = span.enter();

        // Create blockchain client
        let blockchain_client = BlockchainClient::new(
            self.config.blockchain.api_base_url.clone(),
            self.config.blockchain.api_key.clone(),
        );

        // Phase 1: Fetch pending settlements from blockchain
        let settlements = blockchain_client
            .fetch_pending_settlements(self.config.blockchain.settlement_batch_size)
            .await?;

        if settlements.is_empty() {
            tracing::trace!("No pending settlements to process");
            return Ok(());
        }

        tracing::info!(
            settlement_count = settlements.len(),
            max_bets_per_tx = self.config.processor.max_bets_per_tx,
            "Processing batch of pending settlements from blockchain"
        );

        metrics::gauge!("pending_settlements_fetched").set(settlements.len() as f64);

        // Phase 2: Split into chunks for Solana (max 12 bets per transaction)
        let max_per_tx = self.config.processor.max_bets_per_tx.max(1);

        for (chunk_idx, chunk) in settlements.chunks(max_per_tx).enumerate() {
            let chunk_span = tracing::info_span!(
                "process_chunk",
                chunk_idx,
                chunk_size = chunk.len()
            );
            let _chunk_enter = chunk_span.enter();

            // Convert settlements to Bet format
            let bets: Vec<Bet> = chunk
                .iter()
                .map(|s| self.settlement_to_bet(s))
                .collect::<Result<Vec<_>>>()?;

            // Execute on Solana
            let result = self.execute_settlements_on_solana(&bets).await;

            match result {
                Ok((signature, results)) => {
                    tracing::info!(
                        signature = %signature,
                        result_count = results.len(),
                        "Chunk executed successfully on Solana"
                    );

                    // Phase 3: Update settlement statuses on blockchain
                    for (settlement, (bet_id, won, payout)) in chunk.iter().zip(results.iter()) {
                        match blockchain_client
                            .update_settlement_status(
                                settlement.transaction_id,
                                "SettlementComplete",
                                Some(signature.clone()),
                                None, // No error on success
                                settlement.version,
                                None, // No retry on success
                                None, // No retry_after on success
                            )
                            .await
                        {
                            Ok(new_version) => {
                                tracing::info!(
                                    tx_id = settlement.transaction_id,
                                    bet_id = %bet_id,
                                    won,
                                    payout,
                                    new_version,
                                    signature = %signature,
                                    "Settlement completed and status updated on blockchain"
                                );
                            }
                            Err(e) => {
                                let error_str = e.to_string();
                                // If it's a version conflict, another worker already updated it - not critical
                                if error_str.contains("Version conflict") || error_str.contains("already processed") {
                                    tracing::warn!(
                                        tx_id = settlement.transaction_id,
                                        bet_id = %bet_id,
                                        signature = %signature,
                                        "Settlement already updated by another worker - skipping"
                                    );
                                    metrics::counter!("settlement_duplicate_processing_total").increment(1);
                                } else {
                                    tracing::error!(
                                        tx_id = settlement.transaction_id,
                                        bet_id = %bet_id,
                                        signature = %signature,
                                        error = %e,
                                        "CRITICAL: Failed to update settlement status (Solana succeeded but blockchain update failed)"
                                    );
                                    metrics::counter!("settlement_status_update_failures_total").increment(1);
                                }
                                // Continue processing other settlements even if one update fails
                            }
                        }
                    }

                    metrics::counter!("settlements_processed_total").increment(chunk.len() as u64);
                }
                Err(e) => {
                    tracing::error!(
                        chunk_idx,
                        chunk_size = chunk.len(),
                        error = %e,
                        "Settlement chunk failed on Solana"
                    );

                    // Update all settlements in this chunk as failed
                    for settlement in chunk {
                        let error_msg = format!("Solana transaction failed: {}", e);
                        
                        // Calculate retry logic: max 3 retries with 5s, 10s, 15s backoff
                        let new_retry_count = settlement.retry_count + 1;
                        let (status, next_retry_after) = if new_retry_count >= 3 {
                            ("SettlementFailedPermanent", None)
                        } else {
                            let backoff_seconds = (new_retry_count as i64) * 5;
                            let now_ms = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as i64;
                            let retry_after = now_ms + (backoff_seconds * 1000);
                            ("SettlementFailed", Some(retry_after))
                        };
                        
                        match blockchain_client
                            .update_settlement_status(
                                settlement.transaction_id,
                                status,
                                None,
                                Some(error_msg.clone()),
                                settlement.version,
                                Some(new_retry_count),
                                next_retry_after,
                            )
                            .await
                        {
                            Ok(new_version) => {
                                tracing::warn!(
                                    tx_id = settlement.transaction_id,
                                    new_version,
                                    error = %error_msg,
                                    "Settlement marked as failed on blockchain"
                                );
                            }
                            Err(update_err) => {
                                let error_str = update_err.to_string();
                                if error_str.contains("Version conflict") || error_str.contains("already processed") {
                                    tracing::warn!(
                                        tx_id = settlement.transaction_id,
                                        "Settlement already processed by another worker - skipping failure report"
                                    );
                                } else {
                                    tracing::error!(
                                        tx_id = settlement.transaction_id,
                                        solana_error = %e,
                                        update_error = %update_err,
                                        "CRITICAL: Failed to report settlement failure to blockchain API"
                                    );
                                    metrics::counter!("settlement_failure_report_errors_total").increment(1);
                                }
                            }
                        }
                    }

                    metrics::counter!("settlement_chunk_failures_total").increment(1);

                    // Stop processing this batch
                    return Err(e);
                }
            }
        }

        let elapsed = start_time.elapsed();
        tracing::info!(
            duration_ms = elapsed.as_millis(),
            settlement_count = settlements.len(),
            "Batch completed successfully"
        );

        metrics::histogram!("batch_processing_duration_seconds").record(elapsed.as_secs_f64());
        metrics::counter!("batches_processed_total").increment(1);

        Ok(())
    }

    /// Convert GameSettlementInfo to Bet format for Solana submission
    fn settlement_to_bet(&self, settlement: &GameSettlementInfo) -> Result<Bet> {
        Ok(Bet {
            bet_id: Uuid::new_v4(), // Generate UUID for tracking
            created_at: chrono::Utc::now(),
            user_wallet: settlement.player_address.clone(),
            vault_address: String::new(), // Will be derived in Solana tx building
            allowance_pda: settlement.allowance_pda.clone(), // Use allowance from blockchain
            casino_id: None,
            game_type: settlement.game_type.clone(),
            stake_amount: settlement.bet_amount as i64,
            stake_token: settlement.token.clone(),
            choice: "heads".to_string(), // Not relevant for settlements (already determined)
            status: crate::domain::BetStatus::Pending,
            external_batch_id: None,
            solana_tx_id: None,
            retry_count: 0,
            processor_id: None,
            last_error_code: None,
            last_error_message: None,
            payout_amount: Some(settlement.payout as i64),
            won: Some(settlement.outcome == "Win"),
        })
    }

    /// Execute settlements on Solana
    async fn execute_settlements_on_solana(
        &self,
        bets: &[Bet],
    ) -> Result<(String, Vec<(Uuid, bool, i64)>)> {
        let span = tracing::debug_span!(
            "execute_settlements_on_solana",
            bet_count = bets.len()
        );
        let _enter = span.enter();

        // Validate all user wallets are valid pubkeys
        for bet in bets {
            if solana_sdk::pubkey::Pubkey::from_str(&bet.user_wallet).is_err() {
                tracing::error!(
                    bet_id = %bet.bet_id,
                    user_wallet = %bet.user_wallet,
                    "Invalid user wallet pubkey"
                );
                return Err(anyhow::anyhow!(
                    "Invalid user wallet pubkey: {}",
                    bet.user_wallet
                ));
            }
        }

        // Get healthy Solana client
        let client = self
            .solana_client
            .get_healthy_client_or_any()
            .await
            .ok_or_else(|| anyhow::anyhow!("No RPC clients configured"))?;

        // Get vault program ID from environment
        let vault_program_id = solana_sdk::pubkey::Pubkey::from_str(
            &std::env::var("VAULT_PROGRAM_ID").context("VAULT_PROGRAM_ID not set")?
        )
        .context("Invalid VAULT_PROGRAM_ID")?;

        // Submit batch transaction to Solana
        tracing::info!(bet_count = bets.len(), "Submitting batch to Solana");
        crate::solana_tx::submit_batch_transaction(
            &client,
            bets,
            &self.processor_keypair,
            &vault_program_id,
            self.config.processor.max_bets_per_tx,
        )
        .await
    }
}