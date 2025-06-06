use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use diskdb::data_types::DataType;
use diskdb::data_types_pooled::PooledStorageOps;
use std::collections::{HashMap, HashSet};

fn benchmark_string_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_allocation");
    
    let large_string = "x".repeat(1000);
    let test_strings = vec![
        ("small", "hello"),
        ("medium", "This is a medium length string for testing memory allocation"),
        ("large", large_string.as_str()),
    ];
    
    for (name, string) in test_strings {
        // Standard allocation
        group.bench_with_input(
            BenchmarkId::new("standard", name),
            &string,
            |b, s| {
                b.iter(|| {
                    DataType::String(s.to_string())
                });
            }
        );
        
        // Pooled allocation
        group.bench_with_input(
            BenchmarkId::new("pooled", name),
            &string,
            |b, s| {
                b.iter(|| {
                    PooledStorageOps::create_string(s).unwrap()
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_list_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_operations");
    
    let sizes = vec![10, 100, 1000];
    
    for size in sizes {
        // Standard Vec allocation and push
        group.bench_with_input(
            BenchmarkId::new("standard_push", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut data = DataType::List(Vec::new());
                    for i in 0..size {
                        data.lpush(vec![format!("item{}", i)]).unwrap();
                    }
                    data
                });
            }
        );
        
        // Pooled list operations (simulated)
        group.bench_with_input(
            BenchmarkId::new("pooled_push", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut data = PooledStorageOps::create_list(size).unwrap();
                    for i in 0..size {
                        data.lpush(vec![format!("item{}", i)]).unwrap();
                    }
                    data
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_hash_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_operations");
    
    let sizes = vec![10, 100, 1000];
    
    for size in sizes {
        // Standard HashMap allocation
        group.bench_with_input(
            BenchmarkId::new("standard", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut data = DataType::Hash(HashMap::new());
                    for i in 0..size {
                        data.hset(format!("field{}", i), format!("value{}", i)).unwrap();
                    }
                    data
                });
            }
        );
    }
    
    group.finish();
}

fn benchmark_mixed_operations(c: &mut Criterion) {
    c.bench_function("mixed_allocation_standard", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            
            // Allocate strings
            for i in 0..100 {
                results.push(DataType::String(format!("string{}", i)));
            }
            
            // Allocate lists
            for i in 0..50 {
                let mut list = DataType::List(Vec::new());
                list.lpush(vec![format!("item{}", i)]).unwrap();
                results.push(list);
            }
            
            // Allocate sets
            for i in 0..50 {
                let mut set = DataType::Set(HashSet::new());
                set.sadd(vec![format!("member{}", i)]).unwrap();
                results.push(set);
            }
            
            results
        });
    });
    
    c.bench_function("mixed_allocation_pooled", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            
            // Allocate strings with pool
            for i in 0..100 {
                results.push(PooledStorageOps::create_string(&format!("string{}", i)).unwrap());
            }
            
            // Allocate lists with pool
            for i in 0..50 {
                let mut list = PooledStorageOps::create_list(10).unwrap();
                list.lpush(vec![format!("item{}", i)]).unwrap();
                results.push(list);
            }
            
            // Allocate sets
            for i in 0..50 {
                let mut set = DataType::Set(HashSet::new());
                set.sadd(vec![format!("member{}", i)]).unwrap();
                results.push(set);
            }
            
            results
        });
    });
}

#[cfg(feature = "memory_pool")]
fn benchmark_memory_stats(c: &mut Criterion) {
    use diskdb::ffi::memory::{get_memory_stats, reset_memory_stats, init_memory_pool};
    
    c.bench_function("memory_stats_collection", |b| {
        init_memory_pool().unwrap();
        b.iter(|| {
            reset_memory_stats();
            
            // Do some allocations
            for i in 0..1000 {
                let _ = PooledStorageOps::create_string(&format!("test{}", i));
            }
            
            // Get stats
            let stats = get_memory_stats();
            assert!(stats.allocations > 0);
            
            stats
        });
    });
}

criterion_group!(
    benches,
    benchmark_string_allocation,
    benchmark_list_operations,
    benchmark_hash_operations,
    benchmark_mixed_operations,
);

#[cfg(feature = "memory_pool")]
criterion_group!(
    memory_benches,
    benchmark_memory_stats,
);

#[cfg(not(feature = "memory_pool"))]
criterion_main!(benches);

#[cfg(feature = "memory_pool")]
criterion_main!(benches, memory_benches);