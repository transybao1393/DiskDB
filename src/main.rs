mod commands;
mod config;
mod connection;
mod data_types;
mod db;
mod error;
mod protocol;
mod server;
mod storage;
mod tls;

use config::Config;
use error::Result;
use log::info;
use server::Server;
use std::sync::Arc;
use storage::rocksdb_storage::RocksDBStorage;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting DiskDB...");

    let config = Config::from_env();
    let storage = Arc::new(RocksDBStorage::new(&config.database_path)?);
    let server = Server::new(config, storage)?;
    
    server.start().await
}