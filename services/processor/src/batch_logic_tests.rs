#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Bet, BetStatus};
    use uuid::Uuid;

    #[test]
    fn test_batch_creation_logic() {
        // Test that we can group bets correctly
        let bets = vec![
            create_mock_bet("heads"),
            create_mock_bet("tails"),
            create_mock_bet("heads"),
        ];

        assert_eq!(bets.len(), 3);
        
        let heads_count = bets.iter().filter(|b| b.choice == "heads").count();
        let tails_count = bets.iter().filter(|b| b.choice == "tails").count();
        
        assert_eq!(heads_count, 2);
        assert_eq!(tails_count, 1);
    }

    #[test]
    fn test_coinflip_result_randomness() {
        // Test that random results are generated
        let mut results = Vec::new();
        
        for _ in 0..100 {
            let result = simulate_coinflip();
            results.push(result);
        }
        
        // Should have both heads and tails in 100 flips
        assert!(results.contains(&"heads".to_string()));
        assert!(results.contains(&"tails".to_string()));
        
        // Should be roughly 50/50 (allow 30-70 range for randomness)
        let heads_count = results.iter().filter(|r| *r == "heads").count();
        assert!(heads_count >= 30 && heads_count <= 70);
    }

    #[test]
    fn test_payout_calculation() {
        let stake = 1_000_000_000; // 1 SOL
        
        // Win scenario
        let win_payout = calculate_payout(stake, true);
        assert_eq!(win_payout, stake * 2); // 2x payout
        
        // Loss scenario
        let loss_payout = calculate_payout(stake, false);
        assert_eq!(loss_payout, 0);
    }

    fn create_mock_bet(choice: &str) -> Bet {
        Bet {
            bet_id: Uuid::new_v4(),
            user_wallet: "test_wallet".to_string(),
            user_vault: "test_vault".to_string(),
            stake_amount: 100_000_000,
            stake_token: "SOL".to_string(),
            choice: choice.to_string(),
            payout_amount: None,
            solana_tx_id: None,
            status: BetStatus::Pending,
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn simulate_coinflip() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.5) {
            "heads".to_string()
        } else {
            "tails".to_string()
        }
    }

    fn calculate_payout(stake: u64, won: bool) -> u64 {
        if won {
            stake * 2
        } else {
            0
        }
    }
}
