use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error, warn};
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
mod coordinator;

use config::Config;
use worker_pool::WorkerPool;
use blockchain_client::BlockchainClient;
use settlement_worker::SettlementWorker;
use coordinator::Coordinator;

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

    // Initialize blockchain client and settlement workers
    let blockchain_client = Arc::new(BlockchainClient::new(
        config.blockchain.api_base_url.clone(),
        config.blockchain.api_key.clone(),
    ));

    info!(
        settlement_worker_count = config.processor.settlement_worker_count,
        coordinator_enabled = config.processor.coordinator_enabled,
        "Starting settlement workers"
    );

    let mut settlement_handles = Vec::new();

    if config.processor.coordinator_enabled {
        // NEW COORDINATOR MODE: Create channels and spawn coordinator
        info!("Using coordinator-worker architecture");

        let channel_buffer_size = config.processor.coordinator_channel_buffer_size;
        let mut work_senders = Vec::new();
        let mut work_receivers = Vec::new();

        // Create channels for each worker
        for _ in 0..config.processor.settlement_worker_count {
            let (tx, rx) = tokio::sync::mpsc::channel(channel_buffer_size);
            work_senders.push(tx);
            work_receivers.push(rx);
        }

        // Spawn coordinator
        let coordinator = Arc::new(Coordinator::new(
            blockchain_client.clone(),
            work_senders,
            config.clone(),
        ));

        let coordinator_handle = tokio::spawn({
            let coordinator = coordinator.clone();
            async move {
                info!("Coordinator starting");
                coordinator.run().await
            }
        });
        settlement_handles.push(coordinator_handle);

        // Spawn workers with channels
        for (worker_id, receiver) in work_receivers.into_iter().enumerate() {
            let worker_id = worker_id + 1;
            let settlement_worker = SettlementWorker::with_channel(
                blockchain_client.clone(),
                solana_client.clone(),
                processor_keypair_arc.clone(),
                config.clone(),
                worker_id,
                receiver,
            );

            let handle = tokio::spawn(async move {
                info!(worker_id, "Settlement worker started (coordinator mode)");
                settlement_worker.run().await
            });
            
            settlement_handles.push(handle);
        }

        info!(
            worker_count = config.processor.settlement_worker_count,
            "Coordinator and workers spawned"
        );
    } else {
        // LEGACY POLLING MODE: Workers poll independently
        warn!("Using legacy polling mode (not recommended - has race conditions)");

        for worker_id in 1..=config.processor.settlement_worker_count {
            let settlement_worker = SettlementWorker::new(
                blockchain_client.clone(),
                solana_client.clone(),
                processor_keypair_arc.clone(),
                config.clone(),
                worker_id,
            );

            let handle = tokio::spawn(async move {
                info!(worker_id, "Settlement worker started (legacy mode)");
                settlement_worker.run().await
            });
            
            settlement_handles.push(handle);
        }
    }

    info!("All settlement components spawned");

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

    tracing::info!("Processor running");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutdown signal received");

    // Graceful shutdown
    worker_pool.stop().await;
    worker_handle.abort();
    
    // Stop all settlement workers
    for handle in settlement_handles {
        handle.abort();
    }
    
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
