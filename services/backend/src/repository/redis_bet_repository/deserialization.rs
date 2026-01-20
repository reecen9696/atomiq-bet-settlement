//! Deserialization of bets from Redis hash storage
//!
//! Handles parsing Redis hashes back into Bet domain objects.

use chrono::{TimeZone, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::Bet;
use crate::errors::{AppError, Result};
use super::keys::bet_key;
use super::status::status_from_string;

/// Load a bet from Redis hash storage
///
/// # Arguments
/// * `redis` - Redis connection manager
/// * `bet_id` - UUID of the bet to load
///
/// # Returns
/// * `Ok(Some(bet))` - Bet found and parsed successfully
/// * `Ok(None)` - Bet not found
/// * `Err(...)` - Redis error or parsing error
pub async fn load_bet_from_hash(
    redis: &mut ConnectionManager,
    bet_id: Uuid,
) -> Result<Option<Bet>> {
    let key = bet_key(bet_id);
    let map: HashMap<String, String> = redis.hgetall(&key).await?;
    
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
    let status = status_from_string(status_str)
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
        allowance_pda: map.get("allowance_pda").cloned().filter(|v| !v.is_empty()),
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
