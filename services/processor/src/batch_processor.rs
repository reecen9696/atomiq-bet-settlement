use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Batch, BatchStatus, Bet, BetStatus};

pub struct BatchProcessor {
    db_pool: PgPool,
}

impl BatchProcessor {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Phase 1: Lock pending bets into a batch (atomic operation)
    pub async fn create_batch(
        &self,
        processor_id: String,
        bet_ids: Vec<Uuid>,
    ) -> Result<(Batch, Vec<Bet>)> {
        let mut tx = self.db_pool.begin().await?;

        // Create batch record
        let batch = Batch::new(processor_id.clone(), bet_ids.len() as i32);
        
        sqlx::query!(
            r#"
            INSERT INTO batches (
                batch_id, created_at, processor_id, status, bet_count
            )
            VALUES ($1, $2, $3, 'created', $4)
            "#,
            batch.batch_id,
            batch.created_at,
            processor_id,
            batch.bet_count
        )
        .execute(&mut *tx)
        .await?;

        // Lock bets atomically (only if still pending)
        let updated_bets = sqlx::query_as!(
            Bet,
            r#"
            UPDATE bets
            SET 
                status = 'batched',
                external_batch_id = $1,
                processor_id = $2
            WHERE bet_id = ANY($3) AND status = 'pending'
            RETURNING
                bet_id, created_at, user_wallet, vault_address, casino_id,
                game_type, stake_amount, stake_token, choice,
                status as "status: BetStatus",
                external_batch_id, solana_tx_id, retry_count, processor_id,
                last_error_code, last_error_message, payout_amount, won
            "#,
            batch.batch_id,
            processor_id,
            &bet_ids
        )
        .fetch_all(&mut *tx)
        .await?;

        // Commit transaction
        tx.commit().await?;

        tracing::info!(
            "Batch {} created with {} bets",
            batch.batch_id,
            updated_bets.len()
        );

        metrics::counter!("batches_created_total").increment(1);
        metrics::gauge!("bets_per_batch").set(updated_bets.len() as f64);

        Ok((batch, updated_bets))
    }

    /// Phase 2: Update batch status after Solana submission
    pub async fn update_batch_submitted(
        &self,
        batch_id: Uuid,
        solana_tx_id: String,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE batches
            SET status = 'submitted', solana_tx_id = $2
            WHERE batch_id = $1
            "#,
            batch_id,
            solana_tx_id
        )
        .execute(&self.db_pool)
        .await?;

        // Update all bets in batch
        sqlx::query!(
            r#"
            UPDATE bets
            SET status = 'submitted_to_solana', solana_tx_id = $2
            WHERE external_batch_id = $1
            "#,
            batch_id,
            solana_tx_id
        )
        .execute(&self.db_pool)
        .await?;

        tracing::info!("Batch {} submitted to Solana: {}", batch_id, solana_tx_id);

        Ok(())
    }

    /// Phase 3: Update batch and bets after confirmation
    pub async fn update_batch_confirmed(
        &self,
        batch_id: Uuid,
        bet_results: Vec<(Uuid, bool, i64)>, // (bet_id, won, payout)
    ) -> Result<()> {
        let mut tx = self.db_pool.begin().await?;

        // Update batch status
        sqlx::query!(
            r#"
            UPDATE batches
            SET status = 'confirmed'
            WHERE batch_id = $1
            "#,
            batch_id
        )
        .execute(&mut *tx)
        .await?;

        // Update each bet with result
        for (bet_id, won, payout) in bet_results {
            sqlx::query!(
                r#"
                UPDATE bets
                SET 
                    status = 'completed',
                    won = $2,
                    payout_amount = $3
                WHERE bet_id = $1
                "#,
                bet_id,
                won,
                payout as i64
            )
            .execute(&mut *tx)
            .await?;

            // Log to audit trail
            sqlx::query!(
                r#"
                INSERT INTO audit_log (
                    event_type, aggregate_id, metadata, actor
                )
                VALUES ('BET_COMPLETED', $1, $2, 'PROCESSOR')
                "#,
                bet_id.to_string(),
                serde_json::json!({
                    "won": won,
                    "payout": payout,
                    "batch_id": batch_id
                })
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        tracing::info!("Batch {} confirmed and completed", batch_id);
        metrics::counter!("batches_completed_total").increment(1);

        Ok(())
    }

    /// Handle batch failure
    pub async fn update_batch_failed(
        &self,
        batch_id: Uuid,
        error_message: String,
    ) -> Result<()> {
        let mut tx = self.db_pool.begin().await?;

        // Update batch
        sqlx::query!(
            r#"
            UPDATE batches
            SET 
                status = 'failed',
                retry_count = retry_count + 1,
                last_error_message = $2
            WHERE batch_id = $1
            "#,
            batch_id,
            error_message
        )
        .execute(&mut *tx)
        .await?;

        // Revert bets to pending or mark as failed
        sqlx::query!(
            r#"
            UPDATE bets
            SET 
                status = CASE 
                    WHEN retry_count < 5 THEN 'failed_retryable'::bet_status
                    ELSE 'failed_manual_review'::bet_status
                END,
                retry_count = retry_count + 1,
                last_error_message = $2,
                external_batch_id = NULL
            WHERE external_batch_id = $1
            "#,
            batch_id,
            error_message
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        tracing::warn!("Batch {} failed: {}", batch_id, error_message);
        metrics::counter!("batches_failed_total").increment(1);

        Ok(())
    }

    /// Fetch pending bets for batching
    pub async fn fetch_pending_bets(&self, limit: i64) -> Result<Vec<Bet>> {
        let bets = sqlx::query_as!(
            Bet,
            r#"
            SELECT
                bet_id, created_at, user_wallet, vault_address, casino_id,
                game_type, stake_amount, stake_token, choice,
                status as "status: BetStatus",
                external_batch_id, solana_tx_id, retry_count, processor_id,
                last_error_code, last_error_message, payout_amount, won
            FROM bets
            WHERE status = 'pending' OR status = 'failed_retryable'
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
            limit
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(bets)
    }
}
