use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub api_port: u16,
    pub metrics_port: u16,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub solana: SolanaConfig,
    pub betting: BettingConfig,
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
    pub network: String,
    pub rpc_url: String,
    pub commitment: String,
    pub vault_program_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BettingConfig {
    pub min_bet_lamports: u64,
    pub max_bet_lamports: u64,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Config {
            api_port: env::var("API_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()?,
            metrics_port: env::var("METRICS_PORT")
                .unwrap_or_else(|_| "9090".to_string())
                .parse()?,
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set"),
                pool_size: env::var("DATABASE_POOL_SIZE")
                    .unwrap_or_else(|_| "20".to_string())
                    .parse()?,
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            solana: SolanaConfig {
                network: env::var("SOLANA_NETWORK")
                    .unwrap_or_else(|_| "devnet".to_string()),
                rpc_url: env::var("SOLANA_RPC_URL")
                    .expect("SOLANA_RPC_URL must be set"),
                commitment: env::var("SOLANA_COMMITMENT")
                    .unwrap_or_else(|_| "confirmed".to_string()),
                vault_program_id: env::var("VAULT_PROGRAM_ID")
                    .expect("VAULT_PROGRAM_ID must be set"),
            },
            betting: BettingConfig {
                min_bet_lamports: env::var("MIN_BET_LAMPORTS")
                    .unwrap_or_else(|_| "100000000".to_string())
                    .parse()?,
                max_bet_lamports: env::var("MAX_BET_LAMPORTS")
                    .unwrap_or_else(|_| "1000000000000".to_string())
                    .parse()?,
            },
        })
    }
}
