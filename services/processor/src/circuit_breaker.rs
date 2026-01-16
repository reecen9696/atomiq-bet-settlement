use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CircuitBreaker {
    failure_count: Arc<AtomicU64>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: u64,
    reset_timeout: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, reset_timeout_seconds: u64) -> Self {
        Self {
            failure_count: Arc::new(AtomicU64::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_threshold,
            reset_timeout: Duration::from_secs(reset_timeout_seconds),
        }
    }

    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // Check if circuit is open
        {
            let state = self.state.read().await;
            if *state == CircuitState::Open {
                // Check if we should try half-open
                let last_failure = self.last_failure_time.read().await;
                if let Some(last_time) = *last_failure {
                    if last_time.elapsed() > self.reset_timeout {
                        drop(state);
                        drop(last_failure);
                        let mut state = self.state.write().await;
                        *state = CircuitState::HalfOpen;
                        tracing::info!("Circuit breaker transitioning to HalfOpen");
                    } else {
                        return Err(CircuitBreakerError::Open);
                    }
                } else {
                    return Err(CircuitBreakerError::Open);
                }
            }
        }

        // Execute operation
        match operation() {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }

    async fn on_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        let mut state = self.state.write().await;
        if *state == CircuitState::HalfOpen {
            *state = CircuitState::Closed;
            tracing::info!("Circuit breaker closed after successful operation");
        }
    }

    async fn on_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        let mut last_failure = self.last_failure_time.write().await;
        *last_failure = Some(Instant::now());

        if failures >= self.failure_threshold {
            let mut state = self.state.write().await;
            *state = CircuitState::Open;
            tracing::warn!("Circuit breaker opened after {} failures", failures);
        }
    }

    pub async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        *state == CircuitState::Open
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    Open,
    OperationFailed(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::Open => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::OperationFailed(e) => write!(f, "Operation failed: {}", e),
        }
    }
}

impl<E: std::error::Error> std::error::Error for CircuitBreakerError<E> {}
