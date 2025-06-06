use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use diskdb::storage::rocksdb_storage::RocksDBStorage;
use diskdb::storage::Storage;
use diskdb::data_types::DataType;
use diskdb::commands::CommandExecutor;
use diskdb::protocol::{Request, Response};
use std::sync::Arc;
use std::collections::{HashMap, HashSet, BTreeMap};
use tokio::runtime::Runtime;
use tempfile::TempDir;

fn create_test_storage() -> Arc<RocksDBStorage> {
    let temp_dir = TempDir::new().unwrap();
    let storage = RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap();
    Arc::new(storage)
}

fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("operations/string");
    let runtime = Runtime::new().unwrap();
    let storage = create_test_storage();
    let executor = Arc::new(CommandExecutor::new(storage.clone()));
    
    // SET operation - small value
    group.bench_function("set_small", |b| {
        b.to_async(&runtime).iter(|| async {
            let request = Request::Set { 
                key: "testkey".to_string(), 
                value: "small".to_string() 
            };
            executor.execute(black_box(request)).await.unwrap()
        })
    });
    
    // SET operation - 1KB value
    let kb_value = "x".repeat(1024);
    group.bench_function("set_1kb", |b| {
        b.to_async(&runtime).iter(|| async {
            let request = Request::Set { 
                key: "testkey".to_string(), 
                value: kb_value.clone() 
            };
            executor.execute(black_box(request)).await.unwrap()
        })
    });
    
    // GET operation
    runtime.block_on(async {
        storage.set("getkey", DataType::String("testvalue".to_string())).await.unwrap();
    });
    group.bench_function("get_existing", |b| {
        b.to_async(&runtime).iter(|| async {
            let request = Request::Get { key: "getkey".to_string() };
            executor.execute(black_box(request)).await.unwrap()
        })
    });
    
    // INCR operation
    runtime.block_on(async {
        storage.set("counter", DataType::String("0".to_string())).await.unwrap();
    });
    group.bench_function("incr", |b| {
        b.to_async(&runtime).iter(|| async {
            let request = Request::Incr { key: "counter".to_string() };
            executor.execute(black_box(request)).await.unwrap()
        })
    });
    
    group.finish();
}

fn bench_list_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("operations/list");
    let runtime = Runtime::new().unwrap();
    let storage = create_test_storage();
    let executor = Arc::new(CommandExecutor::new(storage.clone()));
    
    // LPUSH - worst case (front insertion)
    group.bench_function("lpush_single", |b| {
        b.to_async(&runtime).iter(|| async {
            let request = Request::LPush { 
                key: "mylist".to_string(), 
                values: vec!["item".to_string()] 
            };
            executor.execute(black_box(request)).await.unwrap()
        })
    });
    
    // LPUSH multiple values
    group.bench_function("lpush_10_items", |b| {
        b.to_async(&runtime).iter(|| async {
            let request = Request::LPush { 
                key: "mylist".to_string(), 
                values: (0..10).map(|i| format!("item{}", i)).collect() 
            };
            executor.execute(black_box(request)).await.unwrap()
        })
    });
    
    // RPUSH - best case (back insertion)
    group.bench_function("rpush_single", |b| {
        b.to_async(&runtime).iter(|| async {
            let request = Request::RPush { 
                key: "mylist".to_string(), 
                values: vec!["item".to_string()] 
            };
            executor.execute(black_box(request)).await.unwrap()
        })
    });
    
    // LRANGE on different list sizes
    for size in [10, 100, 1000] {
        runtime.block_on(async {
            let list_key = format!("list_{}", size);
            let values: Vec<String> = (0..size).map(|i| format!("item{}", i)).collect();
            storage.set(&list_key, DataType::List(values)).await.unwrap();
        });
        
        group.bench_with_input(
            BenchmarkId::new("lrange_full", size),
            &size,
            |b, &size| {
                b.to_async(&runtime).iter(|| async {
                    let request = Request::LRange { 
                        key: format!("list_{}", size), 
                        start: 0, 
                        stop: -1 
                    };
                    executor.execute(black_box(request)).await.unwrap()
                })
            }
        );
    }
    
    group.finish();
}

