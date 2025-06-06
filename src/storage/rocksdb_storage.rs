use crate::data_types::DataType;
use crate::error::{DiskDBError, Result};
use crate::storage::Storage;
use async_trait::async_trait;
use rocksdb::{DB, Options, WriteBatch};
use std::sync::Arc;
use std::path::Path;

pub struct RocksDBStorage {
    db: Arc<DB>,
}

impl RocksDBStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        
        // Clean up existing database for tests
        let path_ref = path.as_ref();
        if path_ref.exists() && path_ref.to_string_lossy().contains("test_db") {
            std::fs::remove_dir_all(path_ref).ok();
        }
        
        let db = DB::open(&opts, path)?;
        
        Ok(Self {
            db: Arc::new(db),
        })
    }
}

#[async_trait]
impl Storage for RocksDBStorage {
    async fn get(&self, key: &str) -> Result<Option<DataType>> {
        match self.db.get(key.as_bytes())? {
            Some(value) => {
                let data: DataType = bincode::deserialize(&value)
                    .map_err(|e| DiskDBError::Database(format!("Deserialization error: {}", e)))?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    async fn set(&self, key: &str, value: DataType) -> Result<()> {
        let serialized = bincode::serialize(&value)
            .map_err(|e| DiskDBError::Database(format!("Serialization error: {}", e)))?;
        self.db.put(key.as_bytes(), serialized)?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let exists = self.exists(key).await?;
        if exists {
            self.db.delete(key.as_bytes())?;
        }
        Ok(exists)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.db.get(key.as_bytes())?.is_some())
    }

    async fn get_type(&self, key: &str) -> Result<Option<String>> {
        match self.get(key).await? {
            Some(data) => Ok(Some(data.type_name().to_string())),
            None => Ok(None),
        }
    }
    
    async fn delete_multiple(&self, keys: &[String]) -> Result<usize> {
        let mut batch = WriteBatch::default();
        let mut deleted = 0;
        
        for key in keys {
            if self.exists(key).await? {
                batch.delete(key.as_bytes());
                deleted += 1;
            }
        }
        
        if deleted > 0 {
            self.db.write(batch)?;
        }
        
        Ok(deleted)
    }
    
    async fn exists_multiple(&self, keys: &[String]) -> Result<usize> {
        let mut count = 0;
        for key in keys {
            if self.exists(key).await? {
                count += 1;
            }
        }
        Ok(count)
    }
}