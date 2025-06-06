use crate::data_types::{DataType, StreamEntry};
use crate::error::Result;
use std::collections::{HashMap, HashSet, BTreeMap};

#[cfg(feature = "memory_pool")]
use crate::ffi::memory::{PooledString, PooledVec, PooledBox, init_memory_pool};

/// Creates a pooled string from a regular string
#[cfg(feature = "memory_pool")]
pub fn to_pooled_string(s: &str) -> Result<PooledString> {
    PooledString::from_str(s)
}

#[cfg(not(feature = "memory_pool"))]
pub fn to_pooled_string(s: &str) -> Result<String> {
    Ok(s.to_string())
}

/// Pooled version of DataType
#[cfg(feature = "memory_pool")]
#[derive(Debug, Clone)]
pub enum PooledDataType {
    String(PooledString),
    List(PooledVec<PooledString>),
    Set(HashSet<PooledString>),
    Hash(HashMap<PooledString, PooledString>),
    SortedSet(BTreeMap<PooledString, f64>),
    Json(PooledBox<serde_json::Value>),
    Stream(PooledVec<PooledStreamEntry>),
}

#[cfg(feature = "memory_pool")]
#[derive(Debug, Clone)]
pub struct PooledStreamEntry {
    pub id: PooledString,
    pub timestamp: std::time::SystemTime,
    pub fields: HashMap<PooledString, PooledString>,
}

/// Convert regular DataType to PooledDataType
#[cfg(feature = "memory_pool")]
impl PooledDataType {
    pub fn from_data_type(data: DataType) -> Result<Self> {
        init_memory_pool()?;
        
        match data {
            DataType::String(s) => {
                Ok(PooledDataType::String(PooledString::from_str(&s)?))
            }
            DataType::List(list) => {
                let mut pooled_list = PooledVec::with_capacity(list.len())?;
                for item in list {
                    pooled_list.push(PooledString::from_str(&item)?)?;
                }
                Ok(PooledDataType::List(pooled_list))
            }
            DataType::Set(set) => {
                let mut pooled_set = HashSet::new();
                for item in set {
                    pooled_set.insert(PooledString::from_str(&item)?);
                }
                Ok(PooledDataType::Set(pooled_set))
            }
            DataType::Hash(hash) => {
                let mut pooled_hash = HashMap::new();
                for (k, v) in hash {
                    pooled_hash.insert(
                        PooledString::from_str(&k)?,
                        PooledString::from_str(&v)?
                    );
                }
                Ok(PooledDataType::Hash(pooled_hash))
            }
            DataType::SortedSet(zset) => {
                let mut pooled_zset = BTreeMap::new();
                for (member, score) in zset {
                    pooled_zset.insert(PooledString::from_str(&member)?, score);
                }
                Ok(PooledDataType::SortedSet(pooled_zset))
            }
            DataType::Json(json) => {
                Ok(PooledDataType::Json(PooledBox::new(json)?))
            }
            DataType::Stream(stream) => {
                let mut pooled_stream = PooledVec::with_capacity(stream.len())?;
                for entry in stream {
                    let mut pooled_fields = HashMap::new();
                    for (k, v) in entry.fields {
                        pooled_fields.insert(
                            PooledString::from_str(&k)?,
                            PooledString::from_str(&v)?
                        );
                    }
                    pooled_stream.push(PooledStreamEntry {
                        id: PooledString::from_str(&entry.id)?,
                        timestamp: entry.timestamp,
                        fields: pooled_fields,
                    })?;
                }
                Ok(PooledDataType::Stream(pooled_stream))
            }
        }
    }
    
    /// Convert PooledDataType back to regular DataType
    pub fn to_data_type(self) -> DataType {
        match self {
            PooledDataType::String(s) => DataType::String(s.to_string()),
            PooledDataType::List(list) => {
                let mut regular_list = Vec::new();
                for item in list.as_slice() {
                    regular_list.push(item.to_string());
                }
                DataType::List(regular_list)
            }
            PooledDataType::Set(set) => {
                let mut regular_set = HashSet::new();
                for item in set {
                    regular_set.insert(item.to_string());
                }
                DataType::Set(regular_set)
            }
            PooledDataType::Hash(hash) => {
                let mut regular_hash = HashMap::new();
                for (k, v) in hash {
                    regular_hash.insert(k.to_string(), v.to_string());
                }
                DataType::Hash(regular_hash)
            }
            PooledDataType::SortedSet(zset) => {
                let mut regular_zset = BTreeMap::new();
                for (member, score) in zset {
                    regular_zset.insert(member.to_string(), score);
                }
                DataType::SortedSet(regular_zset)
            }
            PooledDataType::Json(json) => {
                DataType::Json((*json).clone())
            }
            PooledDataType::Stream(stream) => {
                let mut regular_stream = Vec::new();
                for entry in stream.as_slice() {
                    let mut regular_fields = HashMap::new();
                    for (k, v) in &entry.fields {
                        regular_fields.insert(k.to_string(), v.to_string());
                    }
                    regular_stream.push(StreamEntry {
                        id: entry.id.to_string(),
                        timestamp: entry.timestamp,
                        fields: regular_fields,
                    });
                }
                DataType::Stream(regular_stream)
            }
        }
    }
}

/// Storage operations that use memory pools when available
pub struct PooledStorageOps;

impl PooledStorageOps {
    /// Optimized string allocation
    #[cfg(feature = "memory_pool")]
    pub fn create_string(s: &str) -> Result<DataType> {
        let pooled = PooledString::from_str(s)?;
        Ok(DataType::String(pooled.to_string()))
    }
    
    #[cfg(not(feature = "memory_pool"))]
    pub fn create_string(s: &str) -> Result<DataType> {
        Ok(DataType::String(s.to_string()))
    }
    
    /// Optimized list creation
    #[cfg(feature = "memory_pool")]
    pub fn create_list(capacity: usize) -> Result<DataType> {
        let _ = PooledVec::<PooledString>::with_capacity(capacity)?;
        Ok(DataType::List(Vec::with_capacity(capacity)))
    }
    
    #[cfg(not(feature = "memory_pool"))]
    pub fn create_list(capacity: usize) -> Result<DataType> {
        Ok(DataType::List(Vec::with_capacity(capacity)))
    }
}