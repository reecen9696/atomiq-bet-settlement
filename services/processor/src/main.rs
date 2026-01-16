use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use solana_sdk::signature::Signer;

mod config;
mod circuit_breaker;
mod domain;
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

    tracing::info!("External Processor running");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutdown signal received");

    // Graceful shutdown
    worker_pool.stop().await;
    worker_handle.abort();
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
