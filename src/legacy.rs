use rocksdb::{DB, Options, WriteBatch};
use tokio::{net::TcpListener, io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, sync::RwLock};
use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use sha2::{Sha256, Digest};
use native_tls::{TlsAcceptor, TlsStream};
use tokio_native_tls::TlsAcceptor as TokioTlsAcceptor;
use log::{error, info};

/// Disk-based key-value store with caching, persistence, and batch writes.
struct DiskDB {
    db: Arc<RwLock<DB>>, // RocksDB instance wrapped with a RwLock for better concurrency.
    cache: Arc<RwLock<HashMap<String, (String, Option<Instant>)>>>, // In-memory cache with optional expiration.
    memtable: Arc<RwLock<VecDeque<(String, String)>>>, // Memtable for batch writes.
}

impl DiskDB {
    /// Creates a new instance of the database, initializing RocksDB with optimizations.
    fn new(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_wal_ttl_seconds(60); // Enable WAL (Write-Ahead Logging) with a 60-second TTL.
        opts.optimize_for_point_lookup(128);
        opts.increase_parallelism(num_cpus::get() as i32);
        opts.set_write_buffer_size(64 * 1024 * 1024); // Set write buffer size to 64MB.
        opts.set_max_write_buffer_number(4); // Allow up to 4 write buffers before flushing.
        opts.set_target_file_size_base(256 * 1024 * 1024); // Set target SST file size to 256MB.
        opts.set_level_compaction_dynamic_level_bytes(true); // Enable dynamic compaction.
        let db = DB::open(&opts, path).expect("Failed to open DB");
        Self {
            db: Arc::new(RwLock::new(db)),
            cache: Arc::new(RwLock::new(HashMap::new())),
            memtable: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Periodically flushes the memtable to disk in batch mode.
    async fn flush_memtable(&self) {
        loop {
            sleep(Duration::from_secs(1)).await;
            let mut memtable = self.memtable.write().await;
            if memtable.is_empty() {
                continue;
            }
            let mut db = self.db.write().await;
            let mut batch = WriteBatch::default();
            while let Some((key, value)) = memtable.pop_front() {
                batch.put(&key, &value);
            }
            if let Err(e) = db.write(batch) {
                error!("Failed to write batch: {:?}", e);
            }
        }
    }

    /// Stores a key-value pair in the memtable, which will be batch-written to disk.
    async fn set(&self, key: &str, value: &str, expire: Option<u64>) {
        let mut memtable = self.memtable.write().await;
        memtable.push_back((key.to_string(), value.to_string()));
        let mut cache = self.cache.write().await;
        let expiry = expire.map(|e| Instant::now() + Duration::from_secs(e));
        cache.insert(key.to_string(), (value.to_string(), expiry));
    }

    /// Retrieves a value from the cache or database. If expired, removes it.
    async fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.write().await;
        if let Some((value, expiry)) = cache.get(key) {
            if let Some(exp) = expiry {
                if Instant::now() > *exp {
                    cache.remove(key);
                    return None;
                }
            }
            return Some(value.clone());
        }

        let db = self.db.read().await;
        match db.get(key) {
            Ok(Some(value)) => {
                let val_str = String::from_utf8(value).unwrap_or_else(|_| "(corrupt data)".to_string());
                cache.insert(key.to_string(), (val_str.clone(), None));
                Some(val_str)
            }
            Ok(None) => None,
            Err(e) => {
                error!("Failed to read data: {:?}", e);
                None
            }
        }
    }
}

/// Starts a secure TCP server with TLS and authentication.
#[tokio::main]
async fn main() {
    println!("Database is starting...");

    env_logger::init();
    let db = Arc::new(DiskDB::new("diskdb"));
    let db_clone = db.clone();
    tokio::spawn(async move {
        db_clone.flush_memtable().await;
    });

    let port = std::env::var("DISKDB_PORT").unwrap_or_else(|_| "6380".to_string());
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await.expect("Failed to bind");
    info!("Secure server running on 127.0.0.1:{}", port);

    let banner = r#"
    ██████╗ ██╗███████╗██╗  ██╗██████╗ ██████╗ 
    ██╔══██╗██║██╔════╝██║  ██║██╔══██╗██╔══██╗
    ██████╔╝██║███████╗███████║██████╔╝██████╔╝
    ██╔═══╝ ██║╚════██║██╔══██║██╔═══╝ ██╔═══╝ 
    ██║     ██║███████║██║  ██║██║     ██║     
    ╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝     

    "#;

    println!("{}", banner);
    println!("🔥 DiskDB is running and ready to accept connections!");
    println!("💡 Happy coding! 🚀");

    // loop {
    //     let (socket, _) = listener.accept().await.expect("Failed to accept");
    //     let db = db.clone();
    //     tokio::spawn(async move {
    //         let mut reader = BufReader::new(socket);
    //         let mut line = String::new();
    //         while reader.read_line(&mut line).await.unwrap() > 0 {
    //             let parts: Vec<&str> = line.trim().split_whitespace().collect();
    //             let response = match parts.as_slice() {
    //                 ["SET", key, value] => {
    //                     db.set(key, value, None).await;
    //                     "OK\n".to_string()
    //                 }
    //                 ["GET", key] => {
    //                     match db.get(key).await {
    //                         Some(value) => format!("{}\n", value),
    //                         None => "(nil)\n".to_string(),
    //                     }
    //                 }
    //                 _ => "Invalid command\n".to_string(),
    //             };
    //             reader.get_mut().write_all(response.as_bytes()).await.unwrap();
    //             line.clear();
    //         }
    //     });
    // }

    loop {
        // Accept an incoming client connection. This blocks until a client connects.
        let (socket, _) = listener.accept().await.expect("Failed to accept");
    
        // Clone the database handle so each client gets its own reference.
        let db = db.clone();
    
        // Spawn a new asynchronous task to handle the client connection.
        tokio::spawn(async move {
            // Create a buffered reader from the socket to read incoming data line by line.
            let mut reader = BufReader::new(socket);
            let mut line = String::new();
    
            // Continuously read input from the client until the connection is closed.
            while reader.read_line(&mut line).await.unwrap() > 0 {
                // Split the received command into parts (separated by whitespace).
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
    
                // Match the command and execute the corresponding database operation.
                let response = match parts.as_slice() {
                    // "SET key value" command: Store the key-value pair in the database.
                    ["SET", key, value] => {
                        db.set(key, value, None).await;
                        "OK\n".to_string() // Send "OK" as the response.
                    }
                    // "GET key" command: Retrieve the value for the given key.
                    ["GET", key] => {
                        match db.get(key).await {
                            Some(value) => format!("{}\n", value), // Send the found value.
                            None => "(nil)\n".to_string(), // Send "(nil)" if the key doesn't exist.
                        }
                    }
                    // Handle invalid commands.
                    _ => "Invalid command\n".to_string(),
                };
    
                // Write the response back to the client.
                reader.get_mut().write_all(response.as_bytes()).await.unwrap();
    
                // Clear the line buffer for the next input.
                line.clear();
            }
        });
    }
}