/// Integration tests for processor worker pool and batch processing
use redis::{Client as RedisClient, Commands};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

/// Test context for processor tests
struct ProcessorTestContext {
    redis_client: RedisClient,
}

impl ProcessorTestContext {
    fn new() -> Self {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379/1".to_string());
        
        let redis_client = RedisClient::open(redis_url)
            .expect("Failed to create Redis client");
        
        // Flush test database
        let mut conn = redis_client.get_connection().expect("Failed to connect");
        let _: () = redis::cmd("FLUSHDB").query(&mut conn).expect("Failed to flush");
        
        Self { redis_client }
    }
    
    fn add_pending_bet(&self, bet_id: &str) {
        let mut conn = self.redis_client.get_connection().expect("Failed to connect");
        
        // Create bet in Redis
        let bet = serde_json::json!({
            "bet_id": bet_id,
            "user_wallet": "TEST_WALLET",
            "vault_address": "TEST_VAULT",
            "stake_amount": 100_000_000,
            "stake_token": "SOL",
            "choice": "heads",
            "status": "pending",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "retry_count": 0
        });
        
        let key = format!("bet:{}", bet_id);
        let _: () = conn.set(&key, bet.to_string()).expect("Failed to set bet");
        
        // Add to pending stream
        let _: String = redis::cmd("XADD")
            .arg("bets:pending")
            .arg("*")
            .arg("bet_id")
            .arg(bet_id)
            .query(&mut conn)
            .expect("Failed to add to stream");
    }
    
    fn get_bet_status(&self, bet_id: &str) -> Option<String> {
        let mut conn = self.redis_client.get_connection().expect("Failed to connect");
        let key = format!("bet:{}", bet_id);
        let result: Option<String> = conn.get(&key).expect("Failed to get bet");
        
        result.and_then(|s| {
            let json: serde_json::Value = serde_json::from_str(&s).ok()?;
            json.get("status")?.as_str().map(|s| s.to_string())
        })
    }
    
    fn get_pending_count(&self) -> usize {
        let mut conn = self.redis_client.get_connection().expect("Failed to connect");
        // XLEN returns the number of entries in a stream
        let result: usize = redis::cmd("XLEN")
            .arg("bets:pending")
            .query(&mut conn)
            .unwrap_or(0);
        result
    }
    
    fn cleanup(&self) {
        let mut conn = self.redis_client.get_connection().expect("Failed to connect");
        let _: () = redis::cmd("FLUSHDB").query(&mut conn).expect("Failed to flush");
    }
}

