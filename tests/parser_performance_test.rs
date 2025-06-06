use diskdb::protocol::Request;
use std::time::Instant;

#[test]
fn compare_parser_performance() {
    println!("\n=== Parser Performance Comparison ===\n");
    
    let large_value = format!("SET largekey {}", "x".repeat(1000));
    let test_cases = vec![
        ("GET mykey", "Simple GET"),
        ("SET mykey myvalue", "Simple SET"),
        ("LPUSH mylist item1 item2 item3 item4 item5", "LPUSH with 5 items"),
        ("ZADD myset 1 one 2 two 3 three 4 four 5 five", "ZADD with 5 members"),
        (large_value.as_str(), "SET with 1KB value"),
    ];
    
    for (cmd, description) in test_cases {
        println!("Test: {}", description);
        
        // Test Rust parser
        let start = Instant::now();
        for _ in 0..10000 {
            let _ = Request::parse_rust(cmd).unwrap();
        }
        let rust_duration = start.elapsed();
        let rust_ops_per_sec = 10000.0 / rust_duration.as_secs_f64();
        
        println!("  Rust parser: {:.0} ops/sec ({:.3} µs/op)", 
                 rust_ops_per_sec, 
                 rust_duration.as_micros() as f64 / 10000.0);
        
        // Test C parser if available
        #[cfg(feature = "c_parser")]
        {
            let start = Instant::now();
            for _ in 0..10000 {
                let _ = Request::parse(cmd).unwrap();
            }
            let c_duration = start.elapsed();
            let c_ops_per_sec = 10000.0 / c_duration.as_secs_f64();
            
            println!("  C parser:    {:.0} ops/sec ({:.3} µs/op)", 
                     c_ops_per_sec, 
                     c_duration.as_micros() as f64 / 10000.0);
            
            let improvement = c_ops_per_sec / rust_ops_per_sec;
            println!("  Improvement: {:.1}x faster\n", improvement);
        }
        
        #[cfg(not(feature = "c_parser"))]
        println!("  C parser: Not enabled (use --features c_parser)\n");
    }
}