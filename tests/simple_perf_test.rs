use std::time::Instant;
use diskdb::protocol::{Request, Response};

#[test]
fn test_basic_performance() {
    println!("\n=== DiskDB Basic Performance Tests ===\n");
    
    // Test 1: Protocol parsing performance
    println!("1. Protocol Parsing Performance:");
    let commands = vec![
        "GET key1",
        "SET key1 value1",
        "LPUSH list1 item1 item2 item3",
        "HSET hash1 field1 value1",
        "ZADD zset1 1.0 member1 2.0 member2",
    ];
    
    let start = Instant::now();
    let iterations = 100_000;
    
    for _ in 0..iterations {
        for cmd in &commands {
            let _ = Request::parse_rust(cmd).unwrap();
        }
    }
    
    let elapsed = start.elapsed();
    println!("  Parsed {} commands in {:?}", iterations * commands.len(), elapsed);
    println!("  Throughput: {:.0} commands/sec", (iterations * commands.len()) as f64 / elapsed.as_secs_f64());
    
    // Test 2: Response serialization performance
    println!("\n2. Response Serialization Performance:");
    let responses = vec![
        Response::Ok,
        Response::String(Some("test value".to_string())),
        Response::Integer(42),
        Response::Array(vec![
            Response::String(Some("item1".to_string())),
            Response::String(Some("item2".to_string())),
        ]),
        Response::Error("test error".to_string()),
    ];
    
    let start = Instant::now();
    
    for _ in 0..iterations {
        for resp in &responses {
            let _ = format!("{}", resp);
        }
    }
    
    let elapsed = start.elapsed();
    println!("  Serialized {} responses in {:?}", iterations * responses.len(), elapsed);
    println!("  Throughput: {:.0} responses/sec", (iterations * responses.len()) as f64 / elapsed.as_secs_f64());
    
    // Test 3: Memory allocation performance
    println!("\n3. Memory Allocation Performance:");
    let start = Instant::now();
    let alloc_count = 10_000;
    
    for i in 0..alloc_count {
        let mut vec = Vec::with_capacity(1024);
        vec.extend_from_slice(format!("test data {}", i).as_bytes());
        std::hint::black_box(vec);
    }
    
    let elapsed = start.elapsed();
    println!("  Allocated {} buffers in {:?}", alloc_count, elapsed);
    println!("  Rate: {:.0} allocations/sec", alloc_count as f64 / elapsed.as_secs_f64());
}

#[cfg(feature = "c_parser")]
#[test]
fn test_parser_comparison() {
    println!("\n=== Parser Performance Comparison ===\n");
    
    let commands = vec![
        "GET key1",
        "SET key1 value1",
        "LPUSH list1 item1 item2 item3",
        "HSET hash1 field1 value1",
        "ZADD zset1 1.0 member1 2.0 member2",
    ];
    
    let iterations = 100_000;
    
    // Test Rust parser
    let start = Instant::now();
    for _ in 0..iterations {
        for cmd in &commands {
            let _ = Request::parse_rust(cmd).unwrap();
        }
    }
    let rust_time = start.elapsed();
    
    // Test C parser
    let start = Instant::now();
    for _ in 0..iterations {
        for cmd in &commands {
            let _ = Request::parse(cmd).unwrap();
        }
    }
    let c_time = start.elapsed();
    
    println!("Rust parser: {:?}", rust_time);
    println!("C parser: {:?}", c_time);
    println!("C parser speedup: {:.2}x", rust_time.as_secs_f64() / c_time.as_secs_f64());
}

#[cfg(feature = "memory_pool")]
#[test]
fn test_memory_pool_performance() {
    use diskdb::data_types::DataType;
    use diskdb::data_types_pooled::PooledStorageOps;
    use diskdb::ffi::memory::{init_memory_pool, get_memory_stats};
    
    println!("\n=== Memory Pool Performance ===\n");
    
    init_memory_pool().ok();
    
    let iterations = 10_000;
    
    // Test standard allocation
    let start = Instant::now();
    for i in 0..iterations {
        let _ = DataType::String(format!("test_string_{}", i));
    }
    let std_time = start.elapsed();
    
    // Test pooled allocation
    let start = Instant::now();
    for i in 0..iterations {
        let _ = PooledStorageOps::create_string(&format!("test_string_{}", i)).unwrap();
    }
    let pool_time = start.elapsed();
    
    println!("Standard allocation: {:?}", std_time);
    println!("Pooled allocation: {:?}", pool_time);
    println!("Memory pool speedup: {:.2}x", std_time.as_secs_f64() / pool_time.as_secs_f64());
    
    let stats = get_memory_stats();
    println!("\nMemory pool statistics:");
    println!("  Total allocations: {}", stats.allocations);
    println!("  Pool hits: {}", stats.pool_hits);
    println!("  Hit rate: {:.1}%", (stats.pool_hits as f64 / stats.allocations as f64) * 100.0);
}