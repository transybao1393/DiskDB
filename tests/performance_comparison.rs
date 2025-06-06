#[cfg(test)]
mod performance_comparison {
    use diskdb::{Config, Server, OptimizedServer, OptimizedClient};
    use diskdb::protocol::{Request, Response};
    use diskdb::storage::rocksdb_storage::RocksDBStorage;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;
    use tokio::runtime::Runtime;
    use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
    use std::collections::HashMap;

    const TEST_ITERATIONS: usize = 10_000;
    const CONCURRENT_CLIENTS: usize = 10;
    const PIPELINE_SIZE: usize = 50;

    struct TestResults {
        single_operation_time: Duration,
        bulk_operation_time: Duration,
        concurrent_operation_time: Duration,
        pipeline_operation_time: Option<Duration>,
        memory_stats: Option<MemoryStats>,
    }

    #[derive(Debug)]
    struct MemoryStats {
        allocations: u64,
        pool_hits: u64,
        hit_rate: f64,
    }

    async fn setup_server(config: Config, optimized: bool) -> (TempDir, u16) {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
        
        let port = config.server_port;
        
        tokio::spawn(async move {
            if optimized {
                let server = OptimizedServer::new(config, storage).unwrap();
                let _ = server.start().await;
            } else {
                let server = Server::new(config, storage).unwrap();
                let _ = server.start().await;
            }
        });
        
        // Wait for server to start
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        (temp_dir, port)
    }

    async fn test_single_operations(addr: &str, iterations: usize) -> Duration {
        let start = Instant::now();
        
        for i in 0..iterations {
            let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
            
            // SET operation
            let cmd = format!("SET key{} value{}\n", i, i);
            stream.write_all(cmd.as_bytes()).await.unwrap();
            
            let mut reader = BufReader::new(&mut stream);
            let mut response = String::new();
            reader.read_line(&mut response).await.unwrap();
            
            // GET operation
            let cmd = format!("GET key{}\n", i);
            stream.write_all(cmd.as_bytes()).await.unwrap();
            
            response.clear();
            reader.read_line(&mut response).await.unwrap();
        }
        
        start.elapsed()
    }

    async fn test_bulk_operations(addr: &str, iterations: usize) -> Duration {
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let start = Instant::now();
        
        // Bulk SET operations
        for i in 0..iterations {
            let cmd = format!("SET bulk_key{} bulk_value{}\n", i, i);
            stream.write_all(cmd.as_bytes()).await.unwrap();
        }
        
        // Read all responses
        let mut reader = BufReader::new(&mut stream);
        for _ in 0..iterations {
            let mut response = String::new();
            reader.read_line(&mut response).await.unwrap();
        }
        
        // Bulk GET operations
        for i in 0..iterations {
            let cmd = format!("GET bulk_key{}\n", i);
            stream.write_all(cmd.as_bytes()).await.unwrap();
        }
        
        // Read all responses
        for _ in 0..iterations {
            let mut response = String::new();
            reader.read_line(&mut response).await.unwrap();
        }
        
        start.elapsed()
    }

