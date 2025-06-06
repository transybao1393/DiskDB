use crate::data_types::DataType;
use crate::error::Result;
use async_trait::async_trait;

pub mod rocksdb_storage;

#[async_trait]
pub trait Storage: Send + Sync {
    // Basic operations
    async fn get(&self, key: &str) -> Result<Option<DataType>>;
    async fn set(&self, key: &str, value: DataType) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<bool>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn get_type(&self, key: &str) -> Result<Option<String>>;
    
    // Batch operations
    async fn delete_multiple(&self, keys: &[String]) -> Result<usize>;
    async fn exists_multiple(&self, keys: &[String]) -> Result<usize>;
    
    // Type-safe get operations
    async fn get_string(&self, key: &str) -> Result<Option<String>> {
        match self.get(key).await? {
            Some(DataType::String(s)) => Ok(Some(s)),
            Some(_) => Err(crate::error::DiskDBError::Protocol("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
            None => Ok(None),
        }
    }
    
    async fn get_or_create_list(&self, key: &str) -> Result<DataType> {
        match self.get(key).await? {
            Some(data) => match data {
                DataType::List(_) => Ok(data),
                _ => Err(crate::error::DiskDBError::Protocol("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
            },
            None => Ok(DataType::List(Vec::new())),
        }
    }
    
    async fn get_or_create_set(&self, key: &str) -> Result<DataType> {
        match self.get(key).await? {
            Some(data) => match data {
                DataType::Set(_) => Ok(data),
                _ => Err(crate::error::DiskDBError::Protocol("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
            },
            None => Ok(DataType::Set(std::collections::HashSet::new())),
        }
    }
    
    async fn get_or_create_hash(&self, key: &str) -> Result<DataType> {
        match self.get(key).await? {
            Some(data) => match data {
                DataType::Hash(_) => Ok(data),
                _ => Err(crate::error::DiskDBError::Protocol("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
            },
            None => Ok(DataType::Hash(std::collections::HashMap::new())),
        }
    }
    
    async fn get_or_create_sorted_set(&self, key: &str) -> Result<DataType> {
        match self.get(key).await? {
            Some(data) => match data {
                DataType::SortedSet(_) => Ok(data),
                _ => Err(crate::error::DiskDBError::Protocol("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
            },
            None => Ok(DataType::SortedSet(std::collections::BTreeMap::new())),
        }
    }
    
    async fn get_or_create_stream(&self, key: &str) -> Result<DataType> {
        match self.get(key).await? {
            Some(data) => match data {
                DataType::Stream(_) => Ok(data),
                _ => Err(crate::error::DiskDBError::Protocol("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
            },
            None => Ok(DataType::Stream(Vec::new())),
        }
    }
}