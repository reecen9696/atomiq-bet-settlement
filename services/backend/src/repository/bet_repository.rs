use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Bet, BetStatus, CreateBetRequest};
use crate::errors::{AppError, Result};

#[async_trait]
pub trait BetRepository: Send + Sync {
    async fn create(&self, user_wallet: &str, vault_address: &str, req: CreateBetRequest) -> Result<Bet>;
    async fn find_by_id(&self, bet_id: Uuid) -> Result<Option<Bet>>;
    async fn find_by_user(&self, user_wallet: &str, limit: i64, offset: i64) -> Result<Vec<Bet>>;
    async fn find_pending(&self, limit: i64) -> Result<Vec<Bet>>;
    async fn update_status(&self, bet_id: Uuid, status: BetStatus, solana_tx_id: Option<String>) -> Result<()>;
    async fn update_status_with_version(&self, bet_id: Uuid, expected_version: i32, status: BetStatus) -> Result<bool>;
}

pub struct PostgresBetRepository {
    pool: PgPool,
}

impl PostgresBetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BetRepository for PostgresBetRepository {
    async fn create(&self, user_wallet: &str, vault_address: &str, req: CreateBetRequest) -> Result<Bet> {
        let bet_id = Uuid::new_v4();
        let now = Utc::now();

        let bet = sqlx::query_as!(
            Bet,
            r#"
            INSERT INTO bets (
                bet_id, created_at, user_wallet, vault_address, game_type,
                stake_amount, stake_token, choice, status
            )
            VALUES ($1, $2, $3, $4, 'coinflip', $5, $6, $7, 'pending')
            RETURNING
                bet_id, created_at, user_wallet, vault_address, casino_id,
                game_type, stake_amount, stake_token, choice,
                status as "status: BetStatus",
                external_batch_id, solana_tx_id, retry_count, processor_id,
                last_error_code, last_error_message, payout_amount, won
            "#,
            bet_id,
            now,
            user_wallet,
            vault_address,
            req.stake_amount as i64,
            req.stake_token,
            req.choice,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(bet)
    }

    async fn find_by_id(&self, bet_id: Uuid) -> Result<Option<Bet>> {
        let bet = sqlx::query_as!(
            Bet,
            r#"
            SELECT
                bet_id, created_at, user_wallet, vault_address, casino_id,
                game_type, stake_amount, stake_token, choice,
                status as "status: BetStatus",
                external_batch_id, solana_tx_id, retry_count, processor_id,
                last_error_code, last_error_message, payout_amount, won
            FROM bets
            WHERE bet_id = $1
            "#,
            bet_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(bet)
    }

    async fn find_by_user(&self, user_wallet: &str, limit: i64, offset: i64) -> Result<Vec<Bet>> {
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
            WHERE user_wallet = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_wallet,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(bets)
    }

    async fn find_pending(&self, limit: i64) -> Result<Vec<Bet>> {
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
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(bets)
    }

    async fn update_status(&self, bet_id: Uuid, status: BetStatus, solana_tx_id: Option<String>) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE bets
            SET status = $2, solana_tx_id = COALESCE($3, solana_tx_id)
            WHERE bet_id = $1
            "#,
            bet_id,
            status as BetStatus,
            solana_tx_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_status_with_version(&self, bet_id: Uuid, expected_version: i32, status: BetStatus) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            UPDATE bets
            SET status = $2, version = version + 1
            WHERE bet_id = $1 AND version = $3
            "#,
            bet_id,
            status as BetStatus,
            expected_version
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
#[path = "bet_repository_tests.rs"]
mod bet_repository_tests;
