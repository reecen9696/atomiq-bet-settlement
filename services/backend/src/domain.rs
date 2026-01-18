use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::LamportAmount;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BetStatus {
    Pending,
    Batched,
    SubmittedToSolana,
    ConfirmedOnSolana,
    Completed,
    FailedRetryable,
    FailedManualReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bet {
    pub bet_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub user_wallet: String,
    pub vault_address: String,
    pub allowance_pda: Option<String>,
    pub casino_id: Option<String>,
    pub game_type: String,
    pub stake_amount: i64,
    pub stake_token: String,
    pub choice: String,
    pub status: BetStatus,
    pub external_batch_id: Option<Uuid>,
    pub solana_tx_id: Option<String>,
    pub retry_count: i32,
    pub processor_id: Option<String>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub payout_amount: Option<i64>,
    pub won: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBetRequest {
    pub user_wallet: Option<String>,
    pub vault_address: Option<String>,
    pub allowance_pda: Option<String>,
    #[serde(deserialize_with = "deserialize_lamport_amount")]
    pub stake_amount: LamportAmount,
    pub stake_token: String,
    pub choice: String,
}

// Custom deserializer for LamportAmount from u64
fn deserialize_lamport_amount<'de, D>(deserializer: D) -> Result<LamportAmount, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let amount_u64 = u64::deserialize(deserializer)?;
    LamportAmount::try_from(amount_u64)
        .map_err(|e| serde::de::Error::custom(format!("Invalid stake amount: {}", e)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BatchStatus {
    Created,
    Submitted,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub batch_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub processor_id: String,
    pub status: BatchStatus,
    pub bet_count: i32,
    pub solana_tx_id: Option<String>,
    pub confirm_slot: Option<i64>,
    pub confirm_status: Option<String>,
    pub retry_count: i32,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBatchRequest {
    pub status: BatchStatus,
    pub solana_tx_id: Option<String>,
    pub bet_results: Vec<BetResult>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetResult {
    pub bet_id: Uuid,
    pub status: BetStatus,
    pub solana_tx_id: Option<String>,
    pub error_message: Option<String>,
    pub won: Option<bool>,
    pub payout_amount: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingBetsResponse {
    pub batch_id: Uuid,
    pub processor_id: String,
    pub bets: Vec<Bet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub event_time: DateTime<Utc>,
    pub event_type: String,
    pub aggregate_id: String,
    pub user_id: Option<String>,
    pub before_state: Option<serde_json::Value>,
    pub after_state: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub actor: String,
}
