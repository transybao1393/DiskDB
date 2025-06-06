use crate::data_types::DataType;
use crate::error::Result;
use crate::protocol::{Request, Response};
use crate::storage::Storage;
use async_trait::async_trait;
use std::sync::Arc;

pub mod get;
pub mod set;

#[async_trait]
pub trait Command: Send + Sync {
    async fn execute(&self, storage: Arc<dyn Storage>) -> Result<Response>;
}

pub struct CommandExecutor {
    storage: Arc<dyn Storage>,
}

impl CommandExecutor {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    pub async fn execute(&self, request: Request) -> Result<Response> {
        match request {
            // String operations
            Request::Get { key } => {
                match self.storage.get(&key).await? {
                    Some(DataType::String(value)) => Ok(Response::Value(value)),
                    Some(_) => Ok(Response::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
                    None => Ok(Response::Null),
                }
            }
            Request::Set { key, value } => {
                self.storage.set(&key, DataType::String(value)).await?;
                Ok(Response::Ok)
            }
            Request::Incr { key } => {
                self.execute_incr(&key, 1).await
            }
            Request::Decr { key } => {
                self.execute_incr(&key, -1).await
            }
            Request::IncrBy { key, delta } => {
                self.execute_incr(&key, delta).await
            }
            Request::DecrBy { key, delta } => {
                self.execute_incr(&key, -delta).await
            }
            Request::Append { key, value } => {
                let result = match self.storage.get(&key).await? {
                    Some(DataType::String(mut s)) => {
                        s.push_str(&value);
                        let len = s.len();
                        self.storage.set(&key, DataType::String(s)).await?;
                        len
                    }
                    None => {
                        let len = value.len();
                        self.storage.set(&key, DataType::String(value)).await?;
                        len
                    }
                    Some(_) => return Ok(Response::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
                };
                Ok(Response::Integer(result as i64))
            }
            
            // List operations
            Request::LPush { key, values } => {
                let mut data = self.storage.get_or_create_list(&key).await?;
                let count = data.lpush(values).map_err(crate::error::DiskDBError::Database)?;
                self.storage.set(&key, data).await?;
                Ok(Response::Integer(count as i64))
            }
            Request::RPush { key, values } => {
                let mut data = self.storage.get_or_create_list(&key).await?;
                let count = data.rpush(values).map_err(crate::error::DiskDBError::Database)?;
                self.storage.set(&key, data).await?;
                Ok(Response::Integer(count as i64))
            }
            Request::LPop { key } => {
                match self.storage.get(&key).await? {
                    Some(mut data) => match data.lpop() {
                        Ok(Some(value)) => {
                            if data.as_list().map(|l| l.is_empty()).unwrap_or(false) {
                                self.storage.delete(&key).await?;
                            } else {
                                self.storage.set(&key, data).await?;
                            }
                            Ok(Response::Value(value))
                        }
                        Ok(None) => Ok(Response::Null),
                        Err(e) => Ok(Response::Error(e)),
                    },
                    None => Ok(Response::Null),
                }
            }
            Request::RPop { key } => {
                match self.storage.get(&key).await? {
                    Some(mut data) => match data.rpop() {
                        Ok(Some(value)) => {
                            if data.as_list().map(|l| l.is_empty()).unwrap_or(false) {
                                self.storage.delete(&key).await?;
                            } else {
                                self.storage.set(&key, data).await?;
                            }
                            Ok(Response::Value(value))
                        }
                        Ok(None) => Ok(Response::Null),
                        Err(e) => Ok(Response::Error(e)),
                    },
                    None => Ok(Response::Null),
                }
            }
            Request::LRange { key, start, stop } => {
                match self.storage.get(&key).await? {
                    Some(data) => match data.lrange(start, stop) {
                        Ok(values) => Ok(Response::Array(values)),
                        Err(e) => Ok(Response::Error(e)),
                    },
                    None => Ok(Response::Array(vec![])),
                }
            }
            Request::LLen { key } => {
                match self.storage.get(&key).await? {
                    Some(DataType::List(list)) => Ok(Response::Integer(list.len() as i64)),
                    Some(_) => Ok(Response::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
                    None => Ok(Response::Integer(0)),
                }
            }
            
            // Set operations
            Request::SAdd { key, members } => {
                let mut data = self.storage.get_or_create_set(&key).await?;
                let added = data.sadd(members).map_err(crate::error::DiskDBError::Database)?;
                self.storage.set(&key, data).await?;
                Ok(Response::Integer(added as i64))
            }
            Request::SRem { key, members } => {
                match self.storage.get(&key).await? {
                    Some(mut data) => {
                        let removed = data.srem(members).map_err(crate::error::DiskDBError::Database)?;
                        if data.as_set().map(|s| s.is_empty()).unwrap_or(false) {
                            self.storage.delete(&key).await?;
                        } else {
                            self.storage.set(&key, data).await?;
                        }
                        Ok(Response::Integer(removed as i64))
                    }
                    None => Ok(Response::Integer(0)),
                }
            }
            Request::SMembers { key } => {
                match self.storage.get(&key).await? {
                    Some(DataType::Set(set)) => {
                        let members: Vec<String> = set.into_iter().collect();
                        Ok(Response::Array(members))
                    }
                    Some(_) => Ok(Response::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
                    None => Ok(Response::Array(vec![])),
                }
            }
            Request::SIsMember { key, member } => {
                match self.storage.get(&key).await? {
                    Some(data) => match data.sismember(&member) {
                        Ok(is_member) => Ok(Response::Integer(if is_member { 1 } else { 0 })),
                        Err(e) => Ok(Response::Error(e)),
                    },
                    None => Ok(Response::Integer(0)),
                }
            }
            Request::SCard { key } => {
                match self.storage.get(&key).await? {
                    Some(DataType::Set(set)) => Ok(Response::Integer(set.len() as i64)),
                    Some(_) => Ok(Response::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
                    None => Ok(Response::Integer(0)),
                }
            }
            
            // Hash operations
            Request::HSet { key, field, value } => {
                let mut data = self.storage.get_or_create_hash(&key).await?;
                let is_new = data.hset(field, value).map_err(crate::error::DiskDBError::Database)?;
                self.storage.set(&key, data).await?;
                Ok(Response::Integer(if is_new { 1 } else { 0 }))
            }
            Request::HGet { key, field } => {
                match self.storage.get(&key).await? {
                    Some(data) => match data.hget(&field) {
                        Ok(Some(value)) => Ok(Response::Value(value)),
                        Ok(None) => Ok(Response::Null),
                        Err(e) => Ok(Response::Error(e)),
                    },
                    None => Ok(Response::Null),
                }
            }
            Request::HDel { key, fields } => {
                match self.storage.get(&key).await? {
                    Some(mut data) => {
                        let deleted = data.hdel(fields).map_err(crate::error::DiskDBError::Database)?;
                        if data.as_hash().map(|h| h.is_empty()).unwrap_or(false) {
                            self.storage.delete(&key).await?;
                        } else {
                            self.storage.set(&key, data).await?;
                        }
                        Ok(Response::Integer(deleted as i64))
                    }
                    None => Ok(Response::Integer(0)),
                }
            }
            Request::HGetAll { key } => {
                match self.storage.get(&key).await? {
                    Some(DataType::Hash(hash)) => {
                        let mut result = Vec::new();
                        for (field, value) in hash {
                            result.push(field);
                            result.push(value);
                        }
                        Ok(Response::Array(result))
                    }
                    Some(_) => Ok(Response::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
                    None => Ok(Response::Array(vec![])),
                }
            }
            Request::HExists { key, field } => {
                match self.storage.get(&key).await? {
                    Some(DataType::Hash(hash)) => {
                        Ok(Response::Integer(if hash.contains_key(&field) { 1 } else { 0 }))
                    }
                    Some(_) => Ok(Response::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
                    None => Ok(Response::Integer(0)),
                }
            }
            
            // Sorted Set operations
            Request::ZAdd { key, members } => {
                let mut data = self.storage.get_or_create_sorted_set(&key).await?;
                let added = data.zadd(members).map_err(crate::error::DiskDBError::Database)?;
                self.storage.set(&key, data).await?;
                Ok(Response::Integer(added as i64))
            }
            Request::ZRem { key, members } => {
                match self.storage.get(&key).await? {
                    Some(mut data) => {
                        let removed = data.zrem(members).map_err(crate::error::DiskDBError::Database)?;
                        if data.as_sorted_set().map(|z| z.is_empty()).unwrap_or(false) {
                            self.storage.delete(&key).await?;
                        } else {
                            self.storage.set(&key, data).await?;
                        }
                        Ok(Response::Integer(removed as i64))
                    }
                    None => Ok(Response::Integer(0)),
                }
            }
            Request::ZRange { key, start, stop, with_scores } => {
                match self.storage.get(&key).await? {
                    Some(data) => match data.zrange(start, stop, with_scores) {
                        Ok(members) => {
                            let result: Vec<String> = members.into_iter()
                                .flat_map(|(member, score)| {
                                    if let Some(s) = score {
                                        vec![member, s.to_string()]
                                    } else {
                                        vec![member]
                                    }
                                })
                                .collect();
                            Ok(Response::Array(result))
                        }
                        Err(e) => Ok(Response::Error(e)),
                    },
                    None => Ok(Response::Array(vec![])),
                }
            }
            Request::ZScore { key, member } => {
                match self.storage.get(&key).await? {
                    Some(data) => match data.zscore(&member) {
                        Ok(Some(score)) => Ok(Response::Value(score.to_string())),
                        Ok(None) => Ok(Response::Null),
                        Err(e) => Ok(Response::Error(e)),
                    },
                    None => Ok(Response::Null),
                }
            }
            Request::ZCard { key } => {
                match self.storage.get(&key).await? {
                    Some(DataType::SortedSet(zset)) => Ok(Response::Integer(zset.len() as i64)),
                    Some(_) => Ok(Response::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
                    None => Ok(Response::Integer(0)),
                }
            }
            
            // JSON operations
            Request::JsonSet { key, path, value } => {
                let json_value: serde_json::Value = serde_json::from_str(&value)
                    .map_err(|e| crate::error::DiskDBError::Protocol(format!("Invalid JSON: {}", e)))?;
                
                let mut data = match self.storage.get(&key).await? {
                    Some(DataType::Json(j)) => DataType::Json(j),
                    Some(_) => return Ok(Response::Error("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())),
                    None => DataType::Json(serde_json::Value::Null),
                };
                
                data.json_set(&path, json_value).map_err(crate::error::DiskDBError::Database)?;
                self.storage.set(&key, data).await?;
                Ok(Response::Ok)
            }
            Request::JsonGet { key, path } => {
                match self.storage.get(&key).await? {
                    Some(data) => match data.json_get(&path) {
                        Ok(Some(value)) => Ok(Response::Value(value.to_string())),
                        Ok(None) => Ok(Response::Null),
                        Err(e) => Ok(Response::Error(e)),
                    },
                    None => Ok(Response::Null),
                }
            }
            Request::JsonDel { key, path } => {
                if path == "$" || path == "." {
                    match self.storage.delete(&key).await? {
                        true => Ok(Response::Integer(1)),
                        false => Ok(Response::Integer(0)),
                    }
                } else {
                    Ok(Response::Error("Complex JSON paths not yet implemented".to_string()))
                }
            }
            
            // Stream operations
            Request::XAdd { key, id, fields } => {
                let mut data = self.storage.get_or_create_stream(&key).await?;
                match data.xadd(id, fields) {
                    Ok(entry_id) => {
                        self.storage.set(&key, data).await?;
                        Ok(Response::Value(entry_id))
                    }
                    Err(e) => Ok(Response::Error(e)),
                }
            }
            Request::XRange { key, start, end, count } => {
                match self.storage.get(&key).await? {
                    Some(data) => match data.xrange(&start, &end, count) {
                        Ok(entries) => {
                            let mut result = Vec::new();
                            for entry in entries {
                                result.push(entry.id.clone());
                                for (field, value) in entry.fields {
                                    result.push(field);
                                    result.push(value);
                                }
                            }
                            Ok(Response::Array(result))
                        }
                        Err(e) => Ok(Response::Error(e)),
                    },
                    None => Ok(Response::Array(vec![])),
                }
            }
            Request::XLen { key } => {
                match self.storage.get(&key).await? {
                    Some(data) => match data.xlen() {
                        Ok(len) => Ok(Response::Integer(len as i64)),
                        Err(e) => Ok(Response::Error(e)),
                    },
                    None => Ok(Response::Integer(0)),
                }
            }
            
            // Utility operations
            Request::Type { key } => {
                match self.storage.get_type(&key).await? {
                    Some(type_name) => Ok(Response::Value(type_name)),
                    None => Ok(Response::Value("none".to_string())),
                }
            }
            Request::Del { keys } => {
                let deleted = self.storage.delete_multiple(&keys).await?;
                Ok(Response::Integer(deleted as i64))
            }
            Request::Exists { keys } => {
                let count = self.storage.exists_multiple(&keys).await?;
                Ok(Response::Integer(count as i64))
            }
        }
    }
    
    async fn execute_incr(&self, key: &str, delta: i64) -> Result<Response> {
        let result = match self.storage.get(key).await? {
            Some(mut data) => {
                let new_val = data.incr(delta).map_err(crate::error::DiskDBError::Database)?;
                self.storage.set(key, data).await?;
                new_val
            }
            None => {
                let data = DataType::String(delta.to_string());
                self.storage.set(key, data).await?;
                delta
            }
        };
        Ok(Response::Integer(result))
    }
}