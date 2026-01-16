use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    domain::{Bet, UpdateBatchRequest},
    errors::{AppError, Result},
    repository::{bet_repository::BetRepository, PostgresBetRepository},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct PendingBetsQuery {
    pub limit: Option<i64>,
}

pub async fn get_pending_bets(
    State(state): State<AppState>,
    Query(query): Query<PendingBetsQuery>,
) -> Result<Json<Vec<Bet>>> {
    let limit = query.limit.unwrap_or(100).min(500);

    let repo = PostgresBetRepository::new(state.db.clone());
    let bets = repo.find_pending(limit).await?;

    metrics::gauge!("pending_bets_count", bets.len() as f64);

    Ok(Json(bets))
}

pub async fn update_batch(
    State(state): State<AppState>,
    Path(batch_id): Path<Uuid>,
    Json(req): Json<UpdateBatchRequest>,
) -> Result<Json<serde_json::Value>> {
    tracing::info!("Batch {} update received: {:?}", batch_id, req.status);

    // Update batch status in database
    let batch_status_str = match req.status {
        crate::domain::BatchStatus::Created => "created",
        crate::domain::BatchStatus::Submitted => "submitted",
        crate::domain::BatchStatus::Confirmed => "confirmed",
        crate::domain::BatchStatus::Failed => "failed",
    };

    sqlx::query!(
        r#"
        UPDATE batches
        SET status = $2::batch_status,
            solana_tx_id = COALESCE($3, solana_tx_id),
            last_error_message = $4
        WHERE batch_id = $1
        "#,
        batch_id,
        batch_status_str as _,
        req.solana_tx_id.as_deref(),
        req.error_message.as_deref()
    )
    .execute(&state.db)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to update batch: {}", e))
    .map_err(AppError::Internal)?;

    // Update individual bet statuses
    let repo = PostgresBetRepository::new(state.db.clone());
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

    metrics::counter!("batches_processed_total", 1);
    metrics::counter!("bets_updated_total", updated_count as u64);

    Ok(Json(serde_json::json!({
        "success": true,
        "batch_id": batch_id,
        "updated_count": updated_count,
        "error_count": error_count
    })))
}