    async fn test_concurrent_operations(addr: &str, clients: usize, iterations: usize) -> Duration {
        let start = Instant::now();
        let iterations_per_client = iterations / clients;
        
        let mut handles = vec![];
        
        for client_id in 0..clients {
            let addr = addr.to_string();
            let handle = tokio::spawn(async move {
                for i in 0..iterations_per_client {
                    let mut stream = tokio::net::TcpStream::connect(&addr).await.unwrap();
                    
                    let cmd = format!("SET client{}key{} value{}\n", client_id, i, i);
                    stream.write_all(cmd.as_bytes()).await.unwrap();
                    
                    let mut reader = BufReader::new(&mut stream);
                    let mut response = String::new();
                    reader.read_line(&mut response).await.unwrap();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        start.elapsed()
    }

    async fn test_pipeline_operations(addr: &str, pipeline_size: usize, iterations: usize) -> Duration {
        let client = OptimizedClient::connect(addr).await.unwrap();
        let start = Instant::now();
        
        let batches = iterations / pipeline_size;
        
        for batch in 0..batches {
            let mut requests = Vec::new();
            
            for i in 0..pipeline_size {
                let idx = batch * pipeline_size + i;
                requests.push(Request::Set {
                    key: format!("pipeline_key{}", idx),
                    value: format!("pipeline_value{}", idx),
                });
            }
            
            client.execute_pipeline(requests).await.unwrap();
        }
        
        start.elapsed()
    }

    #[tokio::test]
    async fn compare_all_optimizations() {
        println!("\n=== DiskDB Performance Comparison ===\n");
        
        let mut results: HashMap<String, TestResults> = HashMap::new();
        
        // Test configurations
        let configurations = vec![
            ("Baseline (No optimizations)", vec![]),
            ("C Parser only", vec!["c_parser"]),
            ("Memory Pool only", vec!["memory_pool"]),
            ("Network I/O only", vec![]),  // Uses OptimizedServer
            ("All optimizations", vec!["c_parser", "memory_pool"]),
        ];
        
        let mut port = 7000;
        
        for (name, features) in configurations {
            println!("Testing configuration: {}", name);
            
            let mut config = Config::default();
            config.server_port = port;
            port += 1;
            
            // For network I/O test, we use OptimizedServer
            let optimized = name.contains("Network") || name.contains("All");
            
            let (_temp_dir, server_port) = setup_server(config, optimized).await;
            let addr = format!("127.0.0.1:{}", server_port);
            
            // Run tests
            let single_time = test_single_operations(&addr, 1000).await;
            println!("  Single operations (1000): {:?}", single_time);
            
            let bulk_time = test_bulk_operations(&addr, 5000).await;
            println!("  Bulk operations (5000): {:?}", bulk_time);
            
            let concurrent_time = test_concurrent_operations(&addr, CONCURRENT_CLIENTS, 2000).await;
            println!("  Concurrent operations (10 clients, 2000 total): {:?}", concurrent_time);
            
            let pipeline_time = if optimized {
                let time = test_pipeline_operations(&addr, PIPELINE_SIZE, 2000).await;
                println!("  Pipeline operations (50 batch, 2000 total): {:?}", time);
                Some(time)
            } else {
                None
            };
            
            // Get memory stats if available
            let memory_stats = if features.contains(&"memory_pool") {
                #[cfg(feature = "memory_pool")]
                {
                    use diskdb::ffi::memory::get_memory_stats;
                    let stats = get_memory_stats();
                    Some(MemoryStats {
                        allocations: stats.allocations,
                        pool_hits: stats.pool_hits,
                        hit_rate: if stats.allocations > 0 {
                            (stats.pool_hits as f64 / stats.allocations as f64) * 100.0
                        } else {
                            0.0
                        },
                    })
                }
                #[cfg(not(feature = "memory_pool"))]
                None
            } else {
                None
            };
            
            results.insert(name.to_string(), TestResults {
                single_operation_time: single_time,
                bulk_operation_time: bulk_time,
                concurrent_operation_time: concurrent_time,
                pipeline_operation_time: pipeline_time,
                memory_stats,
            });
            
            println!();
        }
        
        // Generate comparison report
        generate_comparison_report(&results);
    }

    fn generate_comparison_report(results: &HashMap<String, TestResults>) {
        println!("\n=== Performance Comparison Report ===\n");
        
        let baseline = results.get("Baseline (No optimizations)").unwrap();
        
        println!("Configuration                  | Single Ops | Bulk Ops | Concurrent | Pipeline | Improvement");
        println!("------------------------------|------------|----------|------------|----------|-------------");
        
        for (name, result) in results {
            let single_improvement = calculate_improvement(baseline.single_operation_time, result.single_operation_time);
            let bulk_improvement = calculate_improvement(baseline.bulk_operation_time, result.bulk_operation_time);
            let concurrent_improvement = calculate_improvement(baseline.concurrent_operation_time, result.concurrent_operation_time);
            
            print!("{:<30} | {:>10.2?} | {:>8.2?} | {:>10.2?} | ",
                name,
                result.single_operation_time.as_secs_f64(),
                result.bulk_operation_time.as_secs_f64(), 
                result.concurrent_operation_time.as_secs_f64()
            );
            
            if let Some(pipeline_time) = result.pipeline_operation_time {
                print!("{:>8.2?} | ", pipeline_time.as_secs_f64());
            } else {
                print!("{:>8} | ", "N/A");
            }
            
            if name == "Baseline (No optimizations)" {
                println!("Baseline");
            } else {
                let avg_improvement = (single_improvement + bulk_improvement + concurrent_improvement) / 3.0;
                println!("{:>+.1}%", avg_improvement);
            }
            
            if let Some(stats) = &result.memory_stats {
                println!("  Memory stats: {} allocations, {:.1}% pool hit rate", 
                    stats.allocations, stats.hit_rate);
            }
        }
    }

    fn calculate_improvement(baseline: Duration, optimized: Duration) -> f64 {
        ((baseline.as_secs_f64() - optimized.as_secs_f64()) / baseline.as_secs_f64()) * 100.0
    }

    #[tokio::test]
    async fn test_specific_optimizations() {
        println!("\n=== Specific Optimization Tests ===\n");
        
        // Test parser performance
        test_parser_performance().await;
        
        // Test memory pool efficiency
        test_memory_pool_efficiency().await;
        
        // Test network I/O improvements
        test_network_io_improvements().await;
    }

    async fn test_parser_performance() {
        println!("Parser Performance Test:");
        
        let complex_commands = vec![
            "SET key1 \"This is a very long value with spaces and special characters!@#$%^&*()\"",
            "LPUSH mylist item1 item2 item3 item4 item5 item6 item7 item8 item9 item10",
            "ZADD myzset 1.0 member1 2.0 member2 3.0 member3 4.0 member4 5.0 member5",
            "XADD mystream * field1 value1 field2 value2 field3 value3 field4 value4",
        ];
        
        // Test Rust parser
        let start = Instant::now();
        for _ in 0..10000 {
            for cmd in &complex_commands {
                let _ = Request::parse_rust(cmd).unwrap();
            }
        }
        let rust_time = start.elapsed();
        
        // Test C parser if available
        #[cfg(feature = "c_parser")]
        {
            let start = Instant::now();
            for _ in 0..10000 {
                for cmd in &complex_commands {
                    let _ = Request::parse(cmd).unwrap();
                }
            }
            let c_time = start.elapsed();
            
            println!("  Rust parser: {:?}", rust_time);
            println!("  C parser: {:?}", c_time);
            println!("  Speedup: {:.2}x", rust_time.as_secs_f64() / c_time.as_secs_f64());
        }
        
        #[cfg(not(feature = "c_parser"))]
        println!("  Rust parser: {:?} (C parser not enabled)", rust_time);
    }

    async fn test_memory_pool_efficiency() {
        println!("\nMemory Pool Efficiency Test:");
        
        #[cfg(feature = "memory_pool")]
        {
            use diskdb::ffi::memory::{init_memory_pool, reset_memory_stats, get_memory_stats};
            use diskdb::data_types_pooled::PooledStorageOps;
            
            init_memory_pool().unwrap();
            reset_memory_stats();
            
            // Test various allocation patterns
            for i in 0..1000 {
                let _ = PooledStorageOps::create_string(&format!("test_string_{}", i)).unwrap();
                let _ = PooledStorageOps::create_list(100).unwrap();
            }
            
            let stats = get_memory_stats();
            println!("  Total allocations: {}", stats.allocations);
            println!("  Pool hits: {}", stats.pool_hits);
            println!("  Pool misses: {}", stats.pool_misses);
            println!("  Hit rate: {:.2}%", (stats.pool_hits as f64 / stats.allocations as f64) * 100.0);
            println!("  Active objects: {}", stats.active_objects);
        }
        
        #[cfg(not(feature = "memory_pool"))]
        println!("  Memory pool not enabled");
    }

    async fn test_network_io_improvements() {
        println!("\nNetwork I/O Improvements Test:");
        
        // Setup servers
        let mut config1 = Config::default();
        config1.server_port = 7100;
        let (_temp1, _) = setup_server(config1.clone(), false).await;
        
        let mut config2 = Config::default();
        config2.server_port = 7101;
        let (_temp2, _) = setup_server(config2.clone(), true).await;
        
        // Test latency
        let latencies_standard = measure_latencies("127.0.0.1:7100", 100).await;
        let latencies_optimized = measure_latencies("127.0.0.1:7101", 100).await;
        
        println!("  Standard server latency (P50/P99): {:.2}ms / {:.2}ms",
            latencies_standard.0, latencies_standard.1);
        println!("  Optimized server latency (P50/P99): {:.2}ms / {:.2}ms",
            latencies_optimized.0, latencies_optimized.1);
        
        // Test throughput
        let throughput_standard = measure_throughput("127.0.0.1:7100", 1000).await;
        let throughput_optimized = measure_throughput("127.0.0.1:7101", 1000).await;
        
        println!("  Standard server throughput: {:.0} ops/sec", throughput_standard);
        println!("  Optimized server throughput: {:.0} ops/sec", throughput_optimized);
        println!("  Throughput improvement: {:.2}x", throughput_optimized / throughput_standard);
    }

    async fn measure_latencies(addr: &str, samples: usize) -> (f64, f64) {
        let mut latencies = Vec::new();
        
        for _ in 0..samples {
            let start = Instant::now();
            
            let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
            stream.write_all(b"PING\n").await.unwrap();
            
            let mut reader = BufReader::new(&mut stream);
            let mut response = String::new();
            reader.read_line(&mut response).await.unwrap();
            
            latencies.push(start.elapsed().as_secs_f64() * 1000.0); // Convert to ms
        }
        
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = latencies[latencies.len() / 2];
        let p99 = latencies[latencies.len() * 99 / 100];
        
        (p50, p99)
    }

    async fn measure_throughput(addr: &str, operations: usize) -> f64 {
        let start = Instant::now();
        
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        
        for i in 0..operations {
            let cmd = format!("SET tkey{} tvalue{}\n", i, i);
            stream.write_all(cmd.as_bytes()).await.unwrap();
        }
        
        let mut reader = BufReader::new(&mut stream);
        for _ in 0..operations {
            let mut response = String::new();
            reader.read_line(&mut response).await.unwrap();
        }
        
        let duration = start.elapsed();
        operations as f64 / duration.as_secs_f64()
    }
}