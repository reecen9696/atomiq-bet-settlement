//! Simulation logic for testing without real Solana transactions
//!
//! Provides coinflip simulation for development and testing.

use anyhow::Result;
use uuid::Uuid;

use crate::domain::Bet;

/// Simulate bet execution without real Solana transactions
pub async fn simulate_bets(bets: &[Bet]) -> Result<(String, Vec<(Uuid, bool, i64)>)> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let mut results = Vec::new();
    
    for bet in bets {
        // Simulate coinflip outcome
        let won = rng.gen_bool(0.5);
        let payout = if won {
            bet.stake_amount * 2 // 2x payout for winning
        } else {
            0
        };
        
        results.push((bet.bet_id, won, payout));
        
        tracing::trace!(
            bet_id = %bet.bet_id,
            choice = %bet.choice,
            won,
            payout,
            "Bet simulated"
        );
    }

    // Simulate Solana transaction submission
    let signature = format!("SIM_{}", Uuid::new_v4());

    tracing::debug!(
        signature = %signature,
        bet_count = bets.len(),
        "Simulated Solana transaction"
    );
    
    Ok((signature, results))
}