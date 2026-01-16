// Simplified Solana client module for testing without Solana dependencies
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SolanaClientPool {
    rpc_urls: Vec<String>,
    current_index: Arc<RwLock<usize>>,
}

impl SolanaClientPool {
    pub fn new(rpc_urls: Vec<String>) -> Self {
        Self {
            rpc_urls,
            current_index: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn get_client(&self) -> Result<String> {
        let index = self.current_index.read().await;
        Ok(self.rpc_urls.get(*index).unwrap_or(&self.rpc_urls[0]).clone())
    }

    pub async fn check_transaction_status(&self, _tx_id: &str) -> Result<TransactionStatus> {
        // Simulated for testing
        Ok(TransactionStatus::Confirmed)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}
