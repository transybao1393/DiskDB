#[cfg(test)]
mod stress_tests {
    use diskdb::protocol::Request;
    use std::sync::{Arc, Barrier, atomic::{AtomicUsize, Ordering}};
    use std::thread;
    use std::time::{Duration, Instant};
    
    #[cfg(feature = "memory_pool")]
    use diskdb::ffi::memory::{
        init_memory_pool, get_memory_stats, reset_memory_stats,
        PooledString, PooledVec, PooledBox, clear_thread_cache
    };
    
    const STRESS_THREADS: usize = 16;
    const STRESS_DURATION: Duration = Duration::from_secs(10);
    const OPERATIONS_PER_THREAD: usize = 100_000;
    
    #[test]
    fn stress_test_parser_concurrent_access() {
        let counter = Arc::new(AtomicUsize::new(0));
        let start_time = Instant::now();
        let mut handles = vec![];
        
        // Different command patterns for each thread to increase contention
        let command_patterns = vec![
            vec!["GET key{}", "SET key{} value{}", "DEL key{}"],
            vec!["LPUSH list{} item{}", "RPOP list{}", "LLEN list{}"],
            vec!["HSET hash{} field{} value{}", "HGET hash{} field{}", "HDEL hash{} field{}"],
            vec!["ZADD zset{} {} member{}", "ZREM zset{} member{}", "ZSCORE zset{} member{}"],
        ];
        
        for thread_id in 0..STRESS_THREADS {
            let counter_clone = counter.clone();
            let patterns = command_patterns[thread_id % command_patterns.len()].clone();
            
            let handle = thread::spawn(move || {
                let mut local_count = 0;
                let mut iteration = 0;
                
                while start_time.elapsed() < STRESS_DURATION {
                    for pattern in &patterns {
                        let command = pattern.replace("{}", &iteration.to_string());
                        match Request::parse(&command) {
                            Ok(_) => local_count += 1,
                            Err(e) => panic!("Parse error during stress test: {:?}", e),
                        }
                    }
                    iteration += 1;
                }
                
                counter_clone.fetch_add(local_count, Ordering::Relaxed);
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let total_operations = counter.load(Ordering::Relaxed);
        let duration = start_time.elapsed();
        let ops_per_second = total_operations as f64 / duration.as_secs_f64();
        
        println!("Parser stress test results:");
        println!("  Total operations: {}", total_operations);
        println!("  Duration: {:?}", duration);
        println!("  Operations/second: {:.0}", ops_per_second);
        println!("  Operations/thread/second: {:.0}", ops_per_second / STRESS_THREADS as f64);
        
        assert!(total_operations > 0);
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn stress_test_memory_pool_allocation_patterns() {
        init_memory_pool().unwrap();
        reset_memory_stats();
        
        let barrier = Arc::new(Barrier::new(STRESS_THREADS));
        let mut handles = vec![];
        
        for thread_id in 0..STRESS_THREADS {
            let barrier_clone = barrier.clone();
            let handle = thread::spawn(move || {
                // Wait for all threads to start
                barrier_clone.wait();
                
                // Mix of allocation patterns
                let mut allocations: Vec<Box<dyn std::any::Any + Send>> = Vec::new();
                
                for i in 0..1000 {
                    match i % 5 {
                        0 => {
                            // Small string allocations
                            let s = PooledString::from_str(&format!("small_{}", i)).unwrap();
                            allocations.push(Box::new(s));
                        }
                        1 => {
                            // Medium string allocations
                            let s = PooledString::from_str(&"x".repeat(100)).unwrap();
                            allocations.push(Box::new(s));
                        }
                        2 => {
                            // Vector allocations
                            let mut v = PooledVec::with_capacity(50).unwrap();
                            for j in 0..50 {
                                v.push(j).unwrap();
                            }
                            allocations.push(Box::new(v));
                        }
                        3 => {
                            // Box allocations
                            let b = PooledBox::new(thread_id * 1000 + i).unwrap();
                            allocations.push(Box::new(b));
                        }
                        4 => {
                            // Clear some allocations to create fragmentation
                            if allocations.len() > 10 {
                                allocations.truncate(allocations.len() / 2);
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                
                // Clear thread cache periodically
                if thread_id % 4 == 0 {
                    clear_thread_cache();
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let stats = get_memory_stats();
        println!("Memory pool stress test results:");
        println!("  Allocations: {}", stats.allocations);
        println!("  Deallocations: {}", stats.deallocations);
        println!("  Bytes allocated: {}", stats.bytes_allocated);
        println!("  Bytes freed: {}", stats.bytes_freed);
        println!("  Pool hits: {}", stats.pool_hits);
        println!("  Pool misses: {}", stats.pool_misses);
        println!("  Hit rate: {:.2}%", (stats.pool_hits as f64 / stats.allocations as f64) * 100.0);
        
        // Verify no leaks
        assert!(stats.active_objects < 100, "Possible memory leak: {} active objects", stats.active_objects);
    }
    
    #[test]
    fn stress_test_rapid_allocation_deallocation() {
        let barrier = Arc::new(Barrier::new(STRESS_THREADS));
        let mut handles = vec![];
        
        for thread_id in 0..STRESS_THREADS {
            let barrier_clone = barrier.clone();
            let handle = thread::spawn(move || {
                barrier_clone.wait();
                
                // Rapid allocation and deallocation
                for i in 0..10000 {
                    let input = format!("SET key{} value{}", thread_id, i);
                    let result = Request::parse(&input).unwrap();
                    drop(result); // Immediate deallocation
                    
                    // Parse multiple commands in quick succession
                    let commands = vec![
                        format!("GET key{}", i),
                        format!("LPUSH list{} item{}", thread_id, i),
                        format!("HSET hash{} field{} value{}", thread_id, i, i),
                    ];
                    
                    for cmd in commands {
                        let _ = Request::parse(&cmd).unwrap();
                    }
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        println!("Rapid allocation/deallocation test completed successfully");
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn stress_test_memory_pool_extreme_sizes() {
        init_memory_pool().unwrap();
        
        let mut handles = vec![];
        
        // Test allocation of various sizes
        let sizes = vec![
            1, 7, 15, 16, 17, 31, 32, 33, 63, 64, 65,
            127, 128, 129, 255, 256, 257, 511, 512, 513,
            1023, 1024, 1025, 2047, 2048, 2049,
            4095, 4096, 4097, 8191, 8192, 8193,
            16384, 32768, 65536, 131072, 262144,
        ];
        
        for (thread_id, &size) in sizes.iter().enumerate() {
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    // Allocate string of specific size
                    let s = "x".repeat(size);
                    let pooled = PooledString::from_str(&s).unwrap();
                    assert_eq!(pooled.len(), size);
                    
                    // Allocate vector of specific capacity
                    let mut vec = PooledVec::<u8>::with_capacity(size).unwrap();
                    for i in 0..size.min(1000) {
                        vec.push((i % 256) as u8).unwrap();
                    }
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let stats = get_memory_stats();
        println!("Extreme sizes test - Pool efficiency: {:.2}%", 
            (stats.pool_hits as f64 / stats.allocations as f64) * 100.0);
    }
    
    #[test] 
    fn stress_test_parser_memory_usage() {
        // Monitor memory usage during parsing
        let start_memory = get_process_memory_usage();
        
        // Parse many different commands
        for i in 0..100_000 {
            let commands = vec![
                format!("SET key{} {}", i, "x".repeat(100)),
                format!("LPUSH list{} {}", i, vec!["item"; 20].join(" ")),
                format!("HSET hash{} {}", i, (0..10).map(|j| format!("f{} v{}", j, j)).collect::<Vec<_>>().join(" ")),
                format!("ZADD zset{} {}", i, (0..10).map(|j| format!("{}.0 m{}", j, j)).collect::<Vec<_>>().join(" ")),
            ];
            
            for cmd in commands {
                let _ = Request::parse(&cmd).unwrap();
            }
        }
        
        let end_memory = get_process_memory_usage();
        let memory_increase = end_memory - start_memory;
        
        println!("Parser memory usage test:");
        println!("  Start memory: {} MB", start_memory / 1024 / 1024);
        println!("  End memory: {} MB", end_memory / 1024 / 1024);
        println!("  Memory increase: {} MB", memory_increase / 1024 / 1024);
        
        // Ensure reasonable memory usage (less than 100MB increase)
        assert!(memory_increase < 100 * 1024 * 1024, 
            "Excessive memory usage: {} MB", memory_increase / 1024 / 1024);
    }
    
    // Helper function to get process memory usage
    fn get_process_memory_usage() -> usize {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let status = fs::read_to_string("/proc/self/status").unwrap();
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        return parts[1].parse::<usize>().unwrap() * 1024;
                    }
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let output = Command::new("ps")
                .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
                .output()
                .unwrap();
            let rss_kb = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<usize>()
                .unwrap_or(0);
            return rss_kb * 1024;
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            // Fallback: return 0 if we can't determine memory usage
            0
        }
    }
}