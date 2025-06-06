#[cfg(test)]
mod performance_tests {
    use diskdb::protocol::Request;
    use std::time::Instant;
    use std::sync::Arc;
    use std::thread;
    
    #[cfg(feature = "c_parser")]
    use diskdb::ffi::parser::SafeParser;
    
    #[cfg(feature = "memory_pool")]
    use diskdb::ffi::memory::{init_memory_pool, get_memory_stats, reset_memory_stats};
    
    const ITERATIONS: usize = 100_000;
    const THREAD_COUNT: usize = 8;
    
    #[test]
    fn test_parser_performance_single_thread() {
        let test_cases = vec![
            "GET key1",
            "SET key1 value1",
            "LPUSH list1 item1 item2 item3",
            "HSET hash1 field1 value1",
            "ZADD zset1 1.0 member1 2.0 member2",
            "XADD stream1 * field1 value1 field2 value2",
        ];
        
        // Rust parser baseline
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            for case in &test_cases {
                let _ = Request::parse_rust(case).unwrap();
            }
        }
        let rust_duration = start.elapsed();
        println!("Rust parser: {:?} for {} iterations", rust_duration, ITERATIONS);
        
        // C parser performance (if enabled)
        #[cfg(feature = "c_parser")]
        {
            let start = Instant::now();
            for _ in 0..ITERATIONS {
                for case in &test_cases {
                    let _ = Request::parse(case).unwrap();
                }
            }
            let c_duration = start.elapsed();
            println!("C parser: {:?} for {} iterations", c_duration, ITERATIONS);
            println!("Speedup: {:.2}x", rust_duration.as_secs_f64() / c_duration.as_secs_f64());
        }
    }
    
    #[test]
    fn test_parser_performance_multi_thread() {
        let test_cases = Arc::new(vec![
            "GET key1",
            "SET key1 value1", 
            "LPUSH list1 item1 item2 item3",
            "HSET hash1 field1 value1",
            "ZADD zset1 1.0 member1 2.0 member2",
        ]);
        
        // Multi-threaded Rust parser
        let start = Instant::now();
        let mut handles = vec![];
        for _ in 0..THREAD_COUNT {
            let cases = test_cases.clone();
            let handle = thread::spawn(move || {
                for _ in 0..ITERATIONS / THREAD_COUNT {
                    for case in cases.iter() {
                        let _ = Request::parse_rust(case).unwrap();
                    }
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        let rust_duration = start.elapsed();
        println!("Rust parser ({}T): {:?}", THREAD_COUNT, rust_duration);
        
        // Multi-threaded C parser (if enabled)
        #[cfg(feature = "c_parser")]
        {
            let start = Instant::now();
            let mut handles = vec![];
            for _ in 0..THREAD_COUNT {
                let cases = test_cases.clone();
                let handle = thread::spawn(move || {
                    for _ in 0..ITERATIONS / THREAD_COUNT {
                        for case in cases.iter() {
                            let _ = Request::parse(case).unwrap();
                        }
                    }
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
            let c_duration = start.elapsed();
            println!("C parser ({}T): {:?}", THREAD_COUNT, c_duration);
            println!("Speedup: {:.2}x", rust_duration.as_secs_f64() / c_duration.as_secs_f64());
        }
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn test_memory_pool_performance() {
        use diskdb::data_types_pooled::PooledStorageOps;
        use diskdb::data_types::DataType;
        
        init_memory_pool().unwrap();
        reset_memory_stats();
        
        // Test string allocations
        let start = Instant::now();
        let mut strings = Vec::new();
        for i in 0..10000 {
            strings.push(DataType::String(format!("test_string_{}", i)));
        }
        let standard_duration = start.elapsed();
        drop(strings);
        
        reset_memory_stats();
        let start = Instant::now();
        let mut pooled_strings = Vec::new();
        for i in 0..10000 {
            pooled_strings.push(PooledStorageOps::create_string(&format!("test_string_{}", i)).unwrap());
        }
        let pooled_duration = start.elapsed();
        
        let stats = get_memory_stats();
        println!("Standard allocation: {:?}", standard_duration);
        println!("Pooled allocation: {:?}", pooled_duration);
        println!("Speedup: {:.2}x", standard_duration.as_secs_f64() / pooled_duration.as_secs_f64());
        println!("Memory stats: {:?}", stats);
        
        assert!(stats.allocations > 0);
        assert!(stats.pool_hits > 0);
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn test_memory_pool_thread_contention() {
        use diskdb::data_types_pooled::PooledStorageOps;
        
        init_memory_pool().unwrap();
        reset_memory_stats();
        
        let start = Instant::now();
        let mut handles = vec![];
        
        for thread_id in 0..THREAD_COUNT {
            let handle = thread::spawn(move || {
                let mut allocations = Vec::new();
                for i in 0..1000 {
                    allocations.push(
                        PooledStorageOps::create_string(&format!("thread_{}_string_{}", thread_id, i)).unwrap()
                    );
                }
                // Force deallocation
                drop(allocations);
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let duration = start.elapsed();
        let stats = get_memory_stats();
        
        println!("Thread contention test duration: {:?}", duration);
        println!("Total allocations: {}", stats.allocations);
        println!("Total deallocations: {}", stats.deallocations);
        println!("Pool hit rate: {:.2}%", 
            (stats.pool_hits as f64 / stats.allocations as f64) * 100.0);
        
        assert_eq!(stats.allocations, stats.deallocations);
        assert_eq!(stats.active_objects, 0);
    }
    
    #[test]
    fn test_parser_latency_percentiles() {
        use std::time::Duration;
        
        let test_input = "XADD stream1 * field1 value1 field2 value2 field3 value3";
        let mut latencies = Vec::new();
        
        // Warm up
        for _ in 0..1000 {
            let _ = Request::parse(test_input);
        }
        
        // Measure individual parse latencies
        for _ in 0..10000 {
            let start = Instant::now();
            let _ = Request::parse(test_input).unwrap();
            latencies.push(start.elapsed());
        }
        
        latencies.sort();
        let p50 = latencies[latencies.len() / 2];
        let p90 = latencies[latencies.len() * 9 / 10];
        let p99 = latencies[latencies.len() * 99 / 100];
        let p999 = latencies[latencies.len() * 999 / 1000];
        
        println!("Parser latency percentiles:");
        println!("  P50:  {:?}", p50);
        println!("  P90:  {:?}", p90);
        println!("  P99:  {:?}", p99);
        println!("  P99.9: {:?}", p999);
        
        // Ensure no extreme outliers
        assert!(p999 < p50 * 10);
    }
    
    #[cfg(all(feature = "c_parser", feature = "memory_pool"))]
    #[test]
    fn test_combined_optimization_impact() {
        use diskdb::data_types::DataType;
        use diskdb::data_types_pooled::PooledStorageOps;
        
        init_memory_pool().unwrap();
        
        // Simulate real workload
        let commands = vec![
            "SET key1 value1",
            "GET key1",
            "LPUSH list1 item1 item2 item3",
            "HSET hash1 field1 value1",
            "ZADD zset1 1.0 member1",
        ];
        
        let start = Instant::now();
        for _ in 0..10000 {
            for cmd in &commands {
                let request = Request::parse(cmd).unwrap();
                match request {
                    Request::Set { key, value } => {
                        let _ = PooledStorageOps::create_string(&value);
                    }
                    Request::LPush { key, values } => {
                        let _ = PooledStorageOps::create_list(values.len());
                    }
                    _ => {}
                }
            }
        }
        let duration = start.elapsed();
        
        let stats = get_memory_stats();
        println!("Combined optimization test:");
        println!("  Duration: {:?}", duration);
        println!("  Allocations: {}", stats.allocations);
        println!("  Pool efficiency: {:.2}%", 
            (stats.pool_hits as f64 / stats.allocations as f64) * 100.0);
    }
}