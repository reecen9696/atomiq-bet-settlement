use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use solana_sdk::signature::Signer;

mod config;
mod batch_processor;
mod circuit_breaker;
mod domain;
mod reconciliation;
mod retry_strategy;
mod solana_client;
mod solana_tx;
mod worker_pool;

use config::Config;
use worker_pool::WorkerPool;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "processor=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Starting External Processor service");

    // Load configuration
    let config = Config::load()?;
    tracing::info!("Configuration loaded: {} workers", config.processor.worker_count);

    // Initialize database pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.pool_size)
        .connect(&config.database.url)
        .await?;
    tracing::info!("Database connected");

    // Initialize Redis connection
    let redis_client = redis::Client::open(config.redis.url.clone())?;
    let redis_conn = redis_client.get_connection_manager().await?;
    tracing::info!("Redis connected");

    // Initialize Solana client pool
    let solana_client = Arc::new(
        solana_client::SolanaClientPool::new(
            config.solana.rpc_urls.clone(),
            config.solana.commitment.clone(),
        )
        .await?,
    );
    tracing::info!("Solana RPC pool initialized");

    // Load processor keypair
    let processor_keypair = solana_client::load_processor_keypair(&config.processor.keypair_path)?;
    tracing::info!("Processor keypair loaded: {}", processor_keypair.pubkey().to_string());

    // Initialize worker pool
    let worker_pool = Arc::new(WorkerPool::new(
        config.clone(),
        db_pool.clone(),
        redis_conn.clone(),
        solana_client.clone(),
        processor_keypair,
    ));

    // Start metrics server
    let metrics_handle = tokio::spawn(start_metrics_server(config.metrics_port));

    // Start worker pool
    let worker_handle = tokio::spawn({
        let worker_pool = worker_pool.clone();
        async move {
            worker_pool.start().await
        }
    });

    // Start reconciliation job
    let reconciliation_handle = tokio::spawn({
        let config = config.clone();
        let db_pool = db_pool.clone();
        let solana_client = solana_client.clone();
        async move {
            let mut ticker = interval(Duration::from_secs(60));
            loop {
                ticker.tick().await;
                if let Err(e) = reconciliation::reconcile_stuck_transactions(
                    &db_pool,
                    solana_client.as_ref(),
                    config.processor.max_stuck_time_seconds,
                )
                .await
                {
                    tracing::error!("Reconciliation error: {:?}", e);
                }
            }
        }
    });

    tracing::info!("External Processor running");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutdown signal received");

    // Graceful shutdown
    worker_pool.stop().await;
    worker_handle.abort();
    reconciliation_handle.abort();
    metrics_handle.abort();

    tracing::info!("External Processor stopped");

    Ok(())
}

async fn start_metrics_server(port: u16) -> Result<()> {
    use std::net::SocketAddr;
    use axum::{routing::get, Router};

    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    let handle = builder.install_recorder()?;

    let app = Router::new().route(
        "/metrics",
        get(|| async move { handle.render() }),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Processor metrics listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
