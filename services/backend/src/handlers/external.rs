use axum::{
    extract::{Path, Query, State},
    Json,
};
use redis::AsyncCommands;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    domain::{PendingBetsResponse, UpdateBatchRequest},
    errors::{AppError, Result},
    repository::{bet_repository::BetRepository, RedisBetRepository},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct PendingBetsQuery {
    pub limit: Option<i64>,
    pub processor_id: Option<String>,
}

pub async fn get_pending_bets(
    State(state): State<AppState>,
    Query(query): Query<PendingBetsQuery>,
) -> Result<Json<PendingBetsResponse>> {
    let limit = query.limit.unwrap_or(100).min(500);
    let processor_id = query
        .processor_id
        .unwrap_or_else(|| "processor-unknown".to_string());

    let repo = RedisBetRepository::new(state.redis.clone());
    let (batch_id, bets) = repo.claim_pending(limit, &processor_id).await?;

    metrics::gauge!("pending_bets_count").set(bets.len() as f64);

    Ok(Json(PendingBetsResponse {
        batch_id,
        processor_id,
        bets,
    }))
}

pub async fn update_batch(
    State(state): State<AppState>,
    Path(batch_id): Path<Uuid>,
    Json(req): Json<UpdateBatchRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Batch {} update received: {:?}", batch_id, req.status);

    // Store batch summary in Redis (best-effort)
    {
        let mut redis_conn = state.redis.clone();
        let batch_key = format!("batch:{}", batch_id);
        let _: () = redis_conn
            .hset_multiple(
                &batch_key,
                &[
                    ("status", format!("{:?}", req.status).to_lowercase()),
                    ("solana_tx_id", req.solana_tx_id.clone().unwrap_or_default()),
                    ("last_error_message", req.error_message.clone().unwrap_or_default()),
                    ("updated_at_ms", chrono::Utc::now().timestamp_millis().to_string()),
                ],
            )
            .await
            .map_err(AppError::Redis)?;
    }

    // Update individual bet statuses
    let repo = RedisBetRepository::new(state.redis.clone());
    let mut updated_count = 0;
    let mut error_count = 0;

    for bet_result in req.bet_results {
        let bet_id = bet_result.bet_id;
        let status = bet_result.status.clone();
        match repo
            .update_status(bet_id, bet_result.status, bet_result.solana_tx_id)
            .await
        {
            Ok(_) => {
                // Optional result fields (POC: store for UI/status queries)
                let _ = repo
                    .update_bet_fields(
                        bet_id,
                        bet_result.won,
                        bet_result.payout_amount,
                        bet_result.error_message,
                    )
                    .await;
                updated_count += 1;
                tracing::debug!("Updated bet {} to {:?}", bet_id, status);
            }
            Err(e) => {
                error_count += 1;
                tracing::error!("Failed to update bet {}: {}", bet_id, e);
            }
        }
    }

    tracing::info!(
        "Batch {} processed: {} bets updated, {} errors",
        batch_id,
        updated_count,
        error_count
    );

    metrics::counter!("batches_processed_total").increment(1);
    metrics::counter!("bets_updated_total").increment(updated_count as u64);

    Ok(Json(serde_json::json!({
        "success": true,
        "batch_id": batch_id,
        "updated_count": updated_count,
        "error_count": error_count
    })))
}
