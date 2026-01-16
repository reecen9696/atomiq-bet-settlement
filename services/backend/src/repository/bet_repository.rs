use async_trait::async_trait;
use chrono::Utc;
use chrono::TimeZone;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Script};
use uuid::Uuid;

use crate::domain::{Bet, BetStatus, CreateBetRequest};
use crate::errors::{AppError, Result};

#[async_trait]
pub trait BetRepository: Send + Sync {
    async fn create(&self, user_wallet: &str, vault_address: &str, req: CreateBetRequest) -> Result<Bet>;
    async fn find_by_id(&self, bet_id: Uuid) -> Result<Option<Bet>>;
    async fn find_by_user(&self, user_wallet: &str, limit: i64, offset: i64) -> Result<Vec<Bet>>;
    async fn claim_pending(&self, limit: i64, processor_id: &str) -> Result<(Uuid, Vec<Bet>)>;
    async fn update_status(&self, bet_id: Uuid, status: BetStatus, solana_tx_id: Option<String>) -> Result<()>;
    async fn update_status_with_version(&self, bet_id: Uuid, expected_version: i32, status: BetStatus) -> Result<bool>;
}

pub struct RedisBetRepository {
    redis: ConnectionManager,
}

impl RedisBetRepository {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    fn bet_key(bet_id: Uuid) -> String {
        format!("bet:{}", bet_id)
    }

    fn user_index_key(user_wallet: &str) -> String {
        format!("bets:user:{}", user_wallet)
    }

