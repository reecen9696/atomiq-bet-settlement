//! HTTP client for querying the blockchain API
//! 
//! Polls for pending settlements and updates settlement status

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn, info};

const DEFAULT_TIMEOUT_SECS: u64 = 10;
const MAX_RETRIES: u32 = 3;

#[derive(Clone)]
pub struct BlockchainClient {
    http_client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct PendingSettlementResponse {
    pub games: Vec<GameSettlementInfo>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GameSettlementInfo {
    pub transaction_id: u64,
    pub player_address: String,
    pub game_type: String,
    pub bet_amount: u64,
    pub token: String,
    pub outcome: String, // "Win" | "Loss"
    pub payout: u64,
    pub vrf_proof: String,
    pub vrf_output: String,
    pub block_height: u64,
    pub version: u64,
    pub solana_tx_id: Option<String>, // For idempotency check - if already settled
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default)]
    pub next_retry_after: Option<i64>,
    /// Solana allowance PDA for gasless transactions
    #[serde(default)]
    pub allowance_pda: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateSettlementRequest {
    pub status: String,
    pub solana_tx_id: Option<String>,
    pub error_message: Option<String>,
    pub expected_version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_retry_after: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettlementResponse {
    pub success: bool,
    pub new_version: u64,
}

impl BlockchainClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http_client,
            base_url,
            api_key,
        }
    }

    /// Fetch pending settlements from blockchain API
    pub async fn fetch_pending_settlements(&self, limit: usize) -> Result<Vec<GameSettlementInfo>> {
        let url = format!("{}/api/settlement/pending", self.base_url);
        
        for attempt in 1..=MAX_RETRIES {
            match self.fetch_pending_settlements_once(&url, limit).await {
                Ok(games) => {
                    debug!(
                        games_count = games.len(),
                        attempt,
                        "Fetched pending settlements"
                    );
                    return Ok(games);
                }
                Err(e) => {
                    if attempt == MAX_RETRIES {
                        return Err(e).context("Failed to fetch pending settlements after retries");
                    }
                    
                    let backoff_ms = 2u64.pow(attempt - 1) * 1000;
                    warn!(
                        attempt,
                        error = %e,
                        backoff_ms,
                        "Fetch failed, retrying"
                    );
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                }
            }
        }
        
        unreachable!()
    }

    async fn fetch_pending_settlements_once(&self, url: &str, limit: usize) -> Result<Vec<GameSettlementInfo>> {
        info!("Fetching pending settlements from {} with limit={}", url, limit);
        
        let response = self.http_client
            .get(url)
            .header("X-API-Key", &self.api_key)
            .query(&[("limit", limit)])
            .send()
            .await
            .context("HTTP request failed")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Blockchain API error {}: {}", status, body);
        }

        let data: PendingSettlementResponse = response
            .json()
            .await
            .context("Failed to parse response")?;

        info!("Received {} pending settlements from API", data.games.len());
        Ok(data.games)
    }

    /// Update settlement status on blockchain
    pub async fn update_settlement_status(
        &self,
        tx_id: u64,
        status: &str,
        solana_tx_id: Option<String>,
        error_message: Option<String>,
        expected_version: u64,
        retry_count: Option<u32>,
        next_retry_after: Option<i64>,
    ) -> Result<u64> {
        let url = format!("{}/api/settlement/games/{}", self.base_url, tx_id);
        
        let request = UpdateSettlementRequest {
            status: status.to_string(),
            solana_tx_id,
            error_message,
            expected_version,
            retry_count,
            next_retry_after,
        };

        for attempt in 1..=MAX_RETRIES {
            match self.update_settlement_status_once(&url, &request).await {
                Ok(new_version) => {
                    debug!(
                        tx_id,
                        status,
                        new_version,
                        attempt,
                        "Updated settlement status"
                    );
                    return Ok(new_version);
                }
                Err(e) => {
                    let error_str = e.to_string();
                    
                    // Don't retry on conflict (version mismatch) - this means another worker already processed it
                    if error_str.contains("409") || error_str.contains("Version mismatch") {
                        tracing::warn!(
                            tx_id,
                            error = %e,
                            "Settlement already updated by another worker (version conflict)"
                        );
                        return Err(e).context("Version conflict - settlement already processed by another worker");
                    }
                    
                    // Don't retry on other client errors (404, 400, etc.)
                    if let Some(status_code) = self.extract_status_code(&e) {
                        if status_code.is_client_error() {
                            return Err(e).context("Client error, not retrying");
                        }
                    }
                    
                    if attempt == MAX_RETRIES {
                        return Err(e).context("Failed to update settlement status after retries");
                    }
                    
                    let backoff_ms = 2u64.pow(attempt - 1) * 1000;
                    warn!(
                        tx_id,
                        attempt,
                        error = %e,
                        backoff_ms,
                        "Update failed, retrying"
                    );
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                }
            }
        }
        
        unreachable!()
    }

    async fn update_settlement_status_once(&self, url: &str, request: &UpdateSettlementRequest) -> Result<u64> {
        let response = self.http_client
            .post(url)
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .context("HTTP request failed")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            tracing::error!(
                url,
                status = %status,
                body = %body,
                expected_version = request.expected_version,
                "Blockchain API rejected settlement update"
            );
            anyhow::bail!("Blockchain API error {}: {}", status, body);
        }

        let data: UpdateSettlementResponse = response
            .json()
            .await
            .context("Failed to parse response")?;

        Ok(data.new_version)
    }

    fn extract_status_code(&self, error: &anyhow::Error) -> Option<StatusCode> {
        // Try to extract status code from error message
        let error_str = error.to_string();
        if error_str.contains("409") {
            Some(StatusCode::CONFLICT)
        } else if error_str.contains("400") {
            Some(StatusCode::BAD_REQUEST)
        } else if error_str.contains("404") {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = BlockchainClient::new(
            "http://localhost:8080".to_string(),
            "test_key".to_string(),
        );
        assert_eq!(client.base_url, "http://localhost:8080");
    }
}