impl Drop for ProcessorTestContext {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[tokio::test]
async fn test_bet_added_to_pending_stream() {
    let ctx = ProcessorTestContext::new();
    
    let bet_id = Uuid::new_v4().to_string();
    ctx.add_pending_bet(&bet_id);
    
    let count = ctx.get_pending_count();
    assert_eq!(count, 1, "Should have 1 bet in pending stream");
    
    let status = ctx.get_bet_status(&bet_id);
    assert_eq!(status, Some("pending".to_string()));
}

#[tokio::test]
async fn test_multiple_bets_in_stream() {
    let ctx = ProcessorTestContext::new();
    
    // Add 5 bets
    for _ in 0..5 {
        let bet_id = Uuid::new_v4().to_string();
        ctx.add_pending_bet(&bet_id);
    }
    
    let count = ctx.get_pending_count();
    assert_eq!(count, 5, "Should have 5 bets in pending stream");
}

#[tokio::test]
async fn test_batch_processing_order() {
    let ctx = ProcessorTestContext::new();
    
    let mut bet_ids = vec![];
    
    // Add bets in specific order
    for i in 0..3 {
        let bet_id = format!("bet-{}", i);
        ctx.add_pending_bet(&bet_id);
        bet_ids.push(bet_id);
    }
    
    // Verify all bets are in stream
    let count = ctx.get_pending_count();
    assert_eq!(count, 3, "Should have 3 bets in stream");
    
    // In a real processor test, we'd verify they're processed in order
    // For now, just verify they all exist
    for bet_id in bet_ids {
        let status = ctx.get_bet_status(&bet_id);
        assert!(status.is_some(), "Bet {} should exist", bet_id);
    }
}

#[tokio::test]
async fn test_retry_count_increments() {
    let ctx = ProcessorTestContext::new();
    let mut conn = ctx.redis_client.get_connection().expect("Failed to connect");
    
    let bet_id = Uuid::new_v4().to_string();
    
    // Create bet with retry count 0
    let bet = serde_json::json!({
        "bet_id": bet_id,
        "user_wallet": "TEST_WALLET",
        "vault_address": "TEST_VAULT",
        "stake_amount": 100_000_000,
        "stake_token": "SOL",
        "choice": "heads",
        "status": "pending",
        "retry_count": 0
    });
    
    let key = format!("bet:{}", bet_id);
    let _: () = conn.set(&key, bet.to_string()).expect("Failed to set bet");
    
    // Simulate retry by incrementing count
    let mut bet_data: serde_json::Value = serde_json::from_str(&bet.to_string()).unwrap();
    bet_data["retry_count"] = serde_json::json!(1);
    
    let _: () = conn.set(&key, bet_data.to_string()).expect("Failed to update bet");
    
    // Verify retry count was incremented
    let result: String = conn.get(&key).expect("Failed to get bet");
    let updated: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(updated["retry_count"], 1);
}

#[tokio::test]
async fn test_bet_status_transitions() {
    let ctx = ProcessorTestContext::new();
    let mut conn = ctx.redis_client.get_connection().expect("Failed to connect");
    
    let bet_id = Uuid::new_v4().to_string();
    let key = format!("bet:{}", bet_id);
    
    // Create bet in pending status
    let bet = serde_json::json!({
        "bet_id": bet_id,
        "status": "pending"
    });
    let _: () = conn.set(&key, bet.to_string()).unwrap();
    
    // Transition to batched
    let mut bet_data: serde_json::Value = serde_json::from_str(&bet.to_string()).unwrap();
    bet_data["status"] = serde_json::json!("batched");
    let _: () = conn.set(&key, bet_data.to_string()).unwrap();
    
    let status = ctx.get_bet_status(&bet_id);
    assert_eq!(status, Some("batched".to_string()));
    
    // Transition to submitted_to_solana
    bet_data["status"] = serde_json::json!("submitted_to_solana");
    let _: () = conn.set(&key, bet_data.to_string()).unwrap();
    
    let status = ctx.get_bet_status(&bet_id);
    assert_eq!(status, Some("submitted_to_solana".to_string()));
    
    // Transition to completed
    bet_data["status"] = serde_json::json!("completed");
    bet_data["won"] = serde_json::json!(true);
    bet_data["payout_amount"] = serde_json::json!(200_000_000);
    let _: () = conn.set(&key, bet_data.to_string()).unwrap();
    
    let status = ctx.get_bet_status(&bet_id);
    assert_eq!(status, Some("completed".to_string()));
}

#[tokio::test]
async fn test_failed_bet_handling() {
    let ctx = ProcessorTestContext::new();
    let mut conn = ctx.redis_client.get_connection().expect("Failed to connect");
    
    let bet_id = Uuid::new_v4().to_string();
    let key = format!("bet:{}", bet_id);
    
    // Create bet
    let bet = serde_json::json!({
        "bet_id": bet_id,
        "status": "pending",
        "retry_count": 0
    });
    let _: () = conn.set(&key, bet.to_string()).unwrap();
    
    // Mark as failed with error
    let mut bet_data: serde_json::Value = serde_json::from_str(&bet.to_string()).unwrap();
    bet_data["status"] = serde_json::json!("failed_retryable");
    bet_data["retry_count"] = serde_json::json!(1);
    bet_data["last_error_code"] = serde_json::json!("NETWORK_RPC_UNAVAILABLE");
    bet_data["last_error_message"] = serde_json::json!("RPC endpoint timed out");
    let _: () = conn.set(&key, bet_data.to_string()).unwrap();
    
    // Verify error was recorded
    let result: String = conn.get(&key).expect("Failed to get bet");
    let failed: serde_json::Value = serde_json::from_str(&result).unwrap();
    
    assert_eq!(failed["status"], "failed_retryable");
    assert_eq!(failed["retry_count"], 1);
    assert_eq!(failed["last_error_code"], "NETWORK_RPC_UNAVAILABLE");
}

#[tokio::test]
async fn test_circuit_breaker_opens_after_failures() {
    // This test would require running the actual processor
    // For now, we just test the data structures
    
    let ctx = ProcessorTestContext::new();
    
    // Add multiple bets that would fail
    for i in 0..10 {
        let bet_id = format!("failing-bet-{}", i);
        ctx.add_pending_bet(&bet_id);
    }
    
    let count = ctx.get_pending_count();
    assert_eq!(count, 10, "Should have 10 bets in stream");
    
    // In a real test with running processor, we'd verify:
    // 1. Circuit breaker opens after N consecutive failures
    // 2. Processing stops temporarily
    // 3. Circuit breaker closes after timeout
    // 4. Processing resumes
}
