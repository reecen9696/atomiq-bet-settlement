use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use solana_sdk::signature::{Signer, Keypair};

mod config;
mod circuit_breaker;
mod domain;
mod retry_strategy;
mod solana_account_parsing;
mod solana_client;
mod solana_instructions;
mod solana_pda;
mod solana_simulation;
mod solana_tx;
mod worker_pool;
mod blockchain_client;
mod settlement_worker;

use config::Config;
use worker_pool::WorkerPool;
use blockchain_client::BlockchainClient;
use settlement_worker::SettlementWorker;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging with JSON formatting (configurable via env)
    let use_json = std::env::var("LOG_FORMAT")
        .unwrap_or_else(|_| "json".to_string())
        .eq_ignore_ascii_case("json");

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "processor=info".into());

    if use_json {
        // JSON structured logging for production
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        // Human-readable logging for development
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    tracing::info!(
        service = "processor",
        version = env!("CARGO_PKG_VERSION"),
        log_format = if use_json { "json" } else { "text" },
        "Starting processor service"
    );

    // Load configuration
    let config = Config::load()?;
    tracing::info!(
        worker_count = config.processor.worker_count,
        batch_interval_seconds = config.processor.batch_interval_seconds,
        "Configuration loaded"
    );

    // Initialize Solana client pool
    let solana_client = Arc::new(
        solana_client::SolanaClientPool::new(
            config.solana.rpc_urls.clone(),
            config.solana.commitment.clone(),
        )
        .await?,
    );
    tracing::info!(
        rpc_count = config.solana.rpc_urls.len(),
        "Solana RPC pool initialized"
    );

    // Load processor keypair
    let processor_keypair = solana_client::load_processor_keypair(&config.processor.keypair_path)?;
    let processor_keypair_arc = Arc::new(processor_keypair);
    tracing::info!(
        processor_pubkey = %processor_keypair_arc.pubkey(),
        "Processor keypair loaded"
    );

    // Initialize worker pool
    let worker_pool = Arc::new(WorkerPool::new(
        config.clone(),
        solana_client.clone(),
        Keypair::from_bytes(&processor_keypair_arc.to_bytes()).unwrap(),
    ));

    // Initialize blockchain client and settlement worker
    let blockchain_client = Arc::new(BlockchainClient::new(
        config.blockchain.api_base_url.clone(),
        config.blockchain.api_key.clone(),
    ));
    let settlement_worker = Arc::new(SettlementWorker::new(
        blockchain_client,
        solana_client.clone(),
        processor_keypair_arc,
        config.clone(),
    ));

    // Start metrics server
    let metrics_handle = tokio::spawn(start_metrics_server(config.metrics_port));

    // Start worker pool
    let worker_handle = tokio::spawn({
        let worker_pool = worker_pool.clone();
        async move {
            tracing::info!("WorkerPool starting (Redis-based bet processing)");
            worker_pool.start().await
        }
    });

    // Start settlement worker
    let settlement_handle = tokio::spawn({
        let settlement_worker = settlement_worker.clone();
        async move {
            tracing::info!("SettlementWorker starting (blockchain API settlement processing)");
            settlement_worker.run().await
        }
    });

    tracing::info!("Processor running");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutdown signal received");

    // Graceful shutdown
    worker_pool.stop().await;
    worker_handle.abort();
    settlement_handle.abort();
    metrics_handle.abort();

    tracing::info!("Processor stopped");

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
