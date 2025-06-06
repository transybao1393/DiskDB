use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use diskdb::storage::rocksdb_storage::RocksDBStorage;
use diskdb::storage::Storage;
use diskdb::data_types::DataType;
use std::sync::Arc;
use std::collections::{HashMap, HashSet, BTreeMap};
use tokio::runtime::Runtime;
use tempfile::TempDir;

// Custom allocator to track allocations
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct AllocCounter;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for AllocCounter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        DEALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
    }
}

fn reset_alloc_counter() {
    ALLOCATED.store(0, Ordering::SeqCst);
    DEALLOCATED.store(0, Ordering::SeqCst);
}

fn get_allocated_bytes() -> usize {
    ALLOCATED.load(Ordering::SeqCst) - DEALLOCATED.load(Ordering::SeqCst)
}

fn bench_memory_per_key_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/per_key_type");
    let runtime = Runtime::new().unwrap();
    
    // Measure memory usage for different data types
    group.bench_function("string_1kb", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let storage = Arc::new(RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
                
                reset_alloc_counter();
                let before = get_allocated_bytes();
                
                // Store 100 1KB strings
                for i in 0..100 {
                    let key = format!("key_{}", i);
                    let value = "x".repeat(1024);
                    storage.set(&key, DataType::String(value)).await.unwrap();
                }
                
                let after = get_allocated_bytes();
                black_box((after - before) / 100) // Memory per key
            })
        })
    });
    
    group.bench_function("list_100_items", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let storage = Arc::new(RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
                
                reset_alloc_counter();
                let before = get_allocated_bytes();
                
                // Store 10 lists with 100 items each
                for i in 0..10 {
                    let key = format!("list_{}", i);
                    let values: Vec<String> = (0..100).map(|j| format!("item_{}", j)).collect();
                    storage.set(&key, DataType::List(values)).await.unwrap();
                }
                
                let after = get_allocated_bytes();
                black_box((after - before) / 10) // Memory per list
            })
        })
    });
    
    group.bench_function("hash_100_fields", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let storage = Arc::new(RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
                
                reset_alloc_counter();
                let before = get_allocated_bytes();
                
                // Store 10 hashes with 100 fields each
                for i in 0..10 {
                    let key = format!("hash_{}", i);
                    let fields: HashMap<String, String> = (0..100)
                        .map(|j| (format!("field_{}", j), format!("value_{}", j)))
                        .collect();
                    storage.set(&key, DataType::Hash(fields)).await.unwrap();
                }
                
                let after = get_allocated_bytes();
                black_box((after - before) / 10) // Memory per hash
            })
        })
    });
    
    group.finish();
}

fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/allocation_patterns");
    let runtime = Runtime::new().unwrap();
    
    // Measure allocation patterns during operations
    group.bench_function("lpush_allocations", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let storage = Arc::new(RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
                
                // Pre-create a list
                let initial_list: Vec<String> = (0..100).map(|i| format!("item_{}", i)).collect();
                storage.set("mylist", DataType::List(initial_list)).await.unwrap();
                
                reset_alloc_counter();
                
                // Measure allocations for LPUSH
                for i in 0..10 {
                    if let Some(DataType::List(mut list)) = storage.get("mylist").await.unwrap() {
                        list.insert(0, format!("new_item_{}", i));
                        storage.set("mylist", DataType::List(list)).await.unwrap();
                    }
                }
                
                black_box(get_allocated_bytes() / 10) // Allocations per LPUSH
            })
        })
    });
    
    group.bench_function("zrange_allocations", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let storage = Arc::new(RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
                
                // Pre-create a sorted set
                let members: BTreeMap<String, f64> = (0..1000)
                    .map(|i| (format!("member_{}", i), i as f64))
                    .collect();
                storage.set("myzset", DataType::SortedSet(members)).await.unwrap();
                
                reset_alloc_counter();
                
                // Measure allocations for ZRANGE
                for _ in 0..10 {
                    if let Some(DataType::SortedSet(zset)) = storage.get("myzset").await.unwrap() {
                        let _range: Vec<_> = zset.iter().take(100).collect();
                        black_box(_range);
                    }
                }
                
                black_box(get_allocated_bytes() / 10) // Allocations per ZRANGE
            })
        })
    });
    
    group.finish();
}

fn bench_memory_fragmentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/fragmentation");
    let runtime = Runtime::new().unwrap();
    
    group.bench_function("repeated_updates", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let storage = Arc::new(RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
                
                reset_alloc_counter();
                let initial = get_allocated_bytes();
                
                // Simulate workload that causes fragmentation
                for cycle in 0..10 {
                    // Create keys
                    for i in 0..100 {
                        let key = format!("key_{}_{}", cycle, i);
                        let value = "x".repeat(100 + (i % 10) * 100); // Varying sizes
                        storage.set(&key, DataType::String(value)).await.unwrap();
                    }
                    
                    // Delete half the keys
                    for i in 0..50 {
                        let key = format!("key_{}_{}", cycle, i * 2);
                        storage.delete(&key).await.unwrap();
                    }
                }
                
                let final_allocated = get_allocated_bytes();
                black_box((final_allocated - initial) / 1000) // Average memory per surviving key
            })
        })
    });
    
    group.finish();
}

fn bench_serialization_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory/serialization");
    let runtime = Runtime::new().unwrap();
    
    // Measure serialization overhead for different data types
    group.bench_function("json_serialization", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let storage = Arc::new(RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
                
                let json_value = serde_json::json!({
                    "user": {
                        "name": "Alice",
                        "age": 30,
                        "emails": ["alice@example.com", "alice@work.com"],
                        "settings": {
                            "theme": "dark",
                            "notifications": true
                        }
                    }
                });
                
                reset_alloc_counter();
                
                // Store JSON values
                for i in 0..10 {
                    let key = format!("json_{}", i);
                    storage.set(&key, DataType::Json(json_value.clone())).await.unwrap();
                }
                
                black_box(get_allocated_bytes() / 10) // Memory per JSON value
            })
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_memory_per_key_type,
    bench_allocation_patterns,
    bench_memory_fragmentation,
    bench_serialization_overhead
);
criterion_main!(benches);