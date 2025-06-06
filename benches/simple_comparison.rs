use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Instant;

fn benchmark_parser_performance(c: &mut Criterion) {
    println!("\n=== Parser Performance Comparison ===\n");
    
    // Test different types of operations
    let test_cases = vec![
        ("small_allocation", 100),
        ("medium_allocation", 1000),
        ("large_allocation", 10000),
    ];
    
    let mut group = c.benchmark_group("allocation");
    
    for (name, size) in test_cases {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut vec = Vec::with_capacity(size);
                for i in 0..size {
                    vec.push(format!("test_string_{}", i));
                }
                black_box(vec);
            });
        });
    }
    
    group.finish();
}

fn benchmark_memory_operations(c: &mut Criterion) {
    println!("\n=== Memory Operations Comparison ===\n");
    
    let mut group = c.benchmark_group("memory");
    
    // Test string concatenation
    group.bench_function("string_concat", |b| {
        b.iter(|| {
            let mut s = String::new();
            for i in 0..100 {
                s.push_str(&format!("test_{}", i));
            }
            black_box(s);
        });
    });
    
    // Test vector operations
    group.bench_function("vector_push", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..1000 {
                vec.push(i);
            }
            black_box(vec);
        });
    });
    
    group.finish();
}

criterion_group!(benches, benchmark_parser_performance, benchmark_memory_operations);
criterion_main!(benches);