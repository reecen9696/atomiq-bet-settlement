use anyhow::Result;
use redis::aio::ConnectionManager;
use solana_sdk::signature::Keypair;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use uuid::Uuid;

use crate::batch_processor::BatchProcessor;
use crate::circuit_breaker::CircuitBreaker;
use crate::config::Config;
use crate::domain::{Bet, BetStatus};
use crate::retry_strategy::RetryStrategy;
use crate::solana_client::SolanaClientPool;

pub struct WorkerPool {
    config: Config,
    workers: Vec<Worker>,
    running: Arc<RwLock<bool>>,
}

struct Worker {
    id: usize,
    db_pool: PgPool,
    redis: ConnectionManager,
    solana_client: Arc<SolanaClientPool>,
    processor_keypair: Arc<Keypair>,
    batch_processor: BatchProcessor,
    retry_strategy: RetryStrategy,
    circuit_breaker: Arc<CircuitBreaker>,
    config: Config,
}

impl WorkerPool {
    pub fn new(
        config: Config,
        db_pool: PgPool,
        redis: ConnectionManager,
        solana_client: Arc<SolanaClientPool>,
        processor_keypair: Keypair,
    ) -> Self {
        let processor_keypair = Arc::new(processor_keypair);
        let mut workers = Vec::new();

        for id in 0..config.processor.worker_count {
            let circuit_breaker = Arc::new(CircuitBreaker::new(5, 60));
            let retry_strategy = RetryStrategy::new(config.processor.max_retries);
            let batch_processor = BatchProcessor::new(db_pool.clone());

            workers.push(Worker {
                id,
                db_pool: db_pool.clone(),
                redis: redis.clone(),
                solana_client: solana_client.clone(),
                processor_keypair: processor_keypair.clone(),
                batch_processor,
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
                db_pool: worker.db_pool.clone(),
                redis: worker.redis.clone(),
                solana_client: worker.solana_client.clone(),
                processor_keypair: worker.processor_keypair.clone(),
                batch_processor: BatchProcessor::new(worker.db_pool.clone()),
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
                metrics::counter!("worker_circuit_breaker_open_total", 1);
                continue;
            }

            // Process batch
            if let Err(e) = self.process_batch().await {
                tracing::error!("Worker {} batch processing error: {:?}", self.id, e);
                metrics::counter!("worker_errors_total", 1);
            }

            // Health check Solana RPC
            self.solana_client.health_check_all().await;
        }

        tracing::info!("Worker {} stopped", self.id);
        Ok(())
    }

    async fn process_batch(&self) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Fetch pending bets
        let pending_bets = self
            .batch_processor
            .fetch_pending_bets(self.config.processor.batch_size as i64)
            .await?;

        if pending_bets.is_empty() {
            return Ok(());
        }

        tracing::info!(
            "Worker {}: Processing {} pending bets",
            self.id,
            pending_bets.len()
        );

        metrics::gauge!("pending_bets_fetched", pending_bets.len() as f64);

        let bet_ids: Vec<Uuid> = pending_bets.iter().map(|b| b.bet_id).collect();

        // Phase 1: Create batch and lock bets atomically
        let (batch, locked_bets) = self
            .batch_processor
            .create_batch(format!("worker-{}", self.id), bet_ids)
            .await?;

        if locked_bets.is_empty() {
            tracing::warn!("Worker {}: No bets locked (race condition)", self.id);
            return Ok(());
        }

        // Phase 2: Execute bets on Solana (simulate coinflip for POC)
        let bet_results = self.execute_bets_on_solana(&locked_bets).await;

        match bet_results {
            Ok((signature, results)) => {
                // Phase 3: Update batch as submitted
                self.batch_processor
                    .update_batch_submitted(batch.batch_id, signature.clone())
                    .await?;

                // Phase 4: Confirm and complete
                self.batch_processor
                    .update_batch_confirmed(batch.batch_id, results)
                    .await?;

                let elapsed = start_time.elapsed();
                tracing::info!(
                    "Worker {}: Batch {} completed in {:?}",
                    self.id,
                    batch.batch_id,
                    elapsed
                );

                metrics::histogram!("batch_processing_duration_seconds", elapsed.as_secs_f64());
            }
            Err(e) => {
                tracing::error!(
                    "Worker {}: Batch {} failed: {:?}",
                    self.id,
                    batch.batch_id,
                    e
                );

                self.batch_processor
                    .update_batch_failed(batch.batch_id, e.to_string())
                    .await?;
            }
        }

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

use std::str::FromStr;