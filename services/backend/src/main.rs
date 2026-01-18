use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod domain;
mod errors;
mod handlers;
mod middleware;
mod repository;
mod services;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging with JSON formatting (configurable via env)
    let use_json = std::env::var("LOG_FORMAT")
        .unwrap_or_else(|_| "text".to_string())
        .eq_ignore_ascii_case("json");

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "backend=info,tower_http=info".into());

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
        service = "backend",
        version = env!("CARGO_PKG_VERSION"),
        log_format = if use_json { "json" } else { "text" },
        "Starting backend service"
    );

    // Load configuration
    let config = Config::load()?;
    tracing::info!("Configuration loaded");

    // Initialize Redis connection
    let redis_client = redis::Client::open(config.redis.url.clone())?;
    let redis_conn = redis_client
        .get_connection_manager()
        .await?;

    tracing::info!("Redis connected");

    // Initialize application state
    let app_state = AppState::new(config.clone(), redis_conn);

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(handlers::health::health_check))
        .route("/health/detailed", get(handlers::health::detailed_health))
        // Bets
        .route("/api/bets", post(handlers::bets::create_bet))
        .route("/api/bets/:bet_id", get(handlers::bets::get_bet))
        .route("/api/bets", get(handlers::bets::list_user_bets))
        // External processor endpoints
        .route("/api/external/bets/pending", get(handlers::external::get_pending_bets))
        .route("/api/external/batches/:batch_id", post(handlers::external::update_batch))
        // Metrics
        .route("/metrics", get(handlers::metrics::metrics_handler))
        // State
        .with_state(app_state)
        // Middleware
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http());

    // Start metrics server
    let metrics_handle = tokio::spawn(start_metrics_server(config.metrics_port));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.api_port));
    tracing::info!("Backend API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    metrics_handle.await??;

    Ok(())
}

async fn start_metrics_server(port: u16) -> anyhow::Result<()> {
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    let handle = builder.install_recorder()?;

    let app = Router::new().route(
        "/metrics",
        get(|| async move { handle.render() }),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Metrics server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
