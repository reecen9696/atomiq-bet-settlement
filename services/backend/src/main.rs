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
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::load()?;
    tracing::info!("Configuration loaded");

    // Initialize database pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.pool_size)
        .connect(&config.database.url)
        .await?;

    tracing::info!("Database connected");

    // Initialize Redis connection
    let redis_client = redis::Client::open(config.redis.url.clone())?;
    let redis_conn = redis_client
        .get_connection_manager()
        .await?;

    tracing::info!("Redis connected");

    // Initialize application state
    let app_state = AppState::new(config.clone(), db_pool, redis_conn);

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
