use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use std::time::Duration;

pub struct RetryStrategy {
    max_retries: u32,
}

impl RetryStrategy {
    pub fn new(max_retries: u32) -> Self {
        Self { max_retries }
    }

    pub fn create_backoff(&self) -> ExponentialBackoff {
        ExponentialBackoffBuilder::new()
            .with_initial_interval(Duration::from_secs(1))
            .with_max_interval(Duration::from_secs(30))
            .with_multiplier(2.0)
            .with_max_elapsed_time(Some(Duration::from_secs(300))) // 5 minutes max
            .build()
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }

    pub fn is_retryable_error(&self, error: &str) -> bool {
        // Determine if error is transient and should be retried
        error.contains("timeout")
            || error.contains("connection")
            || error.contains("network")
            || error.contains("502")
            || error.contains("503")
            || error.contains("504")
            || error.contains("rate limit")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_retry() {
        let strategy = RetryStrategy::new(3);
        assert!(strategy.should_retry(0));
        assert!(strategy.should_retry(2));
        assert!(!strategy.should_retry(3));
    }

    #[test]
    fn test_is_retryable_error() {
        let strategy = RetryStrategy::new(3);
        assert!(strategy.is_retryable_error("connection timeout"));
        assert!(strategy.is_retryable_error("503 service unavailable"));
        assert!(!strategy.is_retryable_error("invalid signature"));
    }
}
