use anyhow::Result;
use reqwest::Client;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use std::str::FromStr;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use uuid::Uuid;

use crate::circuit_breaker::CircuitBreaker;
use crate::config::Config;
use crate::domain::{
    BatchStatus, Bet, BetResult, BetStatus, PendingBetsResponse, UpdateBatchRequest,
};
use crate::retry_strategy::RetryStrategy;
use crate::solana_client::SolanaClientPool;

pub struct WorkerPool {
    config: Config,
    workers: Vec<Worker>,
    running: Arc<RwLock<bool>>,
}

struct Worker {
    id: usize,
    solana_client: Arc<SolanaClientPool>,
    processor_keypair: Arc<Keypair>,
    http: Client,
    backend_base_url: String,
    retry_strategy: RetryStrategy,
    circuit_breaker: Arc<CircuitBreaker>,
    config: Config,
}

impl WorkerPool {
    pub fn new(
        config: Config,
        solana_client: Arc<SolanaClientPool>,
        processor_keypair: Keypair,
    ) -> Self {
        let processor_keypair = Arc::new(processor_keypair);
        let mut workers = Vec::new();

        let http = Client::new();
        let backend_base_url = config.backend.api_base_url.trim_end_matches('/').to_string();

        for id in 0..config.processor.worker_count {
            let circuit_breaker = Arc::new(CircuitBreaker::new(5, 60));
            let retry_strategy = RetryStrategy::new(config.processor.max_retries);

            workers.push(Worker {
                id,
                solana_client: solana_client.clone(),
                processor_keypair: processor_keypair.clone(),
                http: http.clone(),
                backend_base_url: backend_base_url.clone(),
                retry_strategy,
                circuit_breaker,
                config: config.clone(),
            });
        }

        Self {
            config,
            workers,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        tracing::info!("Starting {} workers", self.workers.len());

        let mut handles = Vec::new();

        for worker in &self.workers {
            let worker_clone = Worker {
                id: worker.id,
                solana_client: worker.solana_client.clone(),
                processor_keypair: worker.processor_keypair.clone(),
                http: worker.http.clone(),
                backend_base_url: worker.backend_base_url.clone(),
                retry_strategy: RetryStrategy::new(worker.config.processor.max_retries),
                circuit_breaker: worker.circuit_breaker.clone(),
                config: worker.config.clone(),
            };

            let running = self.running.clone();

            let handle = tokio::spawn(async move {
                worker_clone.run(running).await
            });

            handles.push(handle);
        }

        // Wait for all workers
        for handle in handles {
            if let Err(e) = handle.await {
                tracing::error!("Worker error: {:?}", e);
            }
        }

        Ok(())
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Stopping worker pool");
    }
}

impl Worker {
    async fn run(&self, running: Arc<RwLock<bool>>) -> Result<()> {
        tracing::info!("Worker {} started", self.id);

        let mut ticker = interval(Duration::from_secs(self.config.processor.batch_interval_seconds));

        loop {
            ticker.tick().await;

            let is_running = *running.read().await;
            if !is_running {
                break;
            }

            // Check circuit breaker
            if self.circuit_breaker.is_open().await {
                tracing::warn!("Worker {}: Circuit breaker is open, skipping batch", self.id);
                metrics::counter!("worker_circuit_breaker_open_total").increment(1);
                continue;
            }

            // Process batch
            if let Err(e) = self.process_batch().await {
                tracing::error!("Worker {} batch processing error: {:?}", self.id, e);
                metrics::counter!("worker_errors_total").increment(1);
            }

            // Health check Solana RPC
            self.solana_client.health_check_all().await;
        }

        tracing::info!("Worker {} stopped", self.id);
        Ok(())
    }

    async fn process_batch(&self) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Fetch + claim pending bets from backend (POC: local emulation of Atomiq contract)
        let processor_id = format!("worker-{}", self.id);
        let url = format!("{}/api/external/bets/pending", self.backend_base_url);