fn bench_set_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("operations/set");
    let runtime = Runtime::new().unwrap();
    let storage = create_test_storage();
    let executor = Arc::new(CommandExecutor::new(storage.clone()));
    
    // SADD single member
    group.bench_function("sadd_single", |b| {
        b.to_async(&runtime).iter(|| async {
            let request = Request::SAdd { 
                key: "myset".to_string(), 
                members: vec!["member".to_string()] 
            };
            executor.execute(black_box(request)).await.unwrap()
        })
    });
    
    // SISMEMBER on different set sizes
    for size in [10, 100, 1000] {
        runtime.block_on(async {
            let set_key = format!("set_{}", size);
            let members: HashSet<String> = (0..size).map(|i| format!("member{}", i)).collect();
            storage.set(&set_key, DataType::Set(members)).await.unwrap();
        });
        
        group.bench_with_input(
            BenchmarkId::new("sismember", size),
            &size,
            |b, &size| {
                b.to_async(&runtime).iter(|| async {
                    let request = Request::SIsMember { 
                        key: format!("set_{}", size), 
                        member: format!("member{}", size/2) // Middle element
                    };
                    executor.execute(black_box(request)).await.unwrap()
                })
            }
        );
    }
    
    group.finish();
}

fn bench_hash_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("operations/hash");
    let runtime = Runtime::new().unwrap();
    let storage = create_test_storage();
    let executor = Arc::new(CommandExecutor::new(storage.clone()));
    
    // HSET single field
    group.bench_function("hset_single", |b| {
        b.to_async(&runtime).iter(|| async {
            let request = Request::HSet { 
                key: "myhash".to_string(), 
                field: "field".to_string(),
                value: "value".to_string()
            };
            executor.execute(black_box(request)).await.unwrap()
        })
    });
    
    // HGETALL on different hash sizes
    for size in [10, 100, 1000] {
        runtime.block_on(async {
            let hash_key = format!("hash_{}", size);
            let fields: HashMap<String, String> = (0..size)
                .map(|i| (format!("field{}", i), format!("value{}", i)))
                .collect();
            storage.set(&hash_key, DataType::Hash(fields)).await.unwrap();
        });
        
        group.bench_with_input(
            BenchmarkId::new("hgetall", size),
            &size,
            |b, &size| {
                b.to_async(&runtime).iter(|| async {
                    let request = Request::HGetAll { 
                        key: format!("hash_{}", size)
                    };
                    executor.execute(black_box(request)).await.unwrap()
                })
            }
        );
    }
    
    group.finish();
}

fn bench_sorted_set_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("operations/sorted_set");
    let runtime = Runtime::new().unwrap();
    let storage = create_test_storage();
    let executor = Arc::new(CommandExecutor::new(storage.clone()));
    
    // ZADD single member
    group.bench_function("zadd_single", |b| {
        b.to_async(&runtime).iter(|| async {
            let request = Request::ZAdd { 
                key: "myzset".to_string(), 
                members: vec![("member".to_string(), 1.0)]
            };
            executor.execute(black_box(request)).await.unwrap()
        })
    });
    
    // ZRANGE on different sorted set sizes
    for size in [10, 100, 1000] {
        runtime.block_on(async {
            let zset_key = format!("zset_{}", size);
            let members: BTreeMap<String, f64> = (0..size)
                .map(|i| (format!("member{}", i), i as f64))
                .collect();
            storage.set(&zset_key, DataType::SortedSet(members)).await.unwrap();
        });
        
        group.bench_with_input(
            BenchmarkId::new("zrange_full", size),
            &size,
            |b, &size| {
                b.to_async(&runtime).iter(|| async {
                    let request = Request::ZRange { 
                        key: format!("zset_{}", size),
                        start: 0,
                        stop: -1,
                        with_scores: false
                    };
                    executor.execute(black_box(request)).await.unwrap()
                })
            }
        );
    }
    
    group.finish();
}

fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("operations/concurrent");
    let runtime = Runtime::new().unwrap();
    let storage = create_test_storage();
    let executor = Arc::new(CommandExecutor::new(storage.clone()));
    
    // Concurrent SET operations
    for threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("concurrent_sets", threads),
            &threads,
            |b, &thread_count| {
                b.to_async(&runtime).iter(|| async {
                    let mut handles = vec![];
                    
                    for t in 0..thread_count {
                        let exec = executor.clone();
                        let handle = tokio::spawn(async move {
                            for i in 0..100 {
                                let request = Request::Set {
                                    key: format!("key_{}_{}", t, i),
                                    value: "value".to_string()
                                };
                                exec.execute(request).await.unwrap();
                            }
                        });
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        handle.await.unwrap();
                    }
                })
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_string_operations,
    bench_list_operations,
    bench_set_operations,
    bench_hash_operations,
    bench_sorted_set_operations,
    bench_concurrent_operations
);
criterion_main!(benches);