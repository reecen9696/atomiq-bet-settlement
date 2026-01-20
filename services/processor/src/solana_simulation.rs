//! Coinflip simulation logic

/// Simulate coinflip outcome
/// 
/// Returns true for heads, false for tails with 50% probability
pub fn simulate_coinflip() -> bool {
    use rand::Rng;
    rand::thread_rng().gen_bool(0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_coinflip_distribution() {
        // Test that over many trials, the distribution is roughly 50/50
        let trials = 1000;
        let mut heads_count = 0;
        
        for _ in 0..trials {
            if simulate_coinflip() {
                heads_count += 1;
            }
        }
        
        let heads_ratio = heads_count as f64 / trials as f64;
        
        // Allow for some variance, but should be roughly 50%
        assert!(heads_ratio > 0.3 && heads_ratio < 0.7, 
               "Heads ratio {} is outside expected range", heads_ratio);
    }
}