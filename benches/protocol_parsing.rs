use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use diskdb::protocol::Request;

fn bench_simple_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol/simple");
    
    // GET command
    group.bench_function("parse_get", |b| {
        b.iter(|| {
            Request::parse(black_box("GET mykey"))
        })
    });
    
    // SET command
    group.bench_function("parse_set", |b| {
        b.iter(|| {
            Request::parse(black_box("SET mykey myvalue"))
        })
    });
    
    // EXISTS with multiple keys
    group.bench_function("parse_exists_multi", |b| {
        b.iter(|| {
            Request::parse(black_box("EXISTS key1 key2 key3 key4 key5"))
        })
    });
    
    group.finish();
}

fn bench_complex_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol/complex");
    
    // ZADD with multiple members
    let zadd_5 = "ZADD myset 1 one 2 two 3 three 4 four 5 five";
    group.bench_function("parse_zadd_5_members", |b| {
        b.iter(|| {
            Request::parse(black_box(zadd_5))
        })
    });
    
    // ZADD with many members
    let zadd_50 = format!("ZADD myset {}", 
        (0..50).map(|i| format!("{} member{}", i, i)).collect::<Vec<_>>().join(" "));
    group.bench_function("parse_zadd_50_members", |b| {
        b.iter(|| {
            Request::parse(black_box(&zadd_50))
        })
    });
    
    // HSET with multiple fields
    let hset_multi = "HSET myhash field1 value1 field2 value2 field3 value3";
    group.bench_function("parse_hset_multi", |b| {
        b.iter(|| {
            Request::parse(black_box(hset_multi))
        })
    });
    
    group.finish();
}

fn bench_large_payloads(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol/large_payloads");
    
    // Different payload sizes
    let sizes = [10, 100, 1_000, 10_000];
    
    for size in &sizes {
        let value = "x".repeat(*size);
        let command = format!("SET mykey {}", value);
        
        group.bench_with_input(
            BenchmarkId::new("parse_set", format!("{}_bytes", size)),
            size,
            |b, _| {
                b.iter(|| {
                    Request::parse(black_box(&command))
                })
            }
        );
    }
    
    group.finish();
}

fn bench_json_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol/json");
    
    // Simple JSON
    let simple_json = r#"JSON.SET user $ {"name":"Alice","age":30}"#;
    group.bench_function("parse_json_simple", |b| {
        b.iter(|| {
            Request::parse(black_box(simple_json))
        })
    });
    
    // Complex nested JSON
    let complex_json = r#"JSON.SET profile $ {"user":{"name":"Bob","settings":{"theme":"dark","notifications":true},"scores":[100,95,87,92]}}"#;
    group.bench_function("parse_json_complex", |b| {
        b.iter(|| {
            Request::parse(black_box(complex_json))
        })
    });
    
    group.finish();
}

fn bench_allocation_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("protocol/allocations");
    
    // Measure allocations for common operations
    let commands = vec![
        ("simple_get", "GET key"),
        ("simple_set", "SET key value"),
        ("lpush_multi", "LPUSH mylist item1 item2 item3"),
        ("sadd_multi", "SADD myset member1 member2 member3 member4 member5"),
    ];
    
    for (name, cmd) in commands {
        group.bench_function(name, |b| {
            b.iter(|| {
                // This will help us identify allocation hotspots
                let _result = Request::parse(black_box(cmd));
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_simple_commands,
    bench_complex_commands,
    bench_large_payloads,
    bench_json_commands,
    bench_allocation_count
);
criterion_main!(benches);