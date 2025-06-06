pub mod connection_pool;
pub mod optimized_client;

pub use connection_pool::{ConnectionPool, PoolStats};
pub use optimized_client::OptimizedClient;