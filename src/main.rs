mod db;
mod server;

use log::info;

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting DiskDB...");

    let db = db::DiskDB::new("diskdb");
    server::start_server(db).await;
}