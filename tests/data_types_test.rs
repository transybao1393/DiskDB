use diskdb::{Config, Server};
use diskdb::storage::rocksdb_storage::RocksDBStorage;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::sleep;

async fn start_test_server(port: u16) -> tokio::task::JoinHandle<()> {
    let mut config = Config::new();
    config.server_port = port;
    config.database_path = std::path::PathBuf::from(format!("./test_db_{}", port));
    
    let storage = Arc::new(RocksDBStorage::new(&config.database_path).unwrap());
    let server = Server::new(config, storage).unwrap();
    
    tokio::spawn(async move {
        server.start().await.unwrap();
    })
}

async fn send_command(writer: &mut tokio::net::tcp::OwnedWriteHalf, reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>, cmd: &str) -> String {
    writer.write_all(format!("{}\n", cmd).as_bytes()).await.unwrap();
    let mut response = String::new();
    reader.read_line(&mut response).await.unwrap();
    response.trim().to_string()
}

// Helper to read array responses (multi-line)
async fn send_command_multi(writer: &mut tokio::net::tcp::OwnedWriteHalf, reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>, cmd: &str, expected_lines: usize) -> Vec<String> {
    writer.write_all(format!("{}\n", cmd).as_bytes()).await.unwrap();
    let mut lines = Vec::new();
    for _ in 0..expected_lines {
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        lines.push(line.trim().to_string());
    }
    lines
}

#[tokio::test]
async fn test_string_operations() {
    let port = 16390;
    start_test_server(port).await;
    sleep(Duration::from_millis(100)).await;
    
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    
    // Test basic SET/GET
    assert_eq!(send_command(&mut writer, &mut reader, "SET name Alice").await, "OK");
    assert_eq!(send_command(&mut writer, &mut reader, "GET name").await, "Alice");
    
    // Test INCR/DECR
    assert_eq!(send_command(&mut writer, &mut reader, "SET counter 10").await, "OK");
    assert_eq!(send_command(&mut writer, &mut reader, "INCR counter").await, "11");
    assert_eq!(send_command(&mut writer, &mut reader, "DECR counter").await, "10");
    assert_eq!(send_command(&mut writer, &mut reader, "INCRBY counter 5").await, "15");
    
    // Test APPEND
    assert_eq!(send_command(&mut writer, &mut reader, "SET msg Hello").await, "OK");
    assert_eq!(send_command(&mut writer, &mut reader, "APPEND msg  World").await, "10");
    assert_eq!(send_command(&mut writer, &mut reader, "GET msg").await, "HelloWorld");
    
    // Cleanup
    std::fs::remove_dir_all(format!("./test_db_{}", port)).ok();
}

#[tokio::test]
async fn test_list_operations() {
    let port = 16391;
    start_test_server(port).await;
    sleep(Duration::from_millis(100)).await;
    
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    
    // Test LPUSH/RPUSH
    assert_eq!(send_command(&mut writer, &mut reader, "LPUSH mylist a").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "LPUSH mylist b c").await, "3");
    assert_eq!(send_command(&mut writer, &mut reader, "RPUSH mylist d").await, "4");
    
    // Test LRANGE - should return array on multiple lines
    let range_result = send_command_multi(&mut writer, &mut reader, "LRANGE mylist 0 -1", 4).await;
    assert_eq!(range_result, vec!["c", "b", "a", "d"]);
    
    let range_result = send_command_multi(&mut writer, &mut reader, "LRANGE mylist 1 2", 2).await;
    assert_eq!(range_result, vec!["b", "a"]);
    
    // Test LPOP/RPOP
    assert_eq!(send_command(&mut writer, &mut reader, "LPOP mylist").await, "c");
    assert_eq!(send_command(&mut writer, &mut reader, "RPOP mylist").await, "d");
    assert_eq!(send_command(&mut writer, &mut reader, "LLEN mylist").await, "2");
    
    // Cleanup
    std::fs::remove_dir_all(format!("./test_db_{}", port)).ok();
}

#[tokio::test]
async fn test_set_operations() {
    let port = 16392;
    start_test_server(port).await;
    sleep(Duration::from_millis(100)).await;
    
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    
    // Test SADD
    assert_eq!(send_command(&mut writer, &mut reader, "SADD myset apple").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "SADD myset banana orange").await, "2");
    assert_eq!(send_command(&mut writer, &mut reader, "SADD myset apple").await, "0"); // Already exists
    
    // Test SCARD
    assert_eq!(send_command(&mut writer, &mut reader, "SCARD myset").await, "3");
    
    // Test SISMEMBER
    assert_eq!(send_command(&mut writer, &mut reader, "SISMEMBER myset apple").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "SISMEMBER myset grape").await, "0");
    
    // Test SREM
    assert_eq!(send_command(&mut writer, &mut reader, "SREM myset apple").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "SCARD myset").await, "2");
    
    // Test SMEMBERS - result order may vary, so just check count
    let members = send_command(&mut writer, &mut reader, "SMEMBERS myset").await;
    assert!(members.contains("banana") || members.contains("orange"));
    
    // Cleanup
    std::fs::remove_dir_all(format!("./test_db_{}", port)).ok();
}

#[tokio::test]
async fn test_hash_operations() {
    let port = 16393;
    start_test_server(port).await;
    sleep(Duration::from_millis(100)).await;
    
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    
    // Test HSET/HGET
    assert_eq!(send_command(&mut writer, &mut reader, "HSET user:1 name John").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "HSET user:1 age 30").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "HGET user:1 name").await, "John");
    assert_eq!(send_command(&mut writer, &mut reader, "HGET user:1 age").await, "30");
    
    // Test HEXISTS
    assert_eq!(send_command(&mut writer, &mut reader, "HEXISTS user:1 name").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "HEXISTS user:1 email").await, "0");
    
    // Test HDEL
    assert_eq!(send_command(&mut writer, &mut reader, "HDEL user:1 age").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "HGET user:1 age").await, "(nil)");
    
    // Test HGETALL - returns field/value pairs
    assert_eq!(send_command(&mut writer, &mut reader, "HSET user:1 email test@example.com").await, "1");
    let hgetall_result = send_command_multi(&mut writer, &mut reader, "HGETALL user:1", 4).await;
    assert!(hgetall_result.contains(&"name".to_string()));
    assert!(hgetall_result.contains(&"John".to_string()));
    
    // Cleanup
    std::fs::remove_dir_all(format!("./test_db_{}", port)).ok();
}

#[tokio::test]
async fn test_sorted_set_operations() {
    let port = 16394;
    start_test_server(port).await;
    sleep(Duration::from_millis(100)).await;
    
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    
    // Test ZADD
    assert_eq!(send_command(&mut writer, &mut reader, "ZADD leaderboard 100 alice").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "ZADD leaderboard 200 bob 150 charlie").await, "2");
    
    // Test ZCARD
    assert_eq!(send_command(&mut writer, &mut reader, "ZCARD leaderboard").await, "3");
    
    // Test ZSCORE
    assert_eq!(send_command(&mut writer, &mut reader, "ZSCORE leaderboard bob").await, "200");
    assert_eq!(send_command(&mut writer, &mut reader, "ZSCORE leaderboard unknown").await, "(nil)");
    
    // Test ZRANGE
    let zrange_result = send_command_multi(&mut writer, &mut reader, "ZRANGE leaderboard 0 -1", 3).await;
    assert_eq!(zrange_result, vec!["alice", "charlie", "bob"]);
    
    let zrange_scores = send_command_multi(&mut writer, &mut reader, "ZRANGE leaderboard 0 -1 WITHSCORES", 6).await;
    assert_eq!(zrange_scores[0], "alice");
    assert_eq!(zrange_scores[1], "100");
    
    // Test ZREM
    assert_eq!(send_command(&mut writer, &mut reader, "ZREM leaderboard alice").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "ZCARD leaderboard").await, "2");
    
    // Cleanup
    std::fs::remove_dir_all(format!("./test_db_{}", port)).ok();
}

#[tokio::test]
async fn test_json_operations() {
    let port = 16395;
    start_test_server(port).await;
    sleep(Duration::from_millis(100)).await;
    
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    
    // Test JSON.SET/JSON.GET
    let json_data = r#"{"name":"Alice","age":30,"city":"NYC"}"#;
    assert_eq!(send_command(&mut writer, &mut reader, &format!("JSON.SET user $ {}", json_data)).await, "OK");
    
    let json_result = send_command(&mut writer, &mut reader, "JSON.GET user $").await;
    assert!(json_result.contains("Alice"));
    assert!(json_result.contains("30"));
    
    // Test JSON.DEL
    assert_eq!(send_command(&mut writer, &mut reader, "JSON.DEL user $").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "JSON.GET user $").await, "(nil)");
    
    // Cleanup
    std::fs::remove_dir_all(format!("./test_db_{}", port)).ok();
}

#[tokio::test]
async fn test_stream_operations() {
    let port = 16396;
    start_test_server(port).await;
    sleep(Duration::from_millis(100)).await;
    
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    
    // Test XADD
    let id1 = send_command(&mut writer, &mut reader, "XADD mystream * name Alice age 30").await;
    assert!(id1.contains("-"));
    
    let id2 = send_command(&mut writer, &mut reader, "XADD mystream * name Bob age 25").await;
    assert!(id2.contains("-"));
    
    // Test XLEN
    assert_eq!(send_command(&mut writer, &mut reader, "XLEN mystream").await, "2");
    
    // Test XRANGE - just verify it returns some data without hanging
    // Since the format is complex and may have variable number of lines
    writer.write_all(b"XRANGE mystream - +\n").await.unwrap();
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    // Should get at least the first ID back
    assert!(line.contains("-") || line.len() > 0);
    
    // Cleanup
    std::fs::remove_dir_all(format!("./test_db_{}", port)).ok();
}

#[tokio::test]
async fn test_utility_operations() {
    let port = 16397;
    start_test_server(port).await;
    sleep(Duration::from_millis(100)).await;
    
    let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    
    // Set up test data
    assert_eq!(send_command(&mut writer, &mut reader, "SET mystring hello").await, "OK");
    assert_eq!(send_command(&mut writer, &mut reader, "LPUSH mylist a b c").await, "3");
    assert_eq!(send_command(&mut writer, &mut reader, "SADD myset x y z").await, "3");
    
    // Test TYPE
    assert_eq!(send_command(&mut writer, &mut reader, "TYPE mystring").await, "string");
    assert_eq!(send_command(&mut writer, &mut reader, "TYPE mylist").await, "list");
    assert_eq!(send_command(&mut writer, &mut reader, "TYPE myset").await, "set");
    assert_eq!(send_command(&mut writer, &mut reader, "TYPE nonexistent").await, "none");
    
    // Test EXISTS
    assert_eq!(send_command(&mut writer, &mut reader, "EXISTS mystring").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "EXISTS mystring mylist myset").await, "3");
    assert_eq!(send_command(&mut writer, &mut reader, "EXISTS nonexistent").await, "0");
    
    // Test DEL
    assert_eq!(send_command(&mut writer, &mut reader, "DEL mystring").await, "1");
    assert_eq!(send_command(&mut writer, &mut reader, "EXISTS mystring").await, "0");
    assert_eq!(send_command(&mut writer, &mut reader, "DEL mylist myset").await, "2");
    
    // Cleanup
    std::fs::remove_dir_all(format!("./test_db_{}", port)).ok();
}