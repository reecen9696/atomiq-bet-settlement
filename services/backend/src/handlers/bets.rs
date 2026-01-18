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
    // Create a tracing span for the entire bet creation lifecycle
    let span = tracing::info_span!(
        "create_bet",
        bet.stake_amount = %req.stake_amount,
        bet.choice = %req.choice,
        bet.game_type = "coinflip"
    );
    let _enter = span.enter();

    // Use provided user_wallet or placeholder for testing
    // In production, extract from authenticated session
    let user_wallet = req.user_wallet.take().unwrap_or_else(|| "TEMP_WALLET_ADDRESS".to_string());
    let vault_address = req
        .vault_address
        .clone()
        .unwrap_or_else(|| "TEMP_VAULT_ADDRESS".to_string());

    tracing::debug!(
        user_wallet = %user_wallet,
        vault_address = %vault_address,
        "Creating bet"
    );

    // Validation is now handled by LamportAmount type during deserialization
    // No need for manual range checks

    let repo = RedisBetRepository::new(state.redis.clone());
    let bet = repo.create(&user_wallet, &vault_address, req).await?;

    tracing::info!(
        bet_id = %bet.bet_id,
        "Bet created successfully"
    );

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

    tracing::info!(
        bet_id = %bet.bet_id,
        "Published bet to Redis stream"
    );
    metrics::counter!("bets_created_total").increment(1);

    Ok(Json(CreateBetResponse { bet }))
}

pub async fn get_bet(
    State(state): State<AppState>,
    Path(bet_id): Path<Uuid>,
) -> Result<Json<Bet>> {
    let span = tracing::info_span!("get_bet", %bet_id);
    let _enter = span.enter();

    let repo = RedisBetRepository::new(state.redis.clone());
    let bet = repo
        .find_by_id(bet_id)
        .await?
        .ok_or_else(|| {
            tracing::debug!("Bet not found");
            AppError::not_found(format!("Bet {} not found", bet_id))
        })?;

    tracing::debug!(status = ?bet.status, "Bet retrieved");
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

    let span = tracing::info_span!(
        "list_user_bets",
        user_wallet = %user_wallet,
        limit,
        offset
    );
    let _enter = span.enter();

    let repo = RedisBetRepository::new(state.redis.clone());
    let bets = repo.find_by_user(&user_wallet, limit, offset).await?;

    tracing::debug!(bet_count = bets.len(), "Retrieved user bets");
    Ok(Json(bets))
}
