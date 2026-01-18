//! Worker pool for batch processing
//!
//! This module provides a worker pool architecture for processing bets in parallel.
//! Each worker runs independently and processes batches of bets from the backend.

mod pool;
mod worker;
mod batch_processor;
mod backend_client;
mod simulation;

// Re-export the main interface
pub use pool::WorkerPool;

// Re-export components that might be useful for testing
pub use worker::Worker;
pub use batch_processor::BatchProcessor;
pub use backend_client::BackendClient;
pub use simulation::simulate_bets;