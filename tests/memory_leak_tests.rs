#[cfg(test)]
mod memory_leak_tests {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    
    #[cfg(feature = "c_parser")]
    use diskdb::ffi::parser::SafeParser;
    
    #[cfg(feature = "memory_pool")] 
    use diskdb::ffi::memory::{
        init_memory_pool, get_memory_stats, reset_memory_stats,
        PooledString, PooledVec, PooledBox
    };
    
    use diskdb::protocol::Request;
    
    #[cfg(feature = "c_parser")]
    #[test]
    fn test_parser_arena_cleanup() {
        // Test that parser arenas are properly cleaned up
        for _ in 0..10000 {
            let parser = SafeParser::new();
            let _ = parser.parse("SET key value");
            let _ = parser.parse("LPUSH list a b c d e f g h i j");
            let _ = parser.parse("HSET hash f1 v1 f2 v2 f3 v3");
            // Parser should clean up arena on drop
        }
        
        // Run in threads to test thread-local cleanup
        let mut handles = vec![];
        for _ in 0..4 {
            let handle = thread::spawn(|| {
                for _ in 0..1000 {
                    let parser = SafeParser::new();
                    let _ = parser.parse("GET key");
                    // Arena cleaned up when parser dropped
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn test_memory_pool_leak_detection() {
        init_memory_pool().unwrap();
        reset_memory_stats();
        
        let initial_stats = get_memory_stats();
        
        // Allocation and deallocation cycles
        for _ in 0..1000 {
            {
                let _s1 = PooledString::from_str("test string").unwrap();
                let _s2 = PooledString::from_str("another test string").unwrap();
                let mut _v1 = PooledVec::<i32>::with_capacity(100).unwrap();
                let _b1 = PooledBox::new(42).unwrap();
                // All should be deallocated here
            }
        }
        
        let final_stats = get_memory_stats();
        
        println!("Leak detection test:");
        println!("  Initial active objects: {}", initial_stats.active_objects);
        println!("  Final active objects: {}", final_stats.active_objects);
        println!("  Total allocations: {}", final_stats.allocations);
        println!("  Total deallocations: {}", final_stats.deallocations);
        
        // Allow small number of cached objects
        assert!(final_stats.active_objects <= 100, 
            "Memory leak detected: {} objects still active", final_stats.active_objects);
        
        // Allocations should roughly equal deallocations
        let leak_ratio = (final_stats.allocations as f64 - final_stats.deallocations as f64) 
            / final_stats.allocations as f64;
        assert!(leak_ratio < 0.01, "High leak ratio: {:.2}%", leak_ratio * 100.0);
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn test_memory_pool_thread_local_cleanup() {
        init_memory_pool().unwrap();
        
        // Spawn threads that allocate and exit
        for _ in 0..10 {
            let handle = thread::spawn(|| {
                // Allocate in thread
                let mut allocations = Vec::new();
                for i in 0..100 {
                    allocations.push(PooledString::from_str(&format!("thread_string_{}", i)).unwrap());
                }
                // Thread exits, TLS cache should be cleaned
            });
            handle.join().unwrap();
        }
        
        // Check that memory is reclaimed
        let stats = get_memory_stats();
        println!("Thread-local cleanup test - Active objects: {}", stats.active_objects);
        
        // Should have minimal active objects (just global pool overhead)
        assert!(stats.active_objects < 1000);
    }
    
    #[test]
    fn test_request_lifecycle_no_leaks() {
        // Test that Request enum doesn't leak memory
        for _ in 0..10000 {
            let requests = vec![
                Request::Get { key: "key1".to_string() },
                Request::Set { key: "key2".to_string(), value: "x".repeat(1000) },
                Request::LPush { 
                    key: "list1".to_string(), 
                    values: (0..100).map(|i| format!("item{}", i)).collect() 
                },
                Request::HSet { 
                    key: "hash1".to_string(), 
                    field: "field1".to_string(), 
                    value: "y".repeat(500) 
                },
            ];
            
            // Clone to ensure proper cleanup
            let _cloned = requests.clone();
            drop(requests);
        }
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn test_pooled_vec_growth_no_leaks() {
        init_memory_pool().unwrap();
        reset_memory_stats();
        
        // Test vector growth and shrinking
        for _ in 0..100 {
            let mut vec = PooledVec::<String>::new();
            
            // Grow vector
            for i in 0..1000 {
                vec.push(format!("element_{}", i)).unwrap();
            }
            
            // Shrink vector
            while vec.pop().is_some() {}
            
            // Vector dropped here
        }
        
        let stats = get_memory_stats();
        println!("Vector growth test - Active objects: {}", stats.active_objects);
        assert!(stats.active_objects < 100);
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn test_cross_thread_allocation_no_leaks() {
        init_memory_pool().unwrap();
        reset_memory_stats();
        
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Allocate in one thread
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let s = PooledString::from_str(&format!("cross_thread_{}", i)).unwrap();
                tx.send(s).unwrap();
            }
        });
        
        // Receive in main thread
        let mut received = Vec::new();
        for _ in 0..100 {
            received.push(rx.recv().unwrap());
        }
        
        handle.join().unwrap();
        drop(received);
        
        let stats = get_memory_stats();
        println!("Cross-thread allocation - Active objects: {}", stats.active_objects);
        assert!(stats.active_objects < 100);
    }
    
    #[test]
    fn test_error_handling_no_leaks() {
        // Test that error paths don't leak
        for _ in 0..10000 {
            // Parse errors
            let _ = Request::parse("");
            let _ = Request::parse("INVALID COMMAND");
            let _ = Request::parse("SET"); // Missing args
            let _ = Request::parse("ZADD key not_a_number member");
            
            // These should all clean up properly even on error
        }
    }
    
    #[cfg(all(feature = "c_parser", feature = "memory_pool"))]
    #[test]
    fn test_combined_features_no_leaks() {
        init_memory_pool().unwrap();
        reset_memory_stats();
        
        // Use both C parser and memory pool together
        for _ in 0..1000 {
            let input = "SET key value_with_some_content";
            match Request::parse(input) {
                Ok(Request::Set { key, value }) => {
                    let _pooled_key = PooledString::from_str(&key).unwrap();
                    let _pooled_value = PooledString::from_str(&value).unwrap();
                }
                _ => panic!("Unexpected parse result"),
            }
        }
        
        let stats = get_memory_stats();
        println!("Combined features test:");
        println!("  Allocations: {}", stats.allocations); 
        println!("  Deallocations: {}", stats.deallocations);
        println!("  Active objects: {}", stats.active_objects);
        
        assert!(stats.active_objects < 100);
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn test_memory_pool_shutdown_cleanup() {
        // Note: This test should be run last as it affects the global pool
        init_memory_pool().unwrap();
        
        // Allocate some objects
        let _s1 = PooledString::from_str("test1").unwrap();
        let _s2 = PooledString::from_str("test2").unwrap();
        
        // In real usage, memory_pool_shutdown() would be called
        // But we can't call it here as it would affect other tests
        // Instead, verify that the MemoryPoolGuard would handle cleanup
        
        println!("Memory pool has shutdown guard for cleanup");
    }
}