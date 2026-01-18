/// Common test utilities and fixtures for integration tests
use redis::{Client as RedisClient, Commands};
use serde_json::Value;

/// Test fixtures and helper functions
pub struct TestContext {
    pub base_url: String,
    pub redis_client: RedisClient,
}

impl TestContext {
    /// Create a new test context (backend must be running separately)
    pub async fn new() -> Self {
        // Use separate Redis database for tests (db 1 instead of 0)
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379/1".to_string());
        
        let redis_client = RedisClient::open(redis_url.clone())
            .expect("Failed to create Redis client");
        
        // Flush test database before each test
        let mut conn = redis_client.get_connection().expect("Failed to connect to Redis");
        let _: () = redis::cmd("FLUSHDB").query(&mut conn).expect("Failed to flush Redis");
        
        // Use running backend instance
        let base_url = std::env::var("BACKEND_URL")
            .unwrap_or_else(|_| "http://localhost:3001".to_string());
        
        Self {
            base_url,
            redis_client,
        }
    }
    
    /// Clean up test data after test
    pub fn cleanup(&self) {
        let mut conn = self.redis_client.get_connection().expect("Failed to connect to Redis");
        let _: () = redis::cmd("FLUSHDB").query(&mut conn).expect("Failed to flush Redis");
    }
    
    /// Create a test bet in Redis directly
    pub fn create_test_bet(&self, bet_id: &str, user_wallet: &str, status: &str) -> String {
        let mut conn = self.redis_client.get_connection().expect("Failed to connect to Redis");
        
        let bet_json = serde_json::json!({
            "bet_id": bet_id,
            "user_wallet": user_wallet,
            "vault_address": "TEST_VAULT",
            "stake_amount": 100000000,
            "stake_token": "SOL",
            "choice": "heads",
            "status": status,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "retry_count": 0
        });
        
        let key = format!("bet:{}", bet_id);
        let _: () = conn.set(&key, bet_json.to_string()).expect("Failed to set bet");
        
        // Add to user's bet list
        let user_key = format!("user:{}:bets", user_wallet);
        let _: () = conn.lpush(&user_key, bet_id).expect("Failed to add to user bets");
        
        bet_id.to_string()
    }
    
    /// Get bet from Redis
    pub fn get_bet(&self, bet_id: &str) -> Option<Value> {
        let mut conn = self.redis_client.get_connection().expect("Failed to connect to Redis");
        let key = format!("bet:{}", bet_id);
        let result: Option<String> = conn.get(&key).expect("Failed to get bet");
        result.and_then(|s| serde_json::from_str(&s).ok())
    }
    
    /// Count bets in pending stream
    pub fn count_pending_bets(&self) -> usize {
        let mut conn = self.redis_client.get_connection().expect("Failed to connect to Redis");
        let result: usize = redis::cmd("XLEN")
            .arg("bets:pending")
            .query(&mut conn)
            .unwrap_or(0);
        result
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Helper function to parse error response
pub fn parse_error(body: &str) -> Option<(String, String, String)> {
    let json: Value = serde_json::from_str(body).ok()?;
    let error = json.get("error")?;
    
    Some((
        error.get("code")?.as_str()?.to_string(),
        error.get("message")?.as_str()?.to_string(),
        error.get("category")?.as_str()?.to_string(),
    ))
}

/// Helper function to create test HTTP client
pub fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("Failed to create HTTP client")
}
