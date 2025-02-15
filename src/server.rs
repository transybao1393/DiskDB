use tokio::{net::TcpListener, io::{AsyncBufReadExt, AsyncWriteExt, BufReader}};
use tokio::sync::RwLock;
use std::sync::Arc;
use log::{error, info};
use crate::db::DiskDB;

pub async fn start_server(db: DiskDB) {
    let listener = TcpListener::bind("127.0.0.1:6380").await.unwrap();
    info!("Server running on 127.0.0.1:6380");

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();
        tokio::spawn(async move {
            handle_connection(socket, db).await;
        });
    }
}

async fn handle_connection(socket: tokio::net::TcpStream, db: DiskDB) {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while let Ok(bytes_read) = reader.read_line(&mut line).await {
        if bytes_read == 0 {
            break;
        }

        // Handle the request here
        writer.write_all(b"OK\n").await.unwrap();
        line.clear();
    }
}