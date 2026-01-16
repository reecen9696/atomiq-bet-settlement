use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

impl Batch {
    pub fn new(processor_id: String, bet_count: i32) -> Self {
        Self {
            batch_id: Uuid::new_v4(),
            created_at: Utc::now(),
            processor_id,
            status: BatchStatus::Created,
            bet_count,
            solana_tx_id: None,
            confirm_slot: None,
            confirm_status: None,
            retry_count: 0,
            last_error_code: None,
            last_error_message: None,
        }
    }
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
pub struct UpdateBatchRequest {
    pub status: BatchStatus,
    pub solana_tx_id: Option<String>,
    pub bet_results: Vec<BetResult>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingBetsResponse {
    pub batch_id: Uuid,
    pub processor_id: String,
    pub bets: Vec<Bet>,
}
