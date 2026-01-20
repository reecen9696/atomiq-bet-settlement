/// Integration tests for error handling scenarios
mod common;

use common::{parse_error, test_client, TestContext};
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_validation_error_invalid_amount() {
    let ctx = TestContext::new().await;
    let client = test_client();
    
    // Test amount above maximum (1000 SOL = 1_000_000_000_000 lamports)
    let response = client
        .post(format!("{}/api/bets", ctx.base_url))
        .json(&json!({
            "choice": "heads",
            "stake_amount": 2_000_000_000_000_u64, // 2000 SOL, above max
            "stake_token": "SOL"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = response.text().await.expect("Failed to read response");
    let (code, message, category) = parse_error(&body).expect("Failed to parse error");
    
    assert_eq!(code, "VALIDATION_INVALID_INPUT");
    assert_eq!(category, "Validation");
    assert!(message.contains("Invalid request body") || message.contains("Invalid stake amount"));
}

#[tokio::test]
async fn test_validation_error_missing_field() {
    let ctx = TestContext::new().await;
    let client = test_client();
    
    // Missing required field "stake_token"
    let response = client
        .post(format!("{}/api/bets", ctx.base_url))
        .json(&json!({
            "choice": "heads",
            "stake_amount": 100_000_000
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = response.text().await.expect("Failed to read response");
    let (code, _, category) = parse_error(&body).expect("Failed to parse error");
    
    assert!(code == "VALIDATION_MISSING_FIELD" || code == "VALIDATION_INVALID_INPUT");
    assert_eq!(category, "Validation");
}

#[tokio::test]
async fn test_not_found_error() {
    let ctx = TestContext::new().await;
    let client = test_client();
    
    // Request non-existent bet
    let fake_bet_id = Uuid::new_v4();
    let response = client
        .get(format!("{}/api/bets/{}", ctx.base_url, fake_bet_id))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body = response.text().await.expect("Failed to read response");
    let (code, message, category) = parse_error(&body).expect("Failed to parse error");
    
    assert_eq!(code, "NOT_FOUND_BET");
    assert_eq!(category, "NotFound");
    assert!(message.contains(&fake_bet_id.to_string()));
}

#[tokio::test]
async fn test_successful_bet_creation() {
    let ctx = TestContext::new().await;
    let client = test_client();
    
    let response = client
        .post(format!("{}/api/bets", ctx.base_url))
        .json(&json!({
            "choice": "heads",
            "stake_amount": 100_000_000,
            "stake_token": "SOL"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let bet = body.get("bet").expect("No bet in response");
    
    assert!(bet.get("bet_id").is_some());
    assert_eq!(bet.get("stake_amount").unwrap(), &json!(100_000_000));
    assert_eq!(bet.get("choice").unwrap(), &json!("heads"));
    assert_eq!(bet.get("status").unwrap(), &json!("pending"));
    
    // Verify bet was added to Redis stream
    let pending_count = ctx.count_pending_bets();
    assert!(pending_count > 0, "Bet should be in pending stream");
}

#[tokio::test]
async fn test_get_bet_by_id() {
    let ctx = TestContext::new().await;
    let client = test_client();
    
    // Create a test bet directly in Redis
    let bet_id = Uuid::new_v4().to_string();
    ctx.create_test_bet(&bet_id, "TEST_WALLET", "pending");
    
    let response = client
        .get(format!("{}/api/bets/{}", ctx.base_url, bet_id))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let bet: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(bet.get("bet_id").unwrap(), &json!(bet_id));
    assert_eq!(bet.get("status").unwrap(), &json!("pending"));
}

#[tokio::test]
async fn test_list_user_bets() {
    let ctx = TestContext::new().await;
    let client = test_client();
    
    let user_wallet = "TEST_WALLET_123";
    
    // Create multiple test bets
    for i in 0..3 {
        let bet_id = Uuid::new_v4().to_string();
        ctx.create_test_bet(&bet_id, user_wallet, "pending");
    }
    
    let response = client
        .get(format!("{}/api/bets?user_wallet={}", ctx.base_url, user_wallet))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let bets: Vec<serde_json::Value> = response.json().await.expect("Failed to parse JSON");
    assert_eq!(bets.len(), 3, "Should return 3 bets");
}

#[tokio::test]
async fn test_list_user_bets_with_limit() {
    let ctx = TestContext::new().await;
    let client = test_client();
    
    let user_wallet = "TEST_WALLET_LIMIT";
    
    // Create 10 test bets
    for _ in 0..10 {
        let bet_id = Uuid::new_v4().to_string();
        ctx.create_test_bet(&bet_id, user_wallet, "pending");
    }
    
    let response = client
        .get(format!("{}/api/bets?user_wallet={}&limit=5", ctx.base_url, user_wallet))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let bets: Vec<serde_json::Value> = response.json().await.expect("Failed to parse JSON");
    assert_eq!(bets.len(), 5, "Should return only 5 bets due to limit");
}

#[tokio::test]
async fn test_health_endpoint() {
    let ctx = TestContext::new().await;
    let client = test_client();
    
    let response = client
        .get(format!("{}/health", ctx.base_url))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body.get("status").unwrap(), &json!("healthy"));
    assert!(body.get("timestamp").is_some());
}

#[tokio::test]
async fn test_concurrent_bet_creation() {
    let ctx = TestContext::new().await;
    let client = test_client();
    
    // Create 10 bets concurrently
    let mut handles = vec![];
    
    for i in 0..10 {
        let url = format!("{}/api/bets", ctx.base_url);
        let client = client.clone();
        
        let handle = tokio::spawn(async move {
            client
                .post(&url)
                .json(&json!({
                    "choice": if i % 2 == 0 { "heads" } else { "tails" },
                    "stake_amount": 100_000_000,
                    "stake_token": "SOL"
                }))
                .send()
                .await
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut success_count = 0;
    for handle in handles {
        let response = handle.await.expect("Task panicked").expect("Request failed");
        if response.status() == StatusCode::OK {
            success_count += 1;
        }
    }
    
    assert_eq!(success_count, 10, "All concurrent requests should succeed");
}
