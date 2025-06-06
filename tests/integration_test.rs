use diskdb::{Config, Server};
use diskdb::storage::rocksdb_storage::RocksDBStorage;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::sleep;

#[tokio::test]
async fn test_server_basic_operations() {
    // Start server in background
    let mut config = Config::new();
    config.server_port = 16380; // Use different port for testing
    config.database_path = std::path::PathBuf::from("./test_db");
    
    let storage = Arc::new(RocksDBStorage::new(&config.database_path).unwrap());
    let server = Server::new(config, storage).unwrap();
    
    tokio::spawn(async move {
        server.start().await.unwrap();
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect to server
    let stream = TcpStream::connect("127.0.0.1:16380").await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    
    // Test SET command
    writer.write_all(b"SET test_key test_value\n").await.unwrap();
    let mut response = String::new();
    reader.read_line(&mut response).await.unwrap();
    assert_eq!(response.trim(), "OK");
    
    // Test GET command
    writer.write_all(b"GET test_key\n").await.unwrap();
    response.clear();
    reader.read_line(&mut response).await.unwrap();
    assert_eq!(response.trim(), "test_value");
    
    // Test non-existent key
    writer.write_all(b"GET non_existent\n").await.unwrap();
    response.clear();
    reader.read_line(&mut response).await.unwrap();
    assert_eq!(response.trim(), "(nil)");
    
    // Cleanup
    std::fs::remove_dir_all("./test_db").ok();
}

#[tokio::test]
async fn test_invalid_commands() {
    // Start server in background
    let mut config = Config::new();
    config.server_port = 16381;
    config.database_path = std::path::PathBuf::from("./test_db2");
    
    let storage = Arc::new(RocksDBStorage::new(&config.database_path).unwrap());
    let server = Server::new(config, storage).unwrap();
    
    tokio::spawn(async move {
        server.start().await.unwrap();
    });
    
    sleep(Duration::from_millis(100)).await;
    
    let stream = TcpStream::connect("127.0.0.1:16381").await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    
    // Test invalid command
    writer.write_all(b"INVALID_CMD\n").await.unwrap();
    let mut response = String::new();
    reader.read_line(&mut response).await.unwrap();
    assert!(response.starts_with("ERROR:"));
    
    // Test SET with missing arguments
    writer.write_all(b"SET only_key\n").await.unwrap();
    response.clear();
    reader.read_line(&mut response).await.unwrap();
    assert!(response.starts_with("ERROR:"));
    
    // Cleanup
    std::fs::remove_dir_all("./test_db2").ok();
}

#[tokio::test]
async fn test_multiple_clients() {
    // Start server in background
    let mut config = Config::new();
    config.server_port = 16382;
    config.database_path = std::path::PathBuf::from("./test_db3");
    
    let storage = Arc::new(RocksDBStorage::new(&config.database_path).unwrap());
    let server = Server::new(config, storage).unwrap();
    
    tokio::spawn(async move {
        server.start().await.unwrap();
    });
    
    sleep(Duration::from_millis(100)).await;
    
    // Connect multiple clients
    let mut handles = vec![];
    
    for i in 0..3 {
        let handle = tokio::spawn(async move {
            let stream = TcpStream::connect("127.0.0.1:16382").await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            
            // Each client sets a unique key
            let key = format!("client{}_key", i);
            let value = format!("client{}_value", i);
            let command = format!("SET {} {}\n", key, value);
            
            writer.write_all(command.as_bytes()).await.unwrap();
            let mut response = String::new();
            reader.read_line(&mut response).await.unwrap();
            assert_eq!(response.trim(), "OK");
            
            // Verify the key was set
            let get_command = format!("GET {}\n", key);
            writer.write_all(get_command.as_bytes()).await.unwrap();
            response.clear();
            reader.read_line(&mut response).await.unwrap();
            assert_eq!(response.trim(), value);
        });
        
        handles.push(handle);
    }
    
    // Wait for all clients to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Cleanup
    std::fs::remove_dir_all("./test_db3").ok();
}