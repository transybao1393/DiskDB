use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use diskdb::protocol::Request;
use diskdb::data_types::DataType;
use std::collections::HashMap;

#[cfg(feature = "memory_pool")]
use diskdb::data_types_pooled::PooledStorageOps;

#[cfg(feature = "memory_pool")]
use diskdb::ffi::memory::{init_memory_pool, get_memory_stats, reset_memory_stats};

fn benchmark_parser_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_comparison");
    
    let test_commands = vec![
        "GET key1",
        "SET key1 value1",
        "LPUSH list1 item1 item2 item3 item4 item5",
        "HSET hash1 field1 value1",
        "ZADD zset1 1.0 member1 2.0 member2 3.0 member3",
        "XADD stream1 * field1 value1 field2 value2",
    ];
    
    // Benchmark Rust parser
    group.bench_function("rust_parser", |b| {
        b.iter(|| {
            for cmd in &test_commands {
                let _ = Request::parse_rust(black_box(cmd)).unwrap();
            }
        });
    });
    
    // Benchmark C parser if available
    #[cfg(feature = "c_parser")]
    group.bench_function("c_parser", |b| {
        b.iter(|| {
            for cmd in &test_commands {
                let _ = Request::parse(black_box(cmd)).unwrap();
            }
        });
    });
    
    group.finish();
}

fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    
    // Standard allocation
    group.bench_function("standard_string", |b| {
        b.iter(|| {
            let mut strings = Vec::new();
            for i in 0..100 {
                strings.push(DataType::String(format!("test_string_{}", i)));
            }
            black_box(strings);
        });
    });
    
    // Pooled allocation
    #[cfg(feature = "memory_pool")]
    {
        init_memory_pool().ok();
        
        group.bench_function("pooled_string", |b| {
            b.iter(|| {
                let mut strings = Vec::new();
                for i in 0..100 {
                    strings.push(PooledStorageOps::create_string(&format!("test_string_{}", i)).unwrap());
                }
                black_box(strings);
            });
        });
    }
    
    // List allocation comparison
    let sizes = vec![10, 100, 1000];
    
    for size in sizes {
        group.bench_with_input(
            BenchmarkId::new("standard_list", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut list = DataType::List(Vec::with_capacity(size));
                    for i in 0..size {
                        list.lpush(vec![format!("item{}", i)]).unwrap();
                    }
                    black_box(list);
                });
            }
        );
        
        #[cfg(feature = "memory_pool")]
        group.bench_with_input(
            BenchmarkId::new("pooled_list", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut list = PooledStorageOps::create_list(size).unwrap();
                    for i in 0..size {
                        list.lpush(vec![format!("item{}", i)]).unwrap();
                    }
                    black_box(list);
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_data_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_operations");
    
    // Hash operations
    group.bench_function("hash_operations", |b| {
        b.iter(|| {
            let mut hash = DataType::Hash(HashMap::new());
            for i in 0..100 {
                hash.hset(format!("field{}", i), format!("value{}", i)).unwrap();
            }
            for i in 0..100 {
                hash.hget(&format!("field{}", i)).unwrap();
            }
            black_box(hash);
        });
    });
    
    // Set operations
    group.bench_function("set_operations", |b| {
        b.iter(|| {
            let mut set = DataType::Set(std::collections::HashSet::new());
            for i in 0..100 {
                set.sadd(vec![format!("member{}", i)]).unwrap();
            }
            for i in 0..100 {
                set.sismember(&format!("member{}", i)).unwrap();
            }
            black_box(set);
        });
    });
    
    // Sorted set operations
    group.bench_function("zset_operations", |b| {
        b.iter(|| {
            let mut zset = DataType::SortedSet(std::collections::BTreeMap::new());
            for i in 0..100 {
                zset.zadd(vec![(i as f64, format!("member{}", i))]).unwrap();
            }
            let _ = zset.zrange(0, 50, false).unwrap();
            black_box(zset);
        });
    });
    
    group.finish();
}

fn benchmark_combined_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_workload");
    
    // Simulate realistic mixed workload
    group.bench_function("mixed_operations", |b| {
        b.iter(|| {
            let mut operations = Vec::new();
            
            // Parse commands
            let commands = vec![
                "SET user:1000 {\"name\":\"John\",\"age\":30}",
                "GET user:1000",
                "LPUSH notifications:1000 \"New message\"",
                "HSET session:abc123 user_id 1000",
                "ZADD leaderboard 9500 player1",
            ];
            
            for cmd in &commands {
                operations.push(Request::parse(cmd).unwrap());
            }
            
            // Create data structures
            let mut string_data = DataType::String("value".to_string());
            string_data.incr(1).unwrap();
            
            let mut list_data = DataType::List(Vec::new());
            list_data.lpush(vec!["item1".to_string(), "item2".to_string()]).unwrap();
            
            let mut hash_data = DataType::Hash(HashMap::new());
            hash_data.hset("field".to_string(), "value".to_string()).unwrap();
            
            black_box((operations, string_data, list_data, hash_data));
        });
    });
    
    group.finish();
}

#[cfg(feature = "memory_pool")]
fn print_memory_stats() {
    let stats = get_memory_stats();
    println!("\nMemory Pool Statistics:");
    println!("  Allocations: {}", stats.allocations);
    println!("  Deallocations: {}", stats.deallocations);
    println!("  Pool hits: {}", stats.pool_hits);
    println!("  Pool misses: {}", stats.pool_misses);
    println!("  Hit rate: {:.2}%", (stats.pool_hits as f64 / stats.allocations as f64) * 100.0);
    println!("  Active objects: {}", stats.active_objects);
}

criterion_group!(
    benches,
    benchmark_parser_comparison,
    benchmark_memory_allocation,
    benchmark_data_operations,
    benchmark_combined_workload
);

criterion_main!(benches);