#[cfg(test)]
mod security_tests {
    use diskdb::protocol::Request;
    use diskdb::error::DiskDBError;
    
    #[cfg(feature = "c_parser")]
    use diskdb::ffi::parser::SafeParser;
    
    #[cfg(feature = "memory_pool")]
    use diskdb::ffi::memory::{init_memory_pool, PooledString, PooledVec, get_memory_stats};
    
    #[test]
    fn test_parser_buffer_overflow_attempts() {
        // Test extremely long keys
        let long_key = "A".repeat(1_000_000);
        let cmd = format!("GET {}", long_key);
        match Request::parse(&cmd) {
            Ok(Request::Get { key }) => {
                assert_eq!(key.len(), 1_000_000);
            }
            Err(_) => {
                // Also acceptable if parser rejects extremely long input
            }
            _ => panic!("Unexpected result"),
        }
        
        // Test extremely long values
        let long_value = "B".repeat(10_000_000);
        let cmd = format!("SET key {}", long_value);
        match Request::parse(&cmd) {
            Ok(Request::Set { key: _, value }) => {
                assert_eq!(value.len(), 10_000_000);
            }
            Err(_) => {
                // Also acceptable if parser rejects extremely long input
            }
            _ => panic!("Unexpected result"),
        }
        
        // Test many arguments
        let many_args = vec!["item"; 10000].join(" ");
        let cmd = format!("LPUSH list {}", many_args);
        match Request::parse(&cmd) {
            Ok(Request::LPush { key: _, values }) => {
                assert_eq!(values.len(), 10000);
            }
            Err(_) => {
                // Also acceptable if parser rejects too many arguments
            }
            _ => panic!("Unexpected result"),
        }
    }
    
    #[test]
    fn test_parser_malformed_input() {
        let repeated_spaces = format!("GET {}", "key ".repeat(1000));
        let malformed_inputs = vec![
            "",                          // Empty input
            " ",                         // Just whitespace
            "\n\r\t",                   // Just whitespace characters
            "GET",                      // Missing argument
            "SET key",                  // Missing value
            "UNKNOWN_CMD key",          // Unknown command
            "GET key extra",            // Too many arguments
            "SET",                      // Missing both arguments
            "LPUSH",                    // Missing all arguments
            "ZADD zset not_a_number member", // Invalid score
            "INCRBY key not_a_number", // Invalid increment
            "GET\0key",                 // Null byte in command
            "SET key\0 value",          // Null byte in key
            "SET key val\0ue",          // Null byte in value
            "\u{FFFD}\u{FFFE}\u{FFFD}",  // Invalid characters
            &repeated_spaces,           // Repeated spaces
        ];
        
        for input in malformed_inputs {
            match Request::parse(input) {
                Err(DiskDBError::Protocol(_)) => {
                    // Expected - parser correctly rejects malformed input
                }
                Ok(_) => {
                    // Some inputs might be valid (like empty value)
                    println!("Input '{}' was accepted", input);
                }
                Err(e) => {
                    panic!("Unexpected error type for input '{}': {:?}", input, e);
                }
            }
        }
    }
    
    #[test]
    fn test_parser_injection_attempts() {
        // Test command injection attempts
        let injection_attempts = vec![
            "GET key\nSET injected value",     // Newline injection
            "GET key\rSET injected value",     // Carriage return injection
            "GET key; SET injected value",     // Semicolon injection
            "GET key && SET injected value",   // Shell-style command chaining
            "GET key | SET injected value",    // Pipe injection
            "GET ${key}",                       // Variable expansion attempt
            "GET `whoami`",                     // Command substitution attempt
            "GET ../../../etc/passwd",          // Path traversal attempt
            "GET key%00value",                  // Null byte injection
            "GET key%0Avalue",                  // URL-encoded newline
        ];
        
        for attempt in injection_attempts {
            match Request::parse(attempt) {
                Ok(request) => {
                    // Verify the parser treats these as literal values
                    match request {
                        Request::Get { key } => {
                            // The entire string after GET should be the key
                            assert!(key.contains(&attempt[4..]));
                        }
                        _ => panic!("Unexpected request type"),
                    }
                }
                Err(_) => {
                    // Also acceptable if parser rejects suspicious input
                }
            }
        }
    }
    
    #[cfg(feature = "c_parser")]
    #[test]
    fn test_c_parser_memory_safety() {
        use std::thread;
        use std::sync::Arc;
        
        // Test concurrent access to parser
        let test_inputs = Arc::new(vec![
            "GET key1",
            "SET key1 value1",
            "LPUSH list1 a b c d e f g h i j k l m n o p",
            "HSET hash1 field1 value1",
            "ZADD zset1 1.0 member1 2.0 member2 3.0 member3",
        ]);
        
        let mut handles = vec![];
        for _ in 0..10 {
            let inputs = test_inputs.clone();
            let handle = thread::spawn(move || {
                for _ in 0..1000 {
                    for input in inputs.iter() {
                        let _ = Request::parse(input);
                    }
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Test rapid allocation/deallocation
        for _ in 0..10000 {
            let parser = SafeParser::new(1024 * 1024).unwrap();
            drop(parser);
        }
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn test_memory_pool_double_free_protection() {
        init_memory_pool().unwrap();
        
        // Test that dropping twice doesn't cause issues (Rust prevents this)
        {
            let s1 = PooledString::from_str("test").unwrap();
            let s2 = s1.clone();
            drop(s1);
            // s2 still valid
            assert_eq!(s2.as_str(), "test");
        } // s2 dropped here
        
        // Verify no memory corruption
        let s3 = PooledString::from_str("another test").unwrap();
        assert_eq!(s3.as_str(), "another test");
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn test_memory_pool_bounds_checking() {
        init_memory_pool().unwrap();
        
        // Test PooledVec bounds
        let mut vec = PooledVec::<i32>::with_capacity(10).unwrap();
        for i in 0..10 {
            vec.push(i).unwrap();
        }
        
        // Test safe access
        assert_eq!(vec.len(), 10);
        assert_eq!(vec.as_slice()[0], 0);
        assert_eq!(vec.as_slice()[9], 9);
        
        // Rust's bounds checking prevents out-of-bounds access
        // This would panic: vec.as_slice()[10]
        
        // Test pop on empty
        let mut vec2 = PooledVec::<i32>::new();
        assert_eq!(vec2.pop(), None);
    }
    
    #[cfg(feature = "memory_pool")]
    #[test]
    fn test_memory_pool_thread_safety() {
        use std::thread;
        use std::sync::{Arc, Barrier};
        
        init_memory_pool().unwrap();
        
        let barrier = Arc::new(Barrier::new(10));
        let mut handles = vec![];
        
        for thread_id in 0..10 {
            let barrier_clone = barrier.clone();
            let handle = thread::spawn(move || {
                // Synchronize thread start
                barrier_clone.wait();
                
                // Each thread performs many allocations
                let mut allocations = Vec::new();
                for i in 0..1000 {
                    let s = PooledString::from_str(&format!("thread_{}_string_{}", thread_id, i)).unwrap();
                    allocations.push(s);
                }
                
                // Verify all strings are correct
                for (i, s) in allocations.iter().enumerate() {
                    assert_eq!(s.as_str(), format!("thread_{}_string_{}", thread_id, i));
                }
                
                // Let them deallocate
                drop(allocations);
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let stats = get_memory_stats();
        assert_eq!(stats.active_objects, 0, "Memory leak detected!");
    }
    
    #[test]
    fn test_parser_unicode_handling() {
        let unicode_tests = vec![
            ("GET 你好", "你好"),
            ("SET κλειδί τιμή", "κλειδί"),
            ("GET emoji🔑", "emoji🔑"),
            ("SET ключ значение", "ключ"),
            ("GET مفتاح", "مفتاح"),
        ];
        
        for (input, expected_key) in unicode_tests {
            match Request::parse(input) {
                Ok(request) => {
                    match request {
                        Request::Get { key } => assert_eq!(key, expected_key),
                        Request::Set { key, .. } => assert_eq!(key, expected_key),
                        _ => panic!("Unexpected request type"),
                    }
                }
                Err(e) => panic!("Failed to parse valid unicode: {:?}", e),
            }
        }
    }
    
    #[test]
    fn test_parser_edge_cases() {
        // Test edge cases that should be handled gracefully
        let edge_cases = vec![
            ("GET    key", "key"),           // Multiple spaces
            ("GET\tkey", "key"),             // Tab character
            ("GET key ", "key"),             // Trailing space
            (" GET key", "key"),             // Leading space
            ("get KEY", "KEY"),              // Case insensitive command
            ("SET key  value  with  spaces", "key"), // Multiple spaces in value
        ];
        
        for (input, expected_key) in edge_cases {
            match Request::parse(input) {
                Ok(request) => {
                    match request {
                        Request::Get { key } => assert_eq!(key, expected_key),
                        Request::Set { key, .. } => assert_eq!(key, expected_key),
                        _ => {} // Other valid interpretations
                    }
                }
                Err(e) => {
                    println!("Edge case '{}' rejected: {:?}", input, e);
                }
            }
        }
    }
}