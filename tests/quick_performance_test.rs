use diskdb::protocol::Request;
use std::time::Instant;

#[test]
fn quick_protocol_parsing_test() {
    println!("\n=== Quick Protocol Parsing Test ===");
    
    // Test simple GET
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = Request::parse("GET mykey").unwrap();
    }
    let duration = start.elapsed();
    let ops_per_sec = 10000.0 / duration.as_secs_f64();
    println!("GET parsing: {:.0} ops/sec ({:.3} ms/op)", ops_per_sec, duration.as_millis() as f64 / 10000.0);
    
    // Test SET command
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = Request::parse("SET mykey myvalue").unwrap();
    }
    let duration = start.elapsed();
    let ops_per_sec = 10000.0 / duration.as_secs_f64();
    println!("SET parsing: {:.0} ops/sec ({:.3} ms/op)", ops_per_sec, duration.as_millis() as f64 / 10000.0);
    
    // Test complex ZADD
    let zadd_cmd = "ZADD myset 1 one 2 two 3 three 4 four 5 five";
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = Request::parse(zadd_cmd).unwrap();
    }
    let duration = start.elapsed();
    let ops_per_sec = 10000.0 / duration.as_secs_f64();
    println!("ZADD (5 members) parsing: {:.0} ops/sec ({:.3} ms/op)", ops_per_sec, duration.as_millis() as f64 / 10000.0);
    
    // Test large SET
    let large_value = "x".repeat(1000);
    let large_cmd = format!("SET mykey {}", large_value);
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = Request::parse(&large_cmd).unwrap();
    }
    let duration = start.elapsed();
    let ops_per_sec = 1000.0 / duration.as_secs_f64();
    println!("SET (1KB value) parsing: {:.0} ops/sec ({:.3} ms/op)", ops_per_sec, duration.as_millis() as f64 / 1000.0);
}

#[test]
fn quick_storage_test() {
    use diskdb::storage::rocksdb_storage::RocksDBStorage;
    use diskdb::storage::Storage;
    use diskdb::data_types::DataType;
    use tempfile::TempDir;
    use tokio::runtime::Runtime;
    
    println!("\n=== Quick Storage Operations Test ===");
    
    let runtime = Runtime::new().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let storage = RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap();
    
    runtime.block_on(async {
        // Test SET operation
        let start = Instant::now();
        for i in 0..1000 {
            storage.set(&format!("key_{}", i), DataType::String("value".to_string())).await.unwrap();
        }
        let duration = start.elapsed();
        let ops_per_sec = 1000.0 / duration.as_secs_f64();
        println!("SET storage: {:.0} ops/sec ({:.3} ms/op)", ops_per_sec, duration.as_millis() as f64 / 1000.0);
        
        // Test GET operation
        let start = Instant::now();
        for i in 0..1000 {
            let _ = storage.get(&format!("key_{}", i)).await.unwrap();
        }
        let duration = start.elapsed();
        let ops_per_sec = 1000.0 / duration.as_secs_f64();
        println!("GET storage: {:.0} ops/sec ({:.3} ms/op)", ops_per_sec, duration.as_millis() as f64 / 1000.0);
    });
}