use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub processor: ProcessorConfig,
    pub solana: SolanaConfig,
    pub blockchain: BlockchainConfig,
    pub metrics_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessorConfig {
    pub worker_count: usize,
    pub settlement_worker_count: usize,
    pub batch_interval_seconds: u64,
    pub batch_size: usize,
    pub max_bets_per_tx: usize,
    pub max_retries: u32,
    pub keypair_path: String,
    pub max_stuck_time_seconds: i64,
    pub coordinator_enabled: bool,
    pub coordinator_channel_buffer_size: usize,
    pub coordinator_batch_min_size: usize,
    pub coordinator_batch_max_size: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolanaConfig {
    pub rpc_urls: Vec<String>,
    pub commitment: String,
    pub vault_program_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockchainConfig {
    pub api_base_url: String,
    pub api_key: String,
    pub poll_interval_seconds: u64,
    pub settlement_batch_size: usize,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let rpc_primary = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL must be set");
        let rpc_fallback = env::var("SOLANA_RPC_FALLBACK_URL").unwrap_or_else(|_| rpc_primary.clone());
        
        Ok(Config {
            processor: ProcessorConfig {
                worker_count: env::var("PROCESSOR_WORKER_COUNT")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                settlement_worker_count: env::var("SETTLEMENT_WORKER_COUNT")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()?,
                batch_interval_seconds: env::var("PROCESSOR_BATCH_INTERVAL_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                batch_size: env::var("PROCESSOR_BATCH_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
                max_bets_per_tx: env::var("PROCESSOR_MAX_BETS_PER_TX")
                    .unwrap_or_else(|_| "12".to_string())
                    .parse()?,
                max_retries: env::var("PROCESSOR_MAX_RETRIES")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
                keypair_path: env::var("PROCESSOR_KEYPAIR")
                    .expect("PROCESSOR_KEYPAIR must be set"),
                max_stuck_time_seconds: env::var("PROCESSOR_MAX_STUCK_TIME_SECONDS")
                    .unwrap_or_else(|_| "120".to_string())
                    .parse()?,
                coordinator_enabled: env::var("COORDINATOR_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                coordinator_channel_buffer_size: env::var("COORDINATOR_CHANNEL_BUFFER_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
                coordinator_batch_min_size: env::var("COORDINATOR_BATCH_MIN_SIZE")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
                coordinator_batch_max_size: env::var("COORDINATOR_BATCH_MAX_SIZE")
                    .unwrap_or_else(|_| "12".to_string())
                    .parse()?,
            },
            solana: SolanaConfig {
                rpc_urls: vec![rpc_primary, rpc_fallback],
                commitment: env::var("SOLANA_COMMITMENT")
                    .unwrap_or_else(|_| "confirmed".to_string()),
                vault_program_id: env::var("VAULT_PROGRAM_ID")
                    .expect("VAULT_PROGRAM_ID must be set"),
            },
            blockchain: BlockchainConfig {
                api_base_url: env::var("BLOCKCHAIN_API_URL")
                    .expect("BLOCKCHAIN_API_URL must be set"),
                api_key: env::var("BLOCKCHAIN_API_KEY")
                    .expect("BLOCKCHAIN_API_KEY must be set"),
                poll_interval_seconds: env::var("BLOCKCHAIN_POLL_INTERVAL_SECONDS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                settlement_batch_size: env::var("BLOCKCHAIN_SETTLEMENT_BATCH_SIZE")
                    .unwrap_or_else(|_| "50".to_string())
                    .parse()?,
            },
            metrics_port: env::var("PROCESSOR_METRICS_PORT")
                .unwrap_or_else(|_| "9091".to_string())
                .parse()?,
        })
    }
}