        let resp: PendingBetsResponse = self
            .http
            .get(url)
            .query(&[
                ("limit", self.config.processor.batch_size.to_string()),
                ("processor_id", processor_id.clone()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        if resp.bets.is_empty() {
            return Ok(());
        }

        tracing::info!(
            "Worker {}: Processing {} pending bets",
            self.id,
            resp.bets.len()
        );

        metrics::gauge!("pending_bets_fetched").set(resp.bets.len() as f64);

        // Phase 2: Execute bets on Solana (simulate coinflip for POC)
        let bet_results = self.execute_bets_on_solana(&resp.bets).await;

        match bet_results {
            Ok((signature, results)) => {
                // Phase 3: Mark batch submitted
                self.post_batch_update(
                    resp.batch_id,
                    UpdateBatchRequest {
                        status: BatchStatus::Submitted,
                        solana_tx_id: Some(signature.clone()),
                        error_message: None,
                        bet_results: resp
                            .bets
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

                // Phase 4: Mark batch confirmed + bets completed
                self.post_batch_update(
                    resp.batch_id,
                    UpdateBatchRequest {
                        status: BatchStatus::Confirmed,
                        solana_tx_id: Some(signature.clone()),
                        error_message: None,
                        bet_results: results
                            .into_iter()
                            .map(|(bet_id, won, payout_amount)| BetResult {
                                bet_id,
                                status: BetStatus::Completed,
                                solana_tx_id: Some(signature.clone()),
                                error_message: None,
                                won: Some(won),
                                payout_amount: Some(payout_amount),
                            })
                            .collect(),
                    },
                )
                .await?;

                let elapsed = start_time.elapsed();
                tracing::info!(
                    "Worker {}: Batch {} completed in {:?}",
                    self.id,
                    resp.batch_id,
                    elapsed
                );

                metrics::histogram!("batch_processing_duration_seconds").record(elapsed.as_secs_f64());
            }
            Err(e) => {
                tracing::error!("Worker {}: Batch {} failed: {:?}", self.id, resp.batch_id, e);

                // Best-effort: mark bets retryable again
                let _ = self
                    .post_batch_update(
                        resp.batch_id,
                        UpdateBatchRequest {
                            status: BatchStatus::Failed,
                            solana_tx_id: None,
                            error_message: Some(e.to_string()),
                            bet_results: resp
                                .bets
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
            }
        }

        Ok(())
    }

    async fn post_batch_update(&self, batch_id: Uuid, req: UpdateBatchRequest) -> Result<()> {
        let url = format!("{}/api/external/batches/{}", self.backend_base_url, batch_id);
        self.http
            .post(url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    async fn execute_bets_on_solana(
        &self,
        bets: &[Bet],
    ) -> Result<(String, Vec<(Uuid, bool, i64)>)> {
        // Check if we should use real Solana transactions
        let use_real_solana = std::env::var("USE_REAL_SOLANA")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        if use_real_solana {
            // If any bet has an invalid pubkey (common in local/POC calls), fall back to simulation
            // instead of thrashing the queue.
            for bet in bets {
                if solana_sdk::pubkey::Pubkey::from_str(&bet.user_wallet).is_err() {
                    tracing::warn!(
                        "Invalid user wallet pubkey for bet {} ({}); falling back to simulation",
                        bet.bet_id,
                        bet.user_wallet
                    );
                    return self.simulate_bets(bets).await;
                }
            }

            // Real Solana transaction
            let client = self.solana_client.get_healthy_client().await
                .ok_or_else(|| anyhow::anyhow!("No healthy RPC clients available"))?;
            
            let vault_program_id = solana_sdk::pubkey::Pubkey::from_str(
                &std::env::var("VAULT_PROGRAM_ID")?
            )?;

            tracing::info!("Submitting {} bets to Solana", bets.len());
            
            crate::solana_tx::submit_batch_transaction(
                &client,
                bets,
                &self.processor_keypair,
                &vault_program_id,
            ).await
        } else {
            // Simulated transaction for testing
            self.simulate_bets(bets).await
        }
    }

    async fn simulate_bets(
        &self,
        bets: &[Bet],
    ) -> Result<(String, Vec<(Uuid, bool, i64)>)> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let mut results = Vec::new();
        
        for bet in bets {
            // Simulate coinflip outcome
            let won = rng.gen_bool(0.5);
            let payout = if won {
                bet.stake_amount * 2 // 2x payout for winning
            } else {
                0
            };
            
            results.push((bet.bet_id, won, payout));
            
            tracing::debug!(
                "Bet {}: {} -> {}",
                bet.bet_id,
                bet.choice,
                if won { "WON" } else { "LOST" }
            );
        }

        // Simulate Solana transaction submission
        let signature = format!("SIM_{}", Uuid::new_v4());

        tracing::info!("Simulated Solana transaction: {}", signature);
        
        Ok((signature, results))
    }
}