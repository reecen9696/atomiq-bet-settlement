//! Bet repository trait and implementations
//!
//! Provides abstraction over bet storage with Redis implementation.

#[path = "../repository/redis_bet_repository/mod.rs"]
mod redis_bet_repository;

// Re-export everything publicly
pub use redis_bet_repository::RedisBetRepository;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{Bet, BetStatus, CreateBetRequest};
use crate::errors::Result;

/// Repository trait for bet storage and retrieval
#[async_trait]
pub trait BetRepository: Send + Sync {
    /// Create a new bet
    async fn create(&self, user_wallet: &str, vault_address: &str, req: CreateBetRequest) -> Result<Bet>;
    
    /// Find a bet by ID
    async fn find_by_id(&self, bet_id: Uuid) -> Result<Option<Bet>>;
    
    /// Find bets by user wallet with pagination
    async fn find_by_user(&self, user_wallet: &str, limit: i64, offset: i64) -> Result<Vec<Bet>>;
    
    /// Claim pending bets for batch processing
    async fn claim_pending(&self, limit: i64, processor_id: &str) -> Result<(Uuid, Vec<Bet>)>;
    
    /// Update bet status
    async fn update_status(&self, bet_id: Uuid, status: BetStatus, solana_tx_id: Option<String>) -> Result<()>;
    
    /// Update bet status with optimistic locking (compare-and-swap)
    async fn update_status_with_version(&self, bet_id: Uuid, expected_version: i32, status: BetStatus) -> Result<bool>;
}
