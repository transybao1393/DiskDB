use diskdb::protocol::Request;
use std::time::Instant;

fn main() {
    println!("\n=== DiskDB Parser Performance Comparison ===\n");
    
    let large_value = format!("SET largekey {}", "x".repeat(1000));
    let test_cases = vec![
        ("GET mykey", "Simple GET"),
        ("SET mykey myvalue", "Simple SET"),
        ("LPUSH mylist item1 item2 item3 item4 item5", "LPUSH with 5 items"),
        ("ZADD myset 1 one 2 two 3 three 4 four 5 five", "ZADD with 5 members"),
        (large_value.as_str(), "SET with 1KB value"),
    ];
    
    println!("Testing Rust parser only (C parser not enabled in this build)\n");
    
    for (cmd, description) in test_cases {
        println!("Test: {}", description);
        
        // Warm up
        for _ in 0..100 {
            let _ = Request::parse_rust(cmd).unwrap();
        }
        
        // Test Rust parser
        let iterations = 100000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = Request::parse_rust(cmd).unwrap();
        }
        let duration = start.elapsed();
        let ops_per_sec = iterations as f64 / duration.as_secs_f64();
        
        println!("  Rust parser: {:.0} ops/sec ({:.3} µs/op)\n", 
                 ops_per_sec, 
                 duration.as_micros() as f64 / iterations as f64);
    }
    
    println!("\nNote: To test C parser, compile with: cargo run --features c_parser --example parser_benchmark");
}