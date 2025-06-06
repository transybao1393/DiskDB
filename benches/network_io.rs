use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, BatchSize};
use diskdb::{OptimizedServer, Server, Config, OptimizedClient};
use diskdb::storage::rocksdb_storage::RocksDBStorage;
use diskdb::protocol::{Request, Response};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn setup_server(optimized: bool, port: u16) -> (TempDir, Runtime) {
    let temp_dir = TempDir::new().unwrap();
    let storage = Arc::new(RocksDBStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
    
    let mut config = Config::default();
    config.server_port = port;
    config.use_tls = false;
    
    let rt = Runtime::new().unwrap();
    
    rt.spawn(async move {
        if optimized {
            let server = OptimizedServer::new(config, storage).unwrap();
            let _ = server.start().await;
        } else {
            let server = Server::new(config, storage).unwrap();
            let _ = server.start().await;
        }
    });
    
    // Give server time to start
    std::thread::sleep(Duration::from_millis(100));
    
    (temp_dir, rt)
}

fn benchmark_single_request(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_request");
    group.measurement_time(Duration::from_secs(10));
    
    // Standard server
    let (_temp1, rt1) = setup_server(false, 6379);
    group.bench_function("standard", |b| {
        b.iter_batched(
            || {
                rt1.block_on(async {
                    tokio::net::TcpStream::connect("127.0.0.1:6379").await.unwrap()
                })
            },
            |mut stream| {
                rt1.block_on(async {
                    use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
                    
                    stream.write_all(b"SET key1 value1\n").await.unwrap();
                    
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    reader.read_line(&mut line).await.unwrap();
                    
                    assert!(line.contains("OK"));
                })
            },
            BatchSize::SmallInput,
        );
    });
    
    // Optimized server
    let (_temp2, rt2) = setup_server(true, 6380);
    group.bench_function("optimized", |b| {
        b.iter_batched(
            || {
                rt2.block_on(async {
                    tokio::net::TcpStream::connect("127.0.0.1:6380").await.unwrap()
                })
            },
            |mut stream| {
                rt2.block_on(async {
                    use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
                    
                    stream.write_all(b"SET key1 value1\n").await.unwrap();
                    
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    reader.read_line(&mut line).await.unwrap();
                    
                    assert!(line.contains("OK"));
                })
            },
            BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

fn benchmark_pipeline_requests(c: &mut Criterion) {
    let mut group = c.benchmark_group("pipeline_requests");
    group.measurement_time(Duration::from_secs(10));
    
    let batch_sizes = vec![10, 50, 100];
    
    for batch_size in batch_sizes {
        // Standard server (no pipelining)
        let (_temp1, rt1) = setup_server(false, 6381);
        group.bench_with_input(
            BenchmarkId::new("standard", batch_size),
            &batch_size,
            |b, &size| {
                b.iter_batched(
                    || {
                        rt1.block_on(async {
                            tokio::net::TcpStream::connect("127.0.0.1:6381").await.unwrap()
                        })
                    },
                    |mut stream| {
                        rt1.block_on(async {
                            use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
                            
                            // Send multiple requests
                            for i in 0..size {
                                let cmd = format!("SET key{} value{}\n", i, i);
                                stream.write_all(cmd.as_bytes()).await.unwrap();
                            }
                            
                            // Read responses
                            let mut reader = BufReader::new(&mut stream);
                            for _ in 0..size {
                                let mut line = String::new();
                                reader.read_line(&mut line).await.unwrap();
                            }
                        })
                    },
                    BatchSize::SmallInput,
                );
            }
        );
        
        // Optimized server with pipelining
        let (_temp2, rt2) = setup_server(true, 6382);
        group.bench_with_input(
            BenchmarkId::new("optimized", batch_size),
            &batch_size,
            |b, &size| {
                b.iter_batched(
                    || {
                        rt2.block_on(async {
                            tokio::net::TcpStream::connect("127.0.0.1:6382").await.unwrap()
                        })
                    },
                    |mut stream| {
                        rt2.block_on(async {
                            use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
                            
                            // Send all requests at once (pipelined)
                            let mut batch = Vec::new();
                            for i in 0..size {
                                batch.extend_from_slice(format!("SET key{} value{}\n", i, i).as_bytes());
                            }
                            stream.write_all(&batch).await.unwrap();
                            
                            // Read all responses
                            let mut reader = BufReader::new(&mut stream);
                            for _ in 0..size {
                                let mut line = String::new();
                                reader.read_line(&mut line).await.unwrap();
                            }
                        })
                    },
                    BatchSize::SmallInput,
                );
            }
        );
    }
    
    group.finish();
}

fn benchmark_client_connection_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_pool");
    
    let (_temp, rt) = setup_server(true, 6383);
    
    // Without connection pool
    group.bench_function("no_pool", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut stream = tokio::net::TcpStream::connect("127.0.0.1:6383").await.unwrap();
                use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
                
                stream.write_all(b"PING\n").await.unwrap();
                
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                reader.read_line(&mut line).await.unwrap();
            })
        });
    });
    
    // With connection pool
    let client = rt.block_on(async {
        OptimizedClient::connect("127.0.0.1:6383").await.unwrap()
    });
    
    group.bench_function("with_pool", |b| {
        b.iter(|| {
            rt.block_on(async {
                client.ping().await.unwrap();
            })
        });
    });
    
    group.finish();
}

fn benchmark_large_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_response");
    group.measurement_time(Duration::from_secs(10));
    
    // Prepare large data
    let large_value = "x".repeat(1024 * 100); // 100KB
    
    // Standard server
    let (_temp1, rt1) = setup_server(false, 6384);
    rt1.block_on(async {
        let mut stream = tokio::net::TcpStream::connect("127.0.0.1:6384").await.unwrap();
        use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
        
        let cmd = format!("SET largekey {}\n", large_value);
        stream.write_all(cmd.as_bytes()).await.unwrap();
        
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
    });
    
    group.bench_function("standard", |b| {
        b.iter_batched(
            || {
                rt1.block_on(async {
                    tokio::net::TcpStream::connect("127.0.0.1:6384").await.unwrap()
                })
            },
            |mut stream| {
                rt1.block_on(async {
                    use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
                    
                    stream.write_all(b"GET largekey\n").await.unwrap();
                    
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    reader.read_line(&mut line).await.unwrap();
                })
            },
            BatchSize::SmallInput,
        );
    });
    
    // Optimized server with buffer pool
    let (_temp2, rt2) = setup_server(true, 6385);
    rt2.block_on(async {
        let mut stream = tokio::net::TcpStream::connect("127.0.0.1:6385").await.unwrap();
        use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
        
        let cmd = format!("SET largekey {}\n", large_value);
        stream.write_all(cmd.as_bytes()).await.unwrap();
        
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
    });
    
    group.bench_function("optimized", |b| {
        b.iter_batched(
            || {
                rt2.block_on(async {
                    tokio::net::TcpStream::connect("127.0.0.1:6385").await.unwrap()
                })
            },
            |mut stream| {
                rt2.block_on(async {
                    use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
                    
                    stream.write_all(b"GET largekey\n").await.unwrap();
                    
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    reader.read_line(&mut line).await.unwrap();
                })
            },
            BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_single_request,
    benchmark_pipeline_requests,
    benchmark_client_connection_pool,
    benchmark_large_response
);
criterion_main!(benches);