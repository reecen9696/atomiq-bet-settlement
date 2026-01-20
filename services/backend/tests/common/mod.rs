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
        // Use same Redis database as backend (db 0)
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379/0".to_string());
        
        let redis_client = RedisClient::open(redis_url.clone())
            .expect("Failed to create Redis client");
        
        // Clear test data before each test (specific keys only to not interfere with backend)
        let mut conn = redis_client.get_connection().expect("Failed to connect to Redis");
        
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
        // Clean up test-specific keys only (don't flush entire DB since backend is using it)
        let _: () = redis::cmd("DEL")
            .arg("bet:*")
            .arg("bets:user:TEST_WALLET*")
            .arg("bets:pending")
            .arg("bets:claimable")
            .query(&mut conn)
            .unwrap_or_default();
    }
    
    /// Create a test bet in Redis directly using the new hash structure
    pub fn create_test_bet(&self, bet_id: &str, user_wallet: &str, status: &str) -> String {
        let mut conn = self.redis_client.get_connection().expect("Failed to connect to Redis");
        
        let key = format!("bet:{}", bet_id);
        let now_ms = chrono::Utc::now().timestamp_millis();
        
        // Use the new hash structure instead of JSON string
        let _: () = redis::cmd("HSET")
            .arg(&key)
            .arg("bet_id").arg(bet_id)
            .arg("user_wallet").arg(user_wallet)
            .arg("vault_address").arg("TEST_VAULT")
            .arg("stake_amount").arg("100000000")
            .arg("stake_token").arg("SOL")
            .arg("choice").arg("heads")
            .arg("status").arg(status)
            .arg("created_at").arg(chrono::Utc::now().to_rfc3339())
            .arg("retry_count").arg("0")
            .arg("version").arg("1")
            .arg("game_type").arg("coinflip")
            .arg("allowance_pda").arg("")
            .arg("casino_id").arg("")
            .arg("external_batch_id").arg("")
            .arg("solana_tx_id").arg("")
            .arg("processor_id").arg("")
            .arg("last_error_code").arg("")
            .arg("last_error_message").arg("")
            .arg("payout_amount").arg("")
            .arg("won").arg("")
            .query(&mut conn).expect("Failed to create bet hash");
        
        // Add to user's bet sorted set (backend expects sorted set, not list)
        let user_index_key = format!("bets:user:{}", user_wallet);
        let _: () = conn.zadd(&user_index_key, bet_id, now_ms).expect("Failed to add to user index");
        
        // Add to claimable sorted set if status is pending
        if status == "pending" {
            let _: () = conn.zadd("bets:claimable", bet_id, now_ms).expect("Failed to add to claimable index");
            
            // Also add to pending stream
            let _: () = redis::cmd("XADD")
                .arg("bets:pending")
                .arg("*")
                .arg("bet_id").arg(bet_id)
                .query(&mut conn).expect("Failed to add to pending stream");
        }
        
        bet_id.to_string()
    }
    
    /// Get bet from Redis using the new hash structure
    pub fn get_bet(&self, bet_id: &str) -> Option<Value> {
        let mut conn = self.redis_client.get_connection().expect("Failed to connect to Redis");
        let key = format!("bet:{}", bet_id);
        
        // Use HGETALL instead of GET
        let result: std::collections::HashMap<String, String> = conn.hgetall(&key).ok()?;
        
        if result.is_empty() {
            return None;
        }
        
        // Convert hash to JSON Value
        let mut bet_json = serde_json::Map::new();
        for (field, value) in result {
            match field.as_str() {
                "stake_amount" | "retry_count" | "version" => {
                    // Parse numeric fields
                    if let Ok(num) = value.parse::<i64>() {
                        bet_json.insert(field, Value::Number(num.into()));
                    } else {
                        bet_json.insert(field, Value::String(value));
                    }
                }
                _ => {
                    bet_json.insert(field, Value::String(value));
                }
            }
        }
        
        Some(Value::Object(bet_json))
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
