//! Settlement Coordinator
//! 
//! Fetches all pending settlements from blockchain API and distributes to workers
//! via channels. Prevents duplicate processing and enables efficient batching.

use crate::{
    blockchain_client::{BlockchainClient, GameSettlementInfo},
    config::Config,
};
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Work unit sent from coordinator to workers
#[derive(Debug, Clone)]
pub struct SettlementBatch {
    pub batch_id: String,
    pub settlements: Vec<GameSettlementInfo>,
    pub batch_type: BatchType,
}

/// Type of settlement batch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchType {
    Payout,  // Win - pay from casino vault to user
    Spend,   // Loss - spend from user's allowance to casino
}

pub struct Coordinator {
    blockchain_client: Arc<BlockchainClient>,
    work_senders: Vec<mpsc::Sender<SettlementBatch>>,
    config: Config,
    next_worker_index: std::sync::atomic::AtomicUsize,
}

impl Coordinator {
    pub fn new(
        blockchain_client: Arc<BlockchainClient>,
        work_senders: Vec<mpsc::Sender<SettlementBatch>>,
        config: Config,
    ) -> Self {
        Self {
            blockchain_client,
            work_senders,
            config,
            next_worker_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Main coordinator loop - fetches and distributes work
    pub async fn run(&self) {
        let poll_interval = Duration::from_secs(self.config.blockchain.poll_interval_seconds);
        
        info!(
            poll_interval_seconds = self.config.blockchain.poll_interval_seconds,
            worker_count = self.work_senders.len(),
            batch_min = self.config.processor.coordinator_batch_min_size,
            batch_max = self.config.processor.coordinator_batch_max_size,
            "Coordinator starting"
        );

        loop {
            let cycle_start = std::time::Instant::now();
            
            if let Err(e) = self.process_cycle().await {
                error!(error = %e, "Coordinator cycle failed");
            }

            let elapsed = cycle_start.elapsed();
            info!(
                cycle_duration_ms = elapsed.as_millis(),
                "Coordinator cycle completed"
            );

            sleep(poll_interval).await;
        }
    }

    async fn process_cycle(&self) -> Result<()> {
        // 1. Fetch all pending settlements
        let settlements = self.fetch_all_pending().await?;

        if settlements.is_empty() {
            debug!("No pending settlements found");
            return Ok(());
        }

        info!(
            total_settlements = settlements.len(),
            "Fetched pending settlements"
        );

        // 2. Group by outcome type (Win vs Loss)
        let (wins, losses) = self.group_by_outcome(settlements);
        
        info!(
            wins = wins.len(),
            losses = losses.len(),
            "Grouped settlements by outcome"
        );

        // 3. Create batches
        let win_batches = self.create_batches(wins, BatchType::Payout);
        let loss_batches = self.create_batches(losses, BatchType::Spend);

        info!(
            win_batches = win_batches.len(),
            loss_batches = loss_batches.len(),
            total_batches = win_batches.len() + loss_batches.len(),
            "Created settlement batches"
        );

        // 4. Distribute to workers (round-robin)
        let mut distributed = 0;
        
        for batch in win_batches.into_iter().chain(loss_batches.into_iter()) {
            if let Err(e) = self.send_to_worker(batch).await {
                error!(error = %e, "Failed to send batch to worker");
            } else {
                distributed += 1;
            }
        }

        info!(
            distributed_batches = distributed,
            "Work distribution completed"
        );

        Ok(())
    }

    /// Fetch all pending settlements from blockchain API
    async fn fetch_all_pending(&self) -> Result<Vec<GameSettlementInfo>> {
        // Fetch larger batch size to get all pending
        let limit = self.config.blockchain.settlement_batch_size;
        
        self.blockchain_client
            .fetch_pending_settlements(limit)
            .await
            .context("Failed to fetch pending settlements")
    }

    /// Group settlements by outcome type
    fn group_by_outcome(&self, settlements: Vec<GameSettlementInfo>) -> (Vec<GameSettlementInfo>, Vec<GameSettlementInfo>) {
        let mut wins = Vec::new();
        let mut losses = Vec::new();

        for settlement in settlements {
            match settlement.outcome.as_str() {
                "Win" => wins.push(settlement),
                "Loss" => losses.push(settlement),
                other => {
                    warn!(
                        tx_id = settlement.transaction_id,
                        outcome = other,
                        "Unknown outcome type, skipping"
                    );
                }
            }
        }

        (wins, losses)
    }

    /// Create batches from settlements
    /// 
    /// Strategy:
    /// - Min batch size: 3 (amortize TX cost)
    /// - Max batch size: 12 (Solana TX size limit)
    /// - Optimal: 8 (balance cost vs blast radius)
    fn create_batches(&self, settlements: Vec<GameSettlementInfo>, batch_type: BatchType) -> Vec<SettlementBatch> {
        if settlements.is_empty() {
            return Vec::new();
        }

        let min_size = self.config.processor.coordinator_batch_min_size;
        let max_size = self.config.processor.coordinator_batch_max_size;
        
        let mut batches = Vec::new();
        let mut current_batch = Vec::new();

        for settlement in settlements {
            current_batch.push(settlement);

            // Create batch when we hit max size
            if current_batch.len() >= max_size {
                batches.push(SettlementBatch {
                    batch_id: Uuid::new_v4().to_string(),
                    settlements: current_batch.clone(),
                    batch_type,
                });
                current_batch.clear();
            }
        }

        // Handle remaining settlements
        if !current_batch.is_empty() {
            if current_batch.len() >= min_size || batches.is_empty() {
                // Create batch if we have enough or it's the only batch
                batches.push(SettlementBatch {
                    batch_id: Uuid::new_v4().to_string(),
                    settlements: current_batch,
                    batch_type,
                });
            } else {
                // Merge with last batch if too small
                if let Some(last_batch) = batches.last_mut() {
                    last_batch.settlements.extend(current_batch);
                } else {
                    // No batches yet, create one anyway
                    batches.push(SettlementBatch {
                        batch_id: Uuid::new_v4().to_string(),
                        settlements: current_batch,
                        batch_type,
                    });
                }
            }
        }

        debug!(
            batch_count = batches.len(),
            batch_type = ?batch_type,
            avg_size = if batches.is_empty() { 0 } else { 
                batches.iter().map(|b| b.settlements.len()).sum::<usize>() / batches.len()
            },
            "Created batches"
        );

        batches
    }

    /// Send batch to next available worker (round-robin)
    async fn send_to_worker(&self, batch: SettlementBatch) -> Result<()> {
        let worker_index = self.next_worker_index
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.work_senders.len();

        let sender = &self.work_senders[worker_index];
        let batch_id = batch.batch_id.clone();
        let settlement_count = batch.settlements.len();

        sender
            .send(batch)
            .await
            .context("Failed to send batch to worker")?;

        debug!(
            worker_index,
            batch_id = %batch_id,
            settlement_count,
            "Batch sent to worker"
        );

        Ok(())
    }
}
