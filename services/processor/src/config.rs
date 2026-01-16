use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub processor: ProcessorConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub solana: SolanaConfig,
    pub metrics_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessorConfig {
    pub worker_count: usize,
    pub batch_interval_seconds: u64,
    pub batch_size: usize,
    pub max_retries: u32,
    pub keypair_path: String,
    pub max_stuck_time_seconds: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolanaConfig {
    pub rpc_urls: Vec<String>,
    pub commitment: String,
    pub vault_program_id: String,
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
                batch_interval_seconds: env::var("PROCESSOR_BATCH_INTERVAL_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                batch_size: env::var("PROCESSOR_BATCH_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
                max_retries: env::var("PROCESSOR_MAX_RETRIES")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
                keypair_path: env::var("PROCESSOR_KEYPAIR")
                    .expect("PROCESSOR_KEYPAIR must be set"),
                max_stuck_time_seconds: env::var("PROCESSOR_MAX_STUCK_TIME_SECONDS")
                    .unwrap_or_else(|_| "120".to_string())
                    .parse()?,
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
                pool_size: env::var("DATABASE_POOL_SIZE")
                    .unwrap_or_else(|_| "20".to_string())
                    .parse()?,
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            solana: SolanaConfig {
                rpc_urls: vec![rpc_primary, rpc_fallback],
                commitment: env::var("SOLANA_COMMITMENT")
                    .unwrap_or_else(|_| "confirmed".to_string()),
                vault_program_id: env::var("VAULT_PROGRAM_ID")
                    .expect("VAULT_PROGRAM_ID must be set"),
            },
            metrics_port: env::var("PROCESSOR_METRICS_PORT")
                .unwrap_or_else(|_| "9091".to_string())
                .parse()?,
        })
    }
}
