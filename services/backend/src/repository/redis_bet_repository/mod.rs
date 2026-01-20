//! Redis-based BetRepository implementation
//!
//! This module provides a Redis-backed implementation of the BetRepository trait
//! for storing and managing bets. It uses Redis hashes for bet storage and sorted
//! sets for indexing.

mod keys;
mod status;
mod retry;
mod lua_scripts;
mod deserialization;

use async_trait::async_trait;
use chrono::Utc;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Script};
use uuid::Uuid;

use crate::domain::{Bet, BetStatus, CreateBetRequest};
use crate::errors::Result;

// Re-export submodules
pub use keys::*;
pub use status::*;
pub use retry::*;
pub use lua_scripts::*;
pub use deserialization::*;

/// Redis-based implementation of BetRepository
pub struct RedisBetRepository {
    redis: ConnectionManager,
}

impl RedisBetRepository {
    /// Create a new RedisBetRepository
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    /// Update bet fields (won, payout_amount, error_message)
    ///
    /// This is a helper method for updating specific bet fields
    /// without changing the status.
    pub async fn update_bet_fields(
        &self,
        bet_id: Uuid,
        won: Option<bool>,
        payout_amount: Option<i64>,
        error_message: Option<String>,
    ) -> Result<()> {
        let mut redis_conn = self.redis.clone();
        let key = bet_key(bet_id);

        if let Some(won) = won {
            let _: () = redis_conn.hset(&key, "won", won.to_string()).await?;
        }
        if let Some(payout_amount) = payout_amount {
            let _: () = redis_conn
                .hset(&key, "payout_amount", payout_amount.to_string())
                .await?;
        }
        if let Some(error_message) = error_message {
            let _: () = redis_conn
                .hset(&key, "last_error_message", error_message)
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl super::BetRepository for RedisBetRepository {
    async fn create(&self, user_wallet: &str, vault_address: &str, req: CreateBetRequest) -> Result<Bet> {
        let bet_id = Uuid::new_v4();
        let now = Utc::now();
        let now_ms = now.timestamp_millis();

        // Convert LamportAmount to i64 for storage
        let stake_amount_i64 = req.stake_amount.as_u64() as i64;

        let bet = Bet {
            bet_id,
            created_at: now,
            user_wallet: user_wallet.to_string(),
            vault_address: vault_address.to_string(),
            allowance_pda: req.allowance_pda.clone().filter(|v| !v.is_empty()),
            casino_id: None,
            game_type: "coinflip".to_string(),
            stake_amount: stake_amount_i64,
            stake_token: req.stake_token,
            choice: req.choice,
            status: BetStatus::Pending,
            external_batch_id: None,
            solana_tx_id: None,
            retry_count: 0,
            processor_id: None,
            last_error_code: None,
            last_error_message: None,
            payout_amount: None,
            won: None,
        };

        let mut pipe = redis::pipe();
        pipe.atomic();

        let bet_key = bet_key(bet_id);
        let user_index = user_index_key(user_wallet);

        let mut redis_conn = self.redis.clone();

        let _: () = pipe
            .hset_multiple(
                &bet_key,
                &[
                    ("bet_id", bet.bet_id.to_string()),
                    ("created_at_ms", now_ms.to_string()),
                    ("user_wallet", bet.user_wallet.clone()),
                    ("vault_address", bet.vault_address.clone()),
                    ("allowance_pda", bet.allowance_pda.clone().unwrap_or_default()),
                    ("casino_id", "".to_string()),
                    ("game_type", bet.game_type.clone()),
                    ("stake_amount", bet.stake_amount.to_string()),
                    ("stake_token", bet.stake_token.clone()),
                    ("choice", bet.choice.clone()),
                    ("status", status_to_string(&bet.status)),
                    ("external_batch_id", "".to_string()),
                    ("solana_tx_id", "".to_string()),
                    ("retry_count", bet.retry_count.to_string()),
                    ("processor_id", "".to_string()),
                    ("last_error_code", "".to_string()),
                    ("last_error_message", "".to_string()),
                    ("payout_amount", "".to_string()),
                    ("won", "".to_string()),
                    ("version", "0".to_string()),
                ],
            )
            .ignore()
            .zadd(&user_index, bet.bet_id.to_string(), now_ms)
            .ignore()
            .zadd(claimable_index_key(), bet.bet_id.to_string(), now_ms)
            .ignore()
            .query_async(&mut redis_conn)
            .await?;

        Ok(bet)
    }

    async fn find_by_id(&self, bet_id: Uuid) -> Result<Option<Bet>> {
        let mut redis_conn = self.redis.clone();
        load_bet_from_hash(&mut redis_conn, bet_id).await
    }

    async fn find_by_user(&self, user_wallet: &str, limit: i64, offset: i64) -> Result<Vec<Bet>> {
        let mut redis_conn = self.redis.clone();
        let key = user_index_key(user_wallet);

        let start = offset.max(0) as isize;
        let end = (offset + limit - 1).max(-1) as isize;
        let bet_ids: Vec<String> = redis_conn.zrevrange(&key, start, end).await?;

        let mut bets = Vec::new();
        for id_str in bet_ids {
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if let Some(bet) = load_bet_from_hash(&mut redis_conn, id).await? {
                    bets.push(bet);
                }
            }
        }

        Ok(bets)
    }

    async fn claim_pending(&self, limit: i64, processor_id: &str) -> Result<(Uuid, Vec<Bet>)> {
        let limit = limit.max(0).min(500) as i64;
        let batch_id = Uuid::new_v4();

        let mut redis_conn = self.redis.clone();
        let script = Script::new(CLAIM_PENDING_SCRIPT);
        let now_ms = Utc::now().timestamp_millis();
        
        let claimed_ids: Vec<String> = script
            .key(claimable_index_key())
            .key(processing_index_key())
            .arg(limit)
            .arg(batch_id.to_string())
            .arg(processor_id)
            .arg(now_ms)
            .invoke_async(&mut redis_conn)
            .await?;

        let mut bets = Vec::new();
        for id_str in claimed_ids {
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if let Some(bet) = load_bet_from_hash(&mut redis_conn, id).await? {
                    bets.push(bet);
                }
            }
        }

        Ok((batch_id, bets))
    }

    async fn update_status(&self, bet_id: Uuid, status: BetStatus, solana_tx_id: Option<String>) -> Result<()> {
        let mut redis_conn = self.redis.clone();
        let bet_key_str = bet_key(bet_id);

        // Special handling: FailedRetryable implies retries + backoff and can graduate to manual review.
        if matches!(status, BetStatus::FailedRetryable) {
            let now_ms = Utc::now().timestamp_millis();
            let max_retries = max_retry_count();

            // We base the backoff on the *next* retry count (after increment).
            // Compute a conservative backoff using the current retry_count if present.
            // If missing, treat as first retry.
            let current_retry: i32 = redis_conn
                .hget(&bet_key_str, "retry_count")
                .await
                .unwrap_or(0);
            let backoff_ms = compute_backoff_ms(current_retry.saturating_add(1));

            let script = Script::new(FAIL_RETRYABLE_SCRIPT);
            let _: Vec<String> = script
                .key(&bet_key_str)
                .key(claimable_index_key())
                .key(processing_index_key())
                .arg(bet_id.to_string())
                .arg(now_ms)
                .arg(max_retries)
                .arg(backoff_ms)
                .invoke_async(&mut redis_conn)
                .await?;

            return Ok(());
        }

        let status_str = status_to_string(&status);
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.hset(&bet_key_str, "status", status_str).ignore();
        
        if let Some(tx) = solana_tx_id {
            pipe.hset(&bet_key_str, "solana_tx_id", tx).ignore();
        }

        // Clear stale error fields when transitioning out of failure states.
        match status {
            BetStatus::FailedRetryable | BetStatus::FailedManualReview => {}
            _ => {
                pipe.hset(&bet_key_str, "last_error_code", "").ignore();
                pipe.hset(&bet_key_str, "last_error_message", "").ignore();
            }
        }

        match status {
            BetStatus::FailedRetryable | BetStatus::Pending => {
                pipe.zadd(claimable_index_key(), bet_id.to_string(), Utc::now().timestamp_millis())
                    .ignore();
                pipe.zrem(processing_index_key(), bet_id.to_string()).ignore();
            }
            BetStatus::Batched => {
                pipe.zrem(claimable_index_key(), bet_id.to_string()).ignore();
                pipe.zadd(processing_index_key(), bet_id.to_string(), Utc::now().timestamp_millis())
                    .ignore();
            }
            _ => {
                pipe.zrem(claimable_index_key(), bet_id.to_string()).ignore();
                pipe.zrem(processing_index_key(), bet_id.to_string()).ignore();
            }
        }

        let _: () = pipe.query_async(&mut redis_conn).await?;
        Ok(())
    }

    async fn update_status_with_version(&self, bet_id: Uuid, expected_version: i32, status: BetStatus) -> Result<bool> {
        let mut redis_conn = self.redis.clone();
        let script = Script::new(CAS_UPDATE_SCRIPT);
        let updated: i32 = script
            .key(bet_key(bet_id))
            .arg(expected_version)
            .arg(status_to_string(&status))
            .invoke_async(&mut redis_conn)
            .await?;

        Ok(updated == 1)
    }
}
