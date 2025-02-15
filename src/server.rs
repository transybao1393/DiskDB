use tokio::{net::TcpListener, io::{AsyncBufReadExt, AsyncWriteExt, BufReader}};
use tokio::sync::RwLock;
use std::sync::Arc;
use log::{error, info};
use crate::db::DiskDB;

/// Starts the database server and listens for incoming TCP connections.
pub async fn start_server(db: DiskDB) {
    // Bind the server to a specific address and port (127.0.0.1:6380).
    let listener = TcpListener::bind("127.0.0.1:6380").await.unwrap();
    info!("Server running on 127.0.0.1:6380");

    // Infinite loop to continuously accept incoming client connections.
    loop {
        // Accept an incoming connection from a client.
        let (socket, _) = listener.accept().await.unwrap();

        // Clone the database instance so that each connection gets a reference.
        let db = db.clone();

        // Spawn a new asynchronous task to handle the connection.
        tokio::spawn(async move {
            handle_connection(socket, db).await;
        });
    }
}

/// Handles an individual client connection.
async fn handle_connection(socket: tokio::net::TcpStream, db: DiskDB) {
    // Split the TCP stream into a reader and writer.
    let (reader, mut writer) = socket.into_split();
    
    // Wrap the reader in a buffered reader for efficient line-by-line reading.
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read data from the client line by line.
    while let Ok(bytes_read) = reader.read_line(&mut line).await {
        // If no bytes are read, the client has disconnected.
        if bytes_read == 0 {
            break;
        }

        // TODO: Process and execute database commands here.
        
        // Send a simple acknowledgment response to the client.
        writer.write_all(b"OK\n").await.unwrap();

        // Clear the line buffer for the next read operation.
        line.clear();
    }
}