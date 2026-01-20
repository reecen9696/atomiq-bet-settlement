//! Individual worker implementation
//!
//! Handles batch processing for a single worker thread.

use anyhow::Result;
use reqwest::Client;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::circuit_breaker::CircuitBreaker;
use crate::config::Config;
use crate::retry_strategy::RetryStrategy;
use crate::solana_client::SolanaClientPool;

use super::batch_processor::BatchProcessor;

/// Individual worker for processing bets
#[derive(Clone)]
pub struct Worker {
    pub id: usize,
    batch_processor: BatchProcessor,
}

impl Worker {
    /// Create a new worker
    pub fn new(
        id: usize,
        config: Config,
        solana_client: Arc<SolanaClientPool>,
        processor_keypair: Arc<Keypair>,
    ) -> Self {
        let http = Client::new();
        let backend_base_url = config.backend.api_base_url.trim_end_matches('/').to_string();
        let circuit_breaker = Arc::new(CircuitBreaker::new(5, 60));
        let retry_strategy = RetryStrategy::new(config.processor.max_retries);

        let batch_processor = BatchProcessor {
            solana_client,
            processor_keypair,
            http,
            backend_base_url,
            retry_strategy,
            circuit_breaker,
            config,
        };

        Self {
            id,
            batch_processor,
        }
    }

    /// Run the worker's main processing loop
    pub async fn run(&self, running: Arc<RwLock<bool>>) -> Result<()> {
        tracing::info!("Worker {} started", self.id);

        let mut ticker = interval(Duration::from_secs(
            self.batch_processor.config.processor.batch_interval_seconds
        ));

        loop {
            ticker.tick().await;

            let is_running = *running.read().await;
            if !is_running {
                break;
            }

            // Check circuit breaker
            if self.batch_processor.circuit_breaker.is_open().await {
                tracing::warn!("Worker {}: Circuit breaker is open, skipping batch", self.id);
                metrics::counter!("worker_circuit_breaker_open_total").increment(1);
                continue;
            }

            // Process batch
            if let Err(e) = self.batch_processor.process_batch(self.id).await {
                tracing::error!("Worker {} batch processing error: {:?}", self.id, e);
                metrics::counter!("worker_errors_total").increment(1);
            }

            // Health check Solana RPC
            self.batch_processor.solana_client.health_check_all().await;
        }

        tracing::info!("Worker {} stopped", self.id);
        Ok(())
    }
}