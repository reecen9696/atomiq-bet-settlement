use anyhow::Result;
use sqlx::PgPool;
use chrono::{Utc, Duration};

use crate::domain::{Bet, BetStatus};
use crate::solana_client::SolanaClientPool;

/// Reconciliation job to handle stuck transactions
pub async fn reconcile_stuck_transactions(
    db_pool: &PgPool,
    solana_client: &SolanaClientPool,
    max_stuck_time_seconds: i64,
) -> Result<()> {
    let cutoff_time = Utc::now() - Duration::seconds(max_stuck_time_seconds);

    // Find bets stuck in submitted_to_solana status
    let stuck_bets = sqlx::query_as!(
        Bet,
        r#"
        SELECT
            bet_id, created_at, user_wallet, vault_address, casino_id,
            game_type, stake_amount, stake_token, choice,
            status as "status: BetStatus",
            external_batch_id, solana_tx_id, retry_count, processor_id,
            last_error_code, last_error_message, payout_amount, won
        FROM bets
        WHERE status = 'submitted_to_solana'
          AND updated_at < $1
          AND solana_tx_id IS NOT NULL
        LIMIT 100
        "#,
        cutoff_time
    )
    .fetch_all(db_pool)
    .await?;

    if stuck_bets.is_empty() {
        return Ok(());
    }

    tracing::info!("Found {} stuck transactions to reconcile", stuck_bets.len());

    for bet in stuck_bets {
        if let Some(tx_id) = bet.solana_tx_id {
            // Get a client from the pool
            let client = solana_client.get_client().await;
            
            // Query Solana for transaction status using tokio spawn_blocking
            let tx_id_clone = tx_id.clone();
            let status_result = tokio::task::spawn_blocking(move || {
                use solana_sdk::commitment_config::CommitmentConfig;
                let sig = tx_id_clone.parse().ok()?;
                // get_signature_status returns Option<Result<(), TransactionError>>
                // If Some(Ok(())) = confirmed, Some(Err(_)) = failed, None = not found
                client.get_signature_status_with_commitment(&sig, CommitmentConfig::confirmed()).ok()?
            }).await.ok().flatten();

            match status_result {
                Some(status) => {
                    // status is Result<(), TransactionError>
                    if status.is_ok() {
                        // Transaction confirmed
                        sqlx::query!(
                            r#"UPDATE bets SET status = 'confirmed_on_solana' WHERE bet_id = $1"#,
                            bet.bet_id
                        )
                        .execute(db_pool)
                        .await?;
                        tracing::info!("Reconciled bet {}: confirmed", bet.bet_id);
                        metrics::counter!("reconciliation_confirmed_total").increment(1);
                    } else {
                        // Transaction failed
                        sqlx::query!(
                            r#"UPDATE bets SET status = 'failed_retryable', last_error_message = 'TX failed' WHERE bet_id = $1"#,
                            bet.bet_id
                        )
                        .execute(db_pool)
                        .await?;
                        tracing::warn!("Bet {} failed on-chain", bet.bet_id);
                        metrics::counter!("reconciliation_failed_total").increment(1);
                    }
                }
                _ => {
                    // Transaction not found or error
                    tracing::warn!("TX {} not found for bet {}", tx_id, bet.bet_id);
                    
                    if bet.retry_count < 5 {
                        sqlx::query!(
                            r#"UPDATE bets SET status = 'failed_retryable', last_error_message = 'TX not found' WHERE bet_id = $1"#,
                            bet.bet_id
                        )
                        .execute(db_pool)
                        .await?;
                    } else {
                        sqlx::query!(
                            r#"UPDATE bets SET status = 'failed_manual_review' WHERE bet_id = $1"#,
                            bet.bet_id
                        )
                        .execute(db_pool)
                        .await?;
                    }
                    metrics::counter!("reconciliation_not_found_total").increment(1);
                }
            }
        }
    }

    Ok(())
}
