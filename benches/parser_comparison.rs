use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use diskdb::protocol::Request;

#[cfg(feature = "c_parser")]
use diskdb::ffi::parser::parse_request_fast;

fn bench_parser_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_comparison");
    
    // Test cases
    let test_cases = vec![
        ("simple_get", "GET mykey"),
        ("simple_set", "SET mykey myvalue"),
        ("complex_zadd", "ZADD myset 1 one 2 two 3 three 4 four 5 five"),
        ("lpush_multi", "LPUSH mylist item1 item2 item3 item4 item5"),
        ("large_set", &format!("SET mykey {}", "x".repeat(1000))),
    ];
    
    for (name, cmd) in test_cases {
        // Benchmark Rust parser
        group.bench_with_input(
            BenchmarkId::new("rust_parser", name),
            &cmd,
            |b, &cmd| {
                b.iter(|| {
                    Request::parse_rust(black_box(cmd)).unwrap()
                })
            }
        );
        
        // Benchmark C parser if available
        #[cfg(feature = "c_parser")]
        group.bench_with_input(
            BenchmarkId::new("c_parser", name),
            &cmd,
            |b, &cmd| {
                b.iter(|| {
                    parse_request_fast(black_box(cmd)).unwrap()
                })
            }
        );
    }
    
    group.finish();
}

fn bench_allocation_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_allocations");
    
    // Complex command that causes many allocations in Rust parser
    let complex_cmd = "ZADD leaderboard 100 alice 95 bob 90 charlie 85 david 80 eve 75 frank 70 grace 65 henry 60 iris 55 jack";
    
    group.bench_function("rust_parser_complex", |b| {
        b.iter(|| {
            Request::parse_rust(black_box(complex_cmd)).unwrap()
        })
    });
    
    #[cfg(feature = "c_parser")]
    group.bench_function("c_parser_complex", |b| {
        b.iter(|| {
            parse_request_fast(black_box(complex_cmd)).unwrap()
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_parser_comparison, bench_allocation_count);
criterion_main!(benches);