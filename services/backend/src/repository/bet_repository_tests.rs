#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Bet, BetStatus};
    use crate::repository::{bet_repository::BetRepository, PostgresBetRepository};
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_bet() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/atomik_wallet_test".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let repo = PostgresBetRepository::new(pool.clone());

        let request = crate::domain::CreateBetRequest {
            stake_amount: 100_000_000,
            stake_token: "SOL".to_string(),
            choice: "heads".to_string(),
        };

        let bet = repo
            .create("test_wallet", "test_vault", request)
            .await
            .expect("Failed to create bet");

        assert_eq!(bet.user_wallet, "test_wallet");
        assert_eq!(bet.stake_amount, 100_000_000);
        assert_eq!(bet.status, BetStatus::Pending);

        // Cleanup
        sqlx::query!("DELETE FROM bets WHERE bet_id = $1", bet.bet_id)
            .execute(&pool)
            .await
            .expect("Failed to cleanup");
    }

    #[tokio::test]
    async fn test_find_bet_by_id() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/atomik_wallet_test".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let repo = PostgresBetRepository::new(pool.clone());

        let request = crate::domain::CreateBetRequest {
            stake_amount: 200_000_000,
            stake_token: "SOL".to_string(),
            choice: "tails".to_string(),
        };

        let created_bet = repo
            .create("test_wallet_2", "test_vault_2", request)
            .await
            .expect("Failed to create bet");

        let found_bet = repo
            .find_by_id(created_bet.bet_id)
            .await
            .expect("Failed to find bet")
            .expect("Bet not found");

        assert_eq!(found_bet.bet_id, created_bet.bet_id);
        assert_eq!(found_bet.stake_amount, 200_000_000);

        // Cleanup
        sqlx::query!("DELETE FROM bets WHERE bet_id = $1", created_bet.bet_id)
            .execute(&pool)
            .await
            .expect("Failed to cleanup");
    }

    #[tokio::test]
    async fn test_find_pending_bets() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/atomik_wallet_test".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let repo = PostgresBetRepository::new(pool.clone());

        // Create multiple bets
        let mut created_ids = Vec::new();
        for i in 0..3 {
            let request = crate::domain::CreateBetRequest {
                stake_amount: 100_000_000 * (i + 1) as u64,
                stake_token: "SOL".to_string(),
                choice: "heads".to_string(),
            };

            let bet = repo
                .create(&format!("wallet_{}", i), &format!("vault_{}", i), request)
                .await
                .expect("Failed to create bet");
            
            created_ids.push(bet.bet_id);
        }

        let pending = repo
            .find_pending(10)
            .await
            .expect("Failed to find pending bets");

        assert!(pending.len() >= 3);

        // Cleanup
        for bet_id in created_ids {
            sqlx::query!("DELETE FROM bets WHERE bet_id = $1", bet_id)
                .execute(&pool)
                .await
                .expect("Failed to cleanup");
        }
    }

    #[tokio::test]
    async fn test_update_bet_status_with_version() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/atomik_wallet_test".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let repo = PostgresBetRepository::new(pool.clone());

        let request = crate::domain::CreateBetRequest {
            stake_amount: 100_000_000,
            stake_token: "SOL".to_string(),
            choice: "heads".to_string(),
        };

        let bet = repo
            .create("test_wallet_version", "test_vault_version", request)
            .await
            .expect("Failed to create bet");

        // First update should succeed
        let updated = repo
            .update_status_with_version(bet.bet_id, 1, BetStatus::Batched)
            .await
            .expect("Failed to update status");

        assert!(updated);

        // Second update with same version should fail (optimistic locking)
        let updated_again = repo
            .update_status_with_version(bet.bet_id, 1, BetStatus::SubmittedToSolana)
            .await
            .expect("Failed to update status");

        assert!(!updated_again);

        // Cleanup
        sqlx::query!("DELETE FROM bets WHERE bet_id = $1", bet.bet_id)
            .execute(&pool)
            .await
            .expect("Failed to cleanup");
    }
}
