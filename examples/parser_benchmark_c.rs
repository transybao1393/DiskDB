#![cfg(feature = "c_parser")]

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
    
    println!("Comparing Rust parser vs C parser\n");
    
    for (cmd, description) in test_cases {
        println!("Test: {}", description);
        
        // Test Rust parser
        let iterations = 100000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = Request::parse_rust(cmd).unwrap();
        }
        let rust_duration = start.elapsed();
        let rust_ops_per_sec = iterations as f64 / rust_duration.as_secs_f64();
        
        println!("  Rust parser: {:.0} ops/sec ({:.3} µs/op)", 
                 rust_ops_per_sec, 
                 rust_duration.as_micros() as f64 / iterations as f64);
        
        // Test C parser (uses Request::parse which will use C parser when feature is enabled)
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = Request::parse(cmd).unwrap();
        }
        let c_duration = start.elapsed();
        let c_ops_per_sec = iterations as f64 / c_duration.as_secs_f64();
        
        println!("  C parser:    {:.0} ops/sec ({:.3} µs/op)", 
                 c_ops_per_sec, 
                 c_duration.as_micros() as f64 / iterations as f64);
        
        let improvement = c_ops_per_sec / rust_ops_per_sec;
        println!("  Improvement: {:.1}x faster\n", improvement);
    }
    
    // Memory allocation comparison
    println!("Memory Allocation Analysis:");
    println!("  Rust parser: ~3-5 allocations per parse (split_whitespace, to_string, join)");
    println!("  C parser:    0 allocations during parse (thread-local arena)");
    println!("               1 allocation when converting to Rust types");
}

#[cfg(not(feature = "c_parser"))]
fn main() {
    println!("This example requires the c_parser feature.");
    println!("Run with: cargo run --features c_parser --example parser_benchmark_c");
}