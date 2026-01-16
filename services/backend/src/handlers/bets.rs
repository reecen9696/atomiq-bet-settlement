use axum::{
    extract::{Path, Query, State},
    Json,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{Bet, CreateBetRequest},
    errors::{AppError, Result},
    repository::{bet_repository::BetRepository, RedisBetRepository},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ListBetsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub user_wallet: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateBetResponse {
    pub bet: Bet,
}

pub async fn create_bet(
    State(state): State<AppState>,
    // TODO: Extract user_wallet from Privy authentication
    Json(mut req): Json<CreateBetRequest>,
) -> Result<Json<CreateBetResponse>> {
    // Use provided user_wallet or placeholder for testing
    // In production, extract from authenticated session
    let user_wallet = req.user_wallet.take().unwrap_or_else(|| "TEMP_WALLET_ADDRESS".to_string());
    let vault_address = "TEMP_VAULT_ADDRESS";

    // Validate bet amount
    if (req.stake_amount as i64) < state.config.betting.min_bet_lamports as i64
        || (req.stake_amount as i64) > state.config.betting.max_bet_lamports as i64
    {
        return Err(AppError::InvalidInput(
            "Bet amount outside allowed range".to_string(),
        ));
    }

    let repo = RedisBetRepository::new(state.redis.clone());
    let bet = repo.create(&user_wallet, vault_address, req).await?;

    // Publish to Redis stream for processor to pick up immediately
    let mut redis_conn = state.redis.clone();
    let _: String = redis_conn
        .xadd(
            "bets:pending",
            "*",
            &[("bet_id", bet.bet_id.to_string())],
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis publish failed: {}", e)))?;

    tracing::info!("Published bet {} to Redis stream", bet.bet_id);
    metrics::counter!("bets_created_total").increment(1);

    Ok(Json(CreateBetResponse { bet }))
}

pub async fn get_bet(
    State(state): State<AppState>,
    Path(bet_id): Path<Uuid>,
) -> Result<Json<Bet>> {
    let repo = RedisBetRepository::new(state.redis.clone());
    let bet = repo
        .find_by_id(bet_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Bet {} not found", bet_id)))?;

    Ok(Json(bet))
}

pub async fn list_user_bets(
    State(state): State<AppState>,
    Query(query): Query<ListBetsQuery>,
) -> Result<Json<Vec<Bet>>> {
    // TODO: Extract user_wallet from authentication. For POC, allow query override.
    let user_wallet = query
        .user_wallet
        .unwrap_or_else(|| "TEMP_WALLET_ADDRESS".to_string());

    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    let repo = RedisBetRepository::new(state.redis.clone());
    let bets = repo.find_by_user(&user_wallet, limit, offset).await?;

    Ok(Json(bets))
}
