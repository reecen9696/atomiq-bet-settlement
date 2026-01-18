//! Backend API client for worker communication
//!
//! Handles HTTP requests to the backend service.

use anyhow::Result;
use reqwest::Client;
use uuid::Uuid;

use crate::domain::{PendingBetsResponse, UpdateBatchRequest};

/// Client for communicating with the backend API
pub struct BackendClient {
    http: Client,
    base_url: String,
}

impl BackendClient {
    /// Create a new backend client
    pub fn new(base_url: String) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Fetch pending bets from the backend
    pub async fn fetch_pending_bets(
        &self,
        limit: usize,
        processor_id: &str,
    ) -> Result<PendingBetsResponse> {
        let url = format!("{}/api/external/bets/pending", self.base_url);

        tracing::debug!(
            url = %url,
            limit,
            processor_id = %processor_id,
            "Fetching pending bets"
        );

        let resp: PendingBetsResponse = self
            .http
            .get(url)
            .query(&[
                ("limit", limit.to_string()),
                ("processor_id", processor_id.to_string()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(resp)
    }

    /// Post batch update to the backend
    pub async fn post_batch_update(&self, batch_id: Uuid, req: UpdateBatchRequest) -> Result<()> {
        let url = format!("{}/api/external/batches/{}", self.base_url, batch_id);
        
        self.http
            .post(url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?;
        
        Ok(())
    }
}