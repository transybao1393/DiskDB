use diskdb::server::Server;
use diskdb::optimized_server::OptimizedServer;
use diskdb::config::Config;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

async fn send_command(stream: &mut TcpStream, cmd: &str) -> String {
    stream.write_all(cmd.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    
    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await.unwrap();
    String::from_utf8_lossy(&buffer[..n]).to_string()
}

async fn benchmark_server(addr: &str, name: &str, num_requests: usize) -> Duration {
    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let mut stream = TcpStream::connect(addr).await.unwrap();
    
    let start = Instant::now();
    
    // Benchmark SET operations
    for i in 0..num_requests {
        send_command(&mut stream, &format!("SET key{} value{}", i, i)).await;
    }
    
    // Benchmark GET operations
    for i in 0..num_requests {
        send_command(&mut stream, &format!("GET key{}", i)).await;
    }
    
    let elapsed = start.elapsed();
    println!("{} - {} requests completed in {:?}", name, num_requests * 2, elapsed);
    println!("  Throughput: {:.0} req/sec", (num_requests * 2) as f64 / elapsed.as_secs_f64());
    
    elapsed
}

#[tokio::test]
async fn test_performance_comparison() {
    
    println!("\n=== DiskDB Performance Comparison ===\n");
    
    // Test with standard server
    {
        let config = Config::default();
        let config = Config {
            bind_address: "127.0.0.1:6380".to_string(),
            ..config
        };
        
        let server = Server::new(config.clone());
        let server_handle = tokio::spawn(async move {
            server.run().await.unwrap();
        });
        
        let _baseline_time = benchmark_server("127.0.0.1:6380", "Standard Server", 1000).await;
        
        server_handle.abort();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Test with optimized server
    {
        let config = Config::default();
        let config = Config {
            bind_address: "127.0.0.1:6381".to_string(),
            ..config
        };
        
        let server = OptimizedServer::new(config.clone());
        let server_handle = tokio::spawn(async move {
            server.run().await.unwrap();
        });
        
        let _optimized_time = benchmark_server("127.0.0.1:6381", "Optimized Server", 1000).await;
        
        server_handle.abort();
    }
}

#[tokio::test]
async fn test_network_optimizations() {
    use diskdb::network::buffer_pool::GLOBAL_BUFFER_POOL;
    use diskdb::client::connection_pool::ConnectionPool;
    use std::net::SocketAddr;
    
    println!("\n=== Network Optimization Tests ===\n");
    
    // Test buffer pool
    {
        let pool = GLOBAL_BUFFER_POOL.clone();
        let start = Instant::now();
        
        for _ in 0..10000 {
            let mut buffer = pool.get(4096).await;
            buffer.extend_from_slice(b"test data");
            // Buffer automatically returned to pool when dropped
        }
        
        let elapsed = start.elapsed();
        println!("Buffer pool (10k allocations): {:?}", elapsed);
    }
    
    // Test connection pool
    {
        let addr: SocketAddr = "127.0.0.1:6379".parse().unwrap();
        let pool = ConnectionPool::new(addr);
        
        // Pre-warm the pool
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let stats = pool.stats().await;
        println!("\nConnection pool stats:");
        println!("  Active connections: {}", stats.active_connections);
        println!("  Total capacity: {}", stats.total_capacity);
    }
}