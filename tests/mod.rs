// Comprehensive test suite for DiskDB performance optimizations

mod performance_tests;
mod security_tests;
mod stress_tests;
mod memory_leak_tests;

// Integration tests that use all features together
#[cfg(test)]
mod integration_tests {
    use diskdb::protocol::{Request, Response};
    use diskdb::storage::rocksdb_storage::RocksDBStorage;
    use diskdb::storage::Storage;
    use diskdb::commands::CommandExecutor;
    use std::sync::Arc;
    use tempfile::TempDir;
    
    #[cfg(feature = "memory_pool")]
    use diskdb::ffi::memory::{init_memory_pool, get_memory_stats};
    
    #[tokio::test]
    async fn test_full_stack_with_optimizations() {
        #[cfg(feature = "memory_pool")]
        init_memory_pool().unwrap();
        
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
        let executor = CommandExecutor::new(storage.clone());
        
        // Test various commands with parser and memory pool
        let test_commands = vec![
            ("SET key1 value1", Response::Ok),
            ("GET key1", Response::String(Some("value1".to_string()))),
            ("LPUSH list1 a b c", Response::Integer(3)),
            ("LRANGE list1 0 -1", Response::Array(vec![
                Response::String(Some("c".to_string())),
                Response::String(Some("b".to_string())),
                Response::String(Some("a".to_string())),
            ])),
            ("HSET hash1 field1 value1", Response::Integer(1)),
            ("HGET hash1 field1", Response::String(Some("value1".to_string()))),
            ("ZADD zset1 1.0 member1 2.0 member2", Response::Integer(2)),
        ];
        
        for (cmd, expected) in test_commands {
            let request = Request::parse(cmd).unwrap();
            let response = executor.execute(request).await.unwrap();
            assert_eq!(response, expected, "Command {} failed", cmd);
        }
        
        #[cfg(feature = "memory_pool")]
        {
            let stats = get_memory_stats();
            println!("Full stack test memory stats:");
            println!("  Total allocations: {}", stats.allocations);
            println!("  Pool hit rate: {:.2}%", 
                (stats.pool_hits as f64 / stats.allocations as f64) * 100.0);
        }
    }
    
    #[tokio::test]
    async fn test_concurrent_operations_with_optimizations() {
        use tokio::task;
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        #[cfg(feature = "memory_pool")]
        init_memory_pool().unwrap();
        
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
        let executor = Arc::new(CommandExecutor::new(storage.clone()));
        let counter = Arc::new(AtomicUsize::new(0));
        
        let mut handles = vec![];
        
        for thread_id in 0..10 {
            let executor_clone = executor.clone();
            let counter_clone = counter.clone();
            
            let handle = task::spawn(async move {
                for i in 0..100 {
                    let key = format!("key_{}_{}", thread_id, i);
                    let value = format!("value_{}_{}", thread_id, i);
                    
                    // SET
                    let set_cmd = format!("SET {} {}", key, value);
                    let request = Request::parse(&set_cmd).unwrap();
                    executor_clone.execute(request).await.unwrap();
                    
                    // GET
                    let get_cmd = format!("GET {}", key);
                    let request = Request::parse(&get_cmd).unwrap();
                    let response = executor_clone.execute(request).await.unwrap();
                    
                    match response {
                        Response::String(Some(v)) => assert_eq!(v, value),
                        _ => panic!("Unexpected response"),
                    }
                    
                    counter_clone.fetch_add(2, Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        let total_ops = counter.load(Ordering::Relaxed);
        assert_eq!(total_ops, 2000); // 10 threads * 100 iterations * 2 ops
        
        println!("Concurrent operations test completed: {} operations", total_ops);
    }
}