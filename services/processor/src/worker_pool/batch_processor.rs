//! Batch processing orchestration
//!
//! Handles the full lifecycle of batch processing: fetch, execute, update.

use anyhow::Result;
use reqwest::Client;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use std::str::FromStr;
use uuid::Uuid;

use crate::circuit_breaker::CircuitBreaker;
use crate::config::Config;
use crate::domain::{BatchStatus, Bet, BetResult, BetStatus};
use crate::retry_strategy::RetryStrategy;
use crate::solana_client::SolanaClientPool;

use super::backend_client::BackendClient;
use crate::domain::UpdateBatchRequest;

/// Orchestrates batch processing for a worker
#[derive(Clone)]
pub struct BatchProcessor {
    pub solana_client: Arc<SolanaClientPool>,
    pub processor_keypair: Arc<Keypair>,
    pub http: Client,
    pub backend_base_url: String,
    pub retry_strategy: RetryStrategy,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub config: Config,
}

impl BatchProcessor {
    /// Process a single batch of bets
    pub async fn process_batch(&self, worker_id: usize) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Create span for entire batch processing lifecycle
        let span = tracing::info_span!(
            "process_batch",
            worker_id,
            processor_id = format!("worker-{}", worker_id)
        );
        let _enter = span.enter();

        // Create backend client
        let backend_client = BackendClient::new(self.backend_base_url.clone());

        // Phase 1: Fetch + claim pending bets from backend
        let processor_id = format!("worker-{}", worker_id);
        let claim_limit = self.config.processor.batch_size;
        
        let resp = backend_client
            .fetch_pending_bets(claim_limit, &processor_id)
            .await?;

        if resp.bets.is_empty() {
            tracing::trace!("No pending bets to process");
            return Ok(());
        }

        tracing::info!(
            batch_id = %resp.batch_id,
            bet_count = resp.bets.len(),
            max_bets_per_tx = self.config.processor.max_bets_per_tx,
            "Processing batch of pending bets"
        );

        metrics::gauge!("pending_bets_fetched").set(resp.bets.len() as f64);

        // Phase 2: Execute bets on Solana (simulate coinflip for POC)
        // Split into chunks so we don't exceed tx size/compute limits.
        let max_per_tx = self.config.processor.max_bets_per_tx.max(1);

        for (chunk_idx, chunk) in resp.bets.chunks(max_per_tx).enumerate() {
            let chunk_span = tracing::info_span!(
                "process_chunk",
                chunk_idx,
                chunk_size = chunk.len()
            );
            let _chunk_enter = chunk_span.enter();

            let bet_results = self.execute_bets_on_solana(chunk).await;

            match bet_results {
                Ok((signature, results)) => {
                    tracing::info!(
                        signature = %signature,
                        result_count = results.len(),
                        "Chunk executed successfully on Solana"
                    );

                    // Phase 3: Mark chunk submitted
                    backend_client.post_batch_update(
                        resp.batch_id,
                        UpdateBatchRequest {
                            status: BatchStatus::Submitted,
                            solana_tx_id: Some(signature.clone()),
                            error_message: None,
                            bet_results: chunk
                                .iter()
                                .map(|b| BetResult {
                                    bet_id: b.bet_id,
                                    status: BetStatus::SubmittedToSolana,
                                    solana_tx_id: Some(signature.clone()),
                                    error_message: None,
                                    won: None,
                                    payout_amount: None,
                                })
                                .collect(),
                        },
                    )
                    .await?;

                    // Phase 4: Mark chunk confirmed + bets completed
                    backend_client.post_batch_update(
                        resp.batch_id,
                        UpdateBatchRequest {
                            status: BatchStatus::Confirmed,
                            solana_tx_id: Some(signature.clone()),
                            error_message: None,
                            bet_results: results
                                .into_iter()
                                .map(|(bet_id, won, payout_amount)| {
                                    tracing::debug!(
                                        %bet_id,
                                        won,
                                        payout_amount,
                                        "Bet completed"
                                    );
                                    BetResult {
                                        bet_id,
                                        status: BetStatus::Completed,
                                        solana_tx_id: Some(signature.clone()),
                                        error_message: None,
                                        won: Some(won),
                                        payout_amount: Some(payout_amount),
                                    }
                                })
                                .collect(),
                        },
                    )
                    .await?;

                    tracing::info!(
                        signature = %signature,
                        "Chunk marked as confirmed"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        batch_id = %resp.batch_id,
                        chunk_idx,
                        error = %e,
                        "Batch chunk failed"
                    );

                    // Best-effort: mark this chunk retryable again.
                    let _ = backend_client
                        .post_batch_update(
                            resp.batch_id,
                            UpdateBatchRequest {
                                status: BatchStatus::Failed,
                                solana_tx_id: None,
                                error_message: Some(e.to_string()),
                                bet_results: chunk
                                    .iter()
                                    .map(|b| BetResult {
                                        bet_id: b.bet_id,
                                        status: BetStatus::FailedRetryable,
                                        solana_tx_id: None,
                                        error_message: Some(e.to_string()),
                                        won: None,
                                        payout_amount: None,
                                    })
                                    .collect(),
                            },
                        )
                        .await;

                    metrics::counter!("batch_chunk_failures_total").increment(1);

                    // Stop this worker's batch loop early; remaining bets will be re-claimed later.
                    return Err(e);
                }
            }
        }

        let elapsed = start_time.elapsed();
        tracing::info!(
            batch_id = %resp.batch_id,
            duration_ms = elapsed.as_millis(),
            bet_count = resp.bets.len(),
            "Batch completed successfully"
        );

        metrics::histogram!("batch_processing_duration_seconds").record(elapsed.as_secs_f64());
        metrics::counter!("batches_processed_total").increment(1);

        Ok(())
    }

    /// Execute bets on Solana (or simulate for testing)
    async fn execute_bets_on_solana(
        &self,
        bets: &[Bet],
    ) -> Result<(String, Vec<(Uuid, bool, i64)>)> {
        let span = tracing::debug_span!(
            "execute_bets_on_solana",
            bet_count = bets.len()
        );
        let _enter = span.enter();

        // Always use real Solana transactions (production mode)
        // If any bet has an invalid pubkey, this is a configuration error that should be fixed
        for bet in bets {
            if solana_sdk::pubkey::Pubkey::from_str(&bet.user_wallet).is_err() {
                tracing::error!(
                    bet_id = %bet.bet_id,
                    user_wallet = %bet.user_wallet,
                    "Invalid user wallet pubkey - this is a configuration error"
                );
                return Err(anyhow::anyhow!(
                    "Invalid user wallet pubkey: {}. Check bet validation.",
                    bet.user_wallet
                ));
                }
            }

            // Real Solana transaction
            tracing::info!("Submitting real Solana transaction");
            let client = self
                .solana_client
                .get_healthy_client_or_any()
                .await
                .ok_or_else(|| anyhow::anyhow!("No RPC clients configured"))?;
            
            let vault_program_id = solana_sdk::pubkey::Pubkey::from_str(
                &std::env::var("VAULT_PROGRAM_ID")?
            )?;
            
            crate::solana_tx::submit_batch_transaction(
                &client,
                bets,
                &self.processor_keypair,
                &vault_program_id,
                self.config.processor.max_bets_per_tx,
            ).await
    }
}