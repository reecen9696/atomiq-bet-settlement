//! Worker pool management
//!
//! Manages multiple workers for parallel bet processing.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::solana_client::SolanaClientPool;
use solana_sdk::signature::Keypair;

use super::worker::Worker;

/// Pool of workers for processing bets
pub struct WorkerPool {
    config: Config,
    workers: Vec<Worker>,
    running: Arc<RwLock<bool>>,
}

impl WorkerPool {
    /// Create a new worker pool
    pub fn new(
        config: Config,
        solana_client: Arc<SolanaClientPool>,
        processor_keypair: Keypair,
    ) -> Self {
        let processor_keypair = Arc::new(processor_keypair);
        let mut workers = Vec::new();

        for id in 0..config.processor.worker_count {
            workers.push(Worker::new(
                id,
                config.clone(),
                solana_client.clone(),
                processor_keypair.clone(),
            ));
        }

        Self {
            config,
            workers,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start all workers
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        tracing::info!("Starting {} workers", self.workers.len());

        let mut handles = Vec::new();

        for worker in &self.workers {
            let worker_clone = worker.clone();
            let running_clone = self.running.clone();
            
            let handle = tokio::spawn(async move {
                worker_clone.run(running_clone).await
            });
            
            handles.push(handle);
        }

        // Join all worker tasks
        for handle in handles {
            let _ = handle.await?;
        }

        Ok(())
    }

    /// Stop all workers
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Stopping worker pool");
    }
}