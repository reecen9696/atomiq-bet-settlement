//! Retry logic and exponential backoff calculation
//!
//! Configures retry attempts and computes backoff delays for failed bets.

use std::env;

/// Get maximum retry count from environment or default to 5
pub fn max_retry_count() -> i32 {
    env::var("BET_MAX_RETRIES")
        .ok()
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(5)
}

/// Get base backoff delay in milliseconds (default: 2000ms)
pub fn retry_backoff_base_ms() -> i64 {
    env::var("BET_RETRY_BACKOFF_BASE_MS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(2_000)
}

/// Get maximum backoff delay in milliseconds (default: 60000ms)
pub fn retry_backoff_max_ms() -> i64 {
    env::var("BET_RETRY_BACKOFF_MAX_MS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(60_000)
}

/// Compute exponential backoff delay for a given retry attempt
///
/// Uses formula: base * 2^(n-1), capped at max
///
/// # Arguments
/// * `retry_count_after_increment` - The retry count after incrementing (1-indexed)
///
/// # Returns
/// Backoff delay in milliseconds
pub fn compute_backoff_ms(retry_count_after_increment: i32) -> i64 {
    // Exponential backoff: base * 2^(n-1), capped.
    let n = retry_count_after_increment.max(1) as u32;
    let base = retry_backoff_base_ms();
    let max = retry_backoff_max_ms();

    let factor = 2_i64.saturating_pow(n.saturating_sub(1));
    (base.saturating_mul(factor)).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_backoff_progression() {
        // Assuming defaults: base=2000, max=60000
        assert_eq!(compute_backoff_ms(1), 2_000);   // 2000 * 2^0 = 2000
        assert_eq!(compute_backoff_ms(2), 4_000);   // 2000 * 2^1 = 4000
        assert_eq!(compute_backoff_ms(3), 8_000);   // 2000 * 2^2 = 8000
        assert_eq!(compute_backoff_ms(4), 16_000);  // 2000 * 2^3 = 16000
        assert_eq!(compute_backoff_ms(5), 32_000);  // 2000 * 2^4 = 32000
        assert_eq!(compute_backoff_ms(6), 60_000);  // 2000 * 2^5 = 64000, capped to 60000
        assert_eq!(compute_backoff_ms(7), 60_000);  // Stays capped
    }

    #[test]
    fn test_backoff_with_zero_or_negative() {
        // Should handle edge cases gracefully
        assert_eq!(compute_backoff_ms(0), 2_000);
        assert_eq!(compute_backoff_ms(-1), 2_000);
    }
}