    fn claimable_index_key() -> &'static str {
        "bets:claimable"
    }

    fn processing_index_key() -> &'static str {
        "bets:processing"
    }

    fn status_to_string(status: &BetStatus) -> String {
        match status {
            BetStatus::Pending => "pending",
            BetStatus::Batched => "batched",
            BetStatus::SubmittedToSolana => "submitted_to_solana",
            BetStatus::ConfirmedOnSolana => "confirmed_on_solana",
            BetStatus::Completed => "completed",
            BetStatus::FailedRetryable => "failed_retryable",
            BetStatus::FailedManualReview => "failed_manual_review",
        }
        .to_string()
    }

    fn status_from_string(s: &str) -> Option<BetStatus> {
        match s {
            "pending" => Some(BetStatus::Pending),
            "batched" => Some(BetStatus::Batched),
            "submitted_to_solana" => Some(BetStatus::SubmittedToSolana),
            "confirmed_on_solana" => Some(BetStatus::ConfirmedOnSolana),
            "completed" => Some(BetStatus::Completed),
            "failed_retryable" => Some(BetStatus::FailedRetryable),
            "failed_manual_review" => Some(BetStatus::FailedManualReview),
            _ => None,
        }
    }

    async fn load_bet_from_hash(&self, bet_id: Uuid) -> Result<Option<Bet>> {
        let mut redis_conn = self.redis.clone();
        let key = Self::bet_key(bet_id);
        let map: std::collections::HashMap<String, String> = redis_conn.hgetall(&key).await?;
        if map.is_empty() {
            return Ok(None);
        }

        let created_at_ms: i64 = map
            .get("created_at_ms")
            .and_then(|v| v.parse::<i64>().ok())
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Invalid created_at_ms for bet {}", bet_id)))?;
        let created_at = Utc
            .timestamp_millis_opt(created_at_ms)
            .single()
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Invalid created_at_ms timestamp for bet {}", bet_id)))?;

        let status_str = map
            .get("status")
            .map(|s| s.as_str())
            .unwrap_or("pending");
        let status = Self::status_from_string(status_str)
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Invalid status '{}' for bet {}", status_str, bet_id)))?;

        let external_batch_id = map
            .get("external_batch_id")
            .and_then(|v| if v.is_empty() { None } else { Uuid::parse_str(v).ok() });

        let retry_count = map
            .get("retry_count")
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(0);

        let payout_amount = map
            .get("payout_amount")
            .and_then(|v| if v.is_empty() { None } else { v.parse::<i64>().ok() });
        let won = map
            .get("won")
            .and_then(|v| if v.is_empty() { None } else { v.parse::<bool>().ok() });

        Ok(Some(Bet {
            bet_id,
            created_at,
            user_wallet: map.get("user_wallet").cloned().unwrap_or_default(),
            vault_address: map.get("vault_address").cloned().unwrap_or_default(),
            casino_id: map.get("casino_id").cloned().filter(|v| !v.is_empty()),
            game_type: map.get("game_type").cloned().unwrap_or_else(|| "coinflip".to_string()),
            stake_amount: map
                .get("stake_amount")
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(0),
            stake_token: map.get("stake_token").cloned().unwrap_or_default(),
            choice: map.get("choice").cloned().unwrap_or_default(),
            status,
            external_batch_id,
            solana_tx_id: map.get("solana_tx_id").cloned().filter(|v| !v.is_empty()),
            retry_count,
            processor_id: map.get("processor_id").cloned().filter(|v| !v.is_empty()),
            last_error_code: map.get("last_error_code").cloned().filter(|v| !v.is_empty()),
            last_error_message: map.get("last_error_message").cloned().filter(|v| !v.is_empty()),
            payout_amount,
            won,
        }))
    }

    pub async fn update_bet_fields(
        &self,
        bet_id: Uuid,
        won: Option<bool>,
        payout_amount: Option<i64>,
        error_message: Option<String>,
    ) -> Result<()> {
        let mut redis_conn = self.redis.clone();
        let key = Self::bet_key(bet_id);

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
impl BetRepository for RedisBetRepository {
    async fn create(&self, user_wallet: &str, vault_address: &str, req: CreateBetRequest) -> Result<Bet> {
        let bet_id = Uuid::new_v4();
        let now = Utc::now();
        let now_ms = now.timestamp_millis();

        let bet = Bet {
            bet_id,
            created_at: now,
            user_wallet: user_wallet.to_string(),
            vault_address: vault_address.to_string(),
            casino_id: None,
            game_type: "coinflip".to_string(),
            stake_amount: req.stake_amount as i64,
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

        let bet_key = Self::bet_key(bet_id);
        let user_index = Self::user_index_key(user_wallet);

        let mut redis_conn = self.redis.clone();

        let _: () = pipe
            .hset_multiple(
                &bet_key,
                &[
                    ("bet_id", bet.bet_id.to_string()),
                    ("created_at_ms", now_ms.to_string()),
                    ("user_wallet", bet.user_wallet.clone()),
                    ("vault_address", bet.vault_address.clone()),
                    ("casino_id", "".to_string()),
                    ("game_type", bet.game_type.clone()),
                    ("stake_amount", bet.stake_amount.to_string()),
                    ("stake_token", bet.stake_token.clone()),
                    ("choice", bet.choice.clone()),
                    ("status", Self::status_to_string(&bet.status)),
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
            .zadd(Self::claimable_index_key(), bet.bet_id.to_string(), now_ms)
            .ignore()
            .query_async(&mut redis_conn)
            .await?;

        Ok(bet)
    }

    async fn find_by_id(&self, bet_id: Uuid) -> Result<Option<Bet>> {
        self.load_bet_from_hash(bet_id).await
    }

    async fn find_by_user(&self, user_wallet: &str, limit: i64, offset: i64) -> Result<Vec<Bet>> {
        let mut redis_conn = self.redis.clone();
        let key = Self::user_index_key(user_wallet);

        let start = offset.max(0) as isize;
        let end = (offset + limit - 1).max(-1) as isize;
        let bet_ids: Vec<String> = redis_conn.zrevrange(&key, start, end).await?;

        let mut bets = Vec::new();
        for id_str in bet_ids {
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if let Some(bet) = self.load_bet_from_hash(id).await? {
                    bets.push(bet);
                }
            }
        }

        Ok(bets)
    }

    async fn claim_pending(&self, limit: i64, processor_id: &str) -> Result<(Uuid, Vec<Bet>)> {
        let limit = limit.max(0).min(500) as i64;
        let batch_id = Uuid::new_v4();

        static CLAIM_LUA: &str = r#"
local claimable = KEYS[1]
local processing = KEYS[2]
local limit = tonumber(ARGV[1])
local batch_id = ARGV[2]
local processor_id = ARGV[3]

local entries = redis.call('ZRANGE', claimable, 0, limit - 1, 'WITHSCORES')
local claimed = {}

for i = 1, #entries, 2 do
  local bet_id = entries[i]
  local score = entries[i + 1]
  redis.call('ZREM', claimable, bet_id)
  redis.call('ZADD', processing, score, bet_id)
  redis.call('HSET', 'bet:' .. bet_id,
    'status', 'batched',
    'external_batch_id', batch_id,
    'processor_id', processor_id
  )
  table.insert(claimed, bet_id)
end

return claimed
"#;

        let mut redis_conn = self.redis.clone();
        let script = Script::new(CLAIM_LUA);
        let claimed_ids: Vec<String> = script
            .key(Self::claimable_index_key())
            .key(Self::processing_index_key())
            .arg(limit)
            .arg(batch_id.to_string())
            .arg(processor_id)
            .invoke_async(&mut redis_conn)
            .await?;

        let mut bets = Vec::new();
        for id_str in claimed_ids {
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if let Some(bet) = self.load_bet_from_hash(id).await? {
                    bets.push(bet);
                }
            }
        }

        Ok((batch_id, bets))
    }

    async fn update_status(&self, bet_id: Uuid, status: BetStatus, solana_tx_id: Option<String>) -> Result<()> {
        let mut redis_conn = self.redis.clone();
        let bet_key = Self::bet_key(bet_id);

        let status_str = Self::status_to_string(&status);
        let mut pipe = redis::pipe();
        pipe.atomic();
        pipe.hset(&bet_key, "status", status_str).ignore();
        if let Some(tx) = solana_tx_id {
            pipe.hset(&bet_key, "solana_tx_id", tx).ignore();
        }

        // Clear stale error fields when transitioning out of failure states.
        match status {
            BetStatus::FailedRetryable | BetStatus::FailedManualReview => {}
            _ => {
                pipe.hset(&bet_key, "last_error_code", "").ignore();
                pipe.hset(&bet_key, "last_error_message", "").ignore();
            }
        }

        match status {
            BetStatus::FailedRetryable | BetStatus::Pending => {
                pipe.zadd(Self::claimable_index_key(), bet_id.to_string(), Utc::now().timestamp_millis())
                    .ignore();
                pipe.zrem(Self::processing_index_key(), bet_id.to_string()).ignore();
            }
            BetStatus::Batched => {
                pipe.zrem(Self::claimable_index_key(), bet_id.to_string()).ignore();
                pipe.zadd(Self::processing_index_key(), bet_id.to_string(), Utc::now().timestamp_millis())
                    .ignore();
            }
            _ => {
                pipe.zrem(Self::claimable_index_key(), bet_id.to_string()).ignore();
                pipe.zrem(Self::processing_index_key(), bet_id.to_string()).ignore();
            }
        }

        let _: () = pipe.query_async(&mut redis_conn).await?;
        Ok(())
    }

    async fn update_status_with_version(&self, bet_id: Uuid, expected_version: i32, status: BetStatus) -> Result<bool> {
        static CAS_LUA: &str = r#"
local bet_key = KEYS[1]
local expected = tonumber(ARGV[1])
local new_status = ARGV[2]

local current = tonumber(redis.call('HGET', bet_key, 'version') or '0')
if current ~= expected then
  return 0
end

redis.call('HSET', bet_key, 'status', new_status)
redis.call('HINCRBY', bet_key, 'version', 1)
return 1
"#;

        let mut redis_conn = self.redis.clone();
        let script = Script::new(CAS_LUA);
        let updated: i32 = script
            .key(Self::bet_key(bet_id))
            .arg(expected_version)
            .arg(Self::status_to_string(&status))
            .invoke_async(&mut redis_conn)
            .await?;

        Ok(updated == 1)
    }
}


