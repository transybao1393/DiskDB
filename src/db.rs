use rocksdb::{DB, Options, WriteBatch};
use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration}; // Add Duration here
use tokio::time::sleep;
use log::{error, info};

/// Disk-based key-value store with caching, persistence, and batch writes.
#[derive(Clone)]
pub struct DiskDB {
    pub db: Arc<RwLock<DB>>, // RocksDB instance wrapped with a RwLock for better concurrency.
    pub cache: Arc<RwLock<HashMap<String, (String, Option<Instant>)>>>, // In-memory cache with optional expiration.
    pub memtable: Arc<RwLock<VecDeque<(String, String)>>>, // Memtable for batch writes.
}

impl DiskDB {
    /// Creates a new instance of the database, initializing RocksDB with optimizations.
    pub fn new(path: &str) -> Self {
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
    pub async fn flush_memtable(&self) {
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
    pub async fn set(&self, key: &str, value: &str, expire: Option<u64>) {
        let mut memtable = self.memtable.write().await;
        memtable.push_back((key.to_string(), value.to_string()));
        let mut cache = self.cache.write().await;
        let expiry = expire.map(|e| Instant::now() + Duration::from_secs(e));
        cache.insert(key.to_string(), (value.to_string(), expiry));
    }

    /// Retrieves a value from the cache or database. If expired, removes it.
    pub async fn get(&self, key: &str) -> Option<String> {
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