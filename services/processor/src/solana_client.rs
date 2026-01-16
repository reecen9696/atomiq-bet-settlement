use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, read_keypair_file},
};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct SolanaClientPool {
    clients: Vec<HealthCheckedClient>,
    current_index: Arc<RwLock<usize>>,
}

struct HealthCheckedClient {
    client: Arc<RpcClient>,
    url: String,
    last_health_check: Arc<RwLock<Instant>>,
    is_healthy: Arc<RwLock<bool>>,
}

impl SolanaClientPool {
    pub async fn new(rpc_urls: Vec<String>, commitment: String) -> Result<Self> {
        let commitment_config = match commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(),
        };

        let mut clients = Vec::new();
        for url in rpc_urls {
            let client = RpcClient::new_with_commitment(url.clone(), commitment_config);
            clients.push(HealthCheckedClient {
                client: Arc::new(client),
                url: url.clone(),
                last_health_check: Arc::new(RwLock::new(Instant::now())),
                is_healthy: Arc::new(RwLock::new(true)),
            });
        }

        Ok(Self {
            clients,
            current_index: Arc::new(RwLock::new(0)),
        })
    }

    pub async fn get_client(&self) -> Arc<RpcClient> {
        let mut index = self.current_index.write().await;
        let client = &self.clients[*index];
        
        // Round-robin to next client
        *index = (*index + 1) % self.clients.len();
        
        client.client.clone()
    }

    pub async fn get_healthy_client(&self) -> Option<Arc<RpcClient>> {
        for client in &self.clients {
            let is_healthy = *client.is_healthy.read().await;
            if is_healthy {
                return Some(client.client.clone());
            }
        }
        None
    }

    pub async fn mark_unhealthy(&self, client_url: &str) {
        for client in &self.clients {
            if client.url == client_url {
                let mut is_healthy = client.is_healthy.write().await;
                *is_healthy = false;
                tracing::warn!("Marked RPC {} as unhealthy", client_url);
                break;
            }
        }
    }

    pub async fn health_check_all(&self) {
        for client in &self.clients {
            let mut last_check = client.last_health_check.write().await;
            if last_check.elapsed() > Duration::from_secs(60) {
                *last_check = Instant::now();
                drop(last_check);

                // Perform health check (synchronous in solana-client)
                let client_clone = client.client.clone();
                let url_clone = client.url.clone();
                let is_healthy_clone = client.is_healthy.clone();
                
                tokio::task::spawn_blocking(move || {
                    match client_clone.get_health() {
                        Ok(_) => {
                            tracing::debug!("RPC {} is healthy", url_clone);
                            true
                        }
                        Err(e) => {
                            tracing::warn!("RPC {} health check failed: {:?}", url_clone, e);
                            false
                        }
                    }
                }).await.ok().map(|healthy| {
                    let is_healthy_clone = is_healthy_clone.clone();
                    tokio::spawn(async move {
                        let mut is_healthy = is_healthy_clone.write().await;
                        *is_healthy = healthy;
                    });
                });
            }
        }
    }
}

pub fn load_processor_keypair(path: &str) -> Result<Keypair> {
    let keypair = read_keypair_file(Path::new(path))
        .map_err(|e| anyhow::anyhow!("Failed to load processor keypair: {}", e))?;
    Ok(keypair)
}
