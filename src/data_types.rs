use serde::{Deserialize, Serialize, Deserializer, Serializer};
use std::collections::{HashMap, HashSet, BTreeMap};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum DataType {
    String(String),
    List(Vec<String>),
    Set(HashSet<String>),
    Hash(HashMap<String, String>),
    SortedSet(BTreeMap<String, f64>), // member -> score
    Json(serde_json::Value),
    Stream(Vec<StreamEntry>),
}

// Custom serialization to handle JSON values
impl Serialize for DataType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        enum DataTypeRepr {
            String(String),
            List(Vec<String>),
            Set(HashSet<String>),
            Hash(HashMap<String, String>),
            SortedSet(BTreeMap<String, f64>),
            Json(String), // Store JSON as string
            Stream(Vec<StreamEntry>),
        }
        
        let repr = match self {
            DataType::String(s) => DataTypeRepr::String(s.clone()),
            DataType::List(l) => DataTypeRepr::List(l.clone()),
            DataType::Set(s) => DataTypeRepr::Set(s.clone()),
            DataType::Hash(h) => DataTypeRepr::Hash(h.clone()),
            DataType::SortedSet(z) => DataTypeRepr::SortedSet(z.clone()),
            DataType::Json(j) => DataTypeRepr::Json(j.to_string()),
            DataType::Stream(s) => DataTypeRepr::Stream(s.clone()),
        };
        
        repr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DataType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum DataTypeRepr {
            String(String),
            List(Vec<String>),
            Set(HashSet<String>),
            Hash(HashMap<String, String>),
            SortedSet(BTreeMap<String, f64>),
            Json(String), // JSON stored as string
            Stream(Vec<StreamEntry>),
        }
        
        let repr = DataTypeRepr::deserialize(deserializer)?;
        
        Ok(match repr {
            DataTypeRepr::String(s) => DataType::String(s),
            DataTypeRepr::List(l) => DataType::List(l),
            DataTypeRepr::Set(s) => DataType::Set(s),
            DataTypeRepr::Hash(h) => DataType::Hash(h),
            DataTypeRepr::SortedSet(z) => DataType::SortedSet(z),
            DataTypeRepr::Json(j) => {
                let value = serde_json::from_str(&j)
                    .map_err(serde::de::Error::custom)?;
                DataType::Json(value)
            },
            DataTypeRepr::Stream(s) => DataType::Stream(s),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEntry {
    pub id: String,
    pub timestamp: SystemTime,
    pub fields: HashMap<String, String>,
}

impl DataType {
    pub fn type_name(&self) -> &'static str {
        match self {
            DataType::String(_) => "string",
            DataType::List(_) => "list",
            DataType::Set(_) => "set",
            DataType::Hash(_) => "hash",
            DataType::SortedSet(_) => "zset",
            DataType::Json(_) => "json",
            DataType::Stream(_) => "stream",
        }
    }
}

// String operations
impl DataType {
    pub fn as_string(&self) -> Option<&String> {
        match self {
            DataType::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        match self {
            DataType::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn incr(&mut self, delta: i64) -> Result<i64, String> {
        match self {
            DataType::String(s) => {
                let val: i64 = s.parse().map_err(|_| "Value is not an integer")?;
                let new_val = val + delta;
                *s = new_val.to_string();
                Ok(new_val)
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }
}

// List operations
impl DataType {
    pub fn as_list(&self) -> Option<&Vec<String>> {
        match self {
            DataType::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&mut Vec<String>> {
        match self {
            DataType::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn lpush(&mut self, values: Vec<String>) -> Result<usize, String> {
        match self {
            DataType::List(l) => {
                // Push values in the order they appear
                for v in values.into_iter() {
                    l.insert(0, v);
                }
                Ok(l.len())
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn rpush(&mut self, values: Vec<String>) -> Result<usize, String> {
        match self {
            DataType::List(l) => {
                l.extend(values);
                Ok(l.len())
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn lpop(&mut self) -> Result<Option<String>, String> {
        match self {
            DataType::List(l) => Ok(if l.is_empty() { None } else { Some(l.remove(0)) }),
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn rpop(&mut self) -> Result<Option<String>, String> {
        match self {
            DataType::List(l) => Ok(l.pop()),
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn lrange(&self, start: i64, stop: i64) -> Result<Vec<String>, String> {
        match self {
            DataType::List(l) => {
                let len = l.len() as i64;
                let start = if start < 0 { (len + start).max(0) } else { start } as usize;
                let stop = if stop < 0 { (len + stop + 1).max(0) } else { stop + 1 } as usize;
                let stop = stop.min(l.len());
                
                if start >= l.len() {
                    Ok(vec![])
                } else {
                    Ok(l[start..stop].to_vec())
                }
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }
}

// Set operations
impl DataType {
    pub fn as_set(&self) -> Option<&HashSet<String>> {
        match self {
            DataType::Set(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_set_mut(&mut self) -> Option<&mut HashSet<String>> {
        match self {
            DataType::Set(s) => Some(s),
            _ => None,
        }
    }

    pub fn sadd(&mut self, members: Vec<String>) -> Result<usize, String> {
        match self {
            DataType::Set(s) => {
                let mut added = 0;
                for member in members {
                    if s.insert(member) {
                        added += 1;
                    }
                }
                Ok(added)
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn srem(&mut self, members: Vec<String>) -> Result<usize, String> {
        match self {
            DataType::Set(s) => {
                let mut removed = 0;
                for member in members {
                    if s.remove(&member) {
                        removed += 1;
                    }
                }
                Ok(removed)
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn sismember(&self, member: &str) -> Result<bool, String> {
        match self {
            DataType::Set(s) => Ok(s.contains(member)),
            _ => Err("Operation not supported on this type".to_string()),
        }
    }
}

// Hash operations
impl DataType {
    pub fn as_hash(&self) -> Option<&HashMap<String, String>> {
        match self {
            DataType::Hash(h) => Some(h),
            _ => None,
        }
    }

    pub fn as_hash_mut(&mut self) -> Option<&mut HashMap<String, String>> {
        match self {
            DataType::Hash(h) => Some(h),
            _ => None,
        }
    }

    pub fn hset(&mut self, field: String, value: String) -> Result<bool, String> {
        match self {
            DataType::Hash(h) => {
                let is_new = !h.contains_key(&field);
                h.insert(field, value);
                Ok(is_new)
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn hget(&self, field: &str) -> Result<Option<String>, String> {
        match self {
            DataType::Hash(h) => Ok(h.get(field).cloned()),
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn hdel(&mut self, fields: Vec<String>) -> Result<usize, String> {
        match self {
            DataType::Hash(h) => {
                let mut deleted = 0;
                for field in fields {
                    if h.remove(&field).is_some() {
                        deleted += 1;
                    }
                }
                Ok(deleted)
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }
}

// Sorted Set operations
impl DataType {
    pub fn as_sorted_set(&self) -> Option<&BTreeMap<String, f64>> {
        match self {
            DataType::SortedSet(z) => Some(z),
            _ => None,
        }
    }

    pub fn as_sorted_set_mut(&mut self) -> Option<&mut BTreeMap<String, f64>> {
        match self {
            DataType::SortedSet(z) => Some(z),
            _ => None,
        }
    }

    pub fn zadd(&mut self, members: Vec<(f64, String)>) -> Result<usize, String> {
        match self {
            DataType::SortedSet(z) => {
                let mut added = 0;
                for (score, member) in members {
                    if !z.contains_key(&member) {
                        added += 1;
                    }
                    z.insert(member, score);
                }
                Ok(added)
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn zrem(&mut self, members: Vec<String>) -> Result<usize, String> {
        match self {
            DataType::SortedSet(z) => {
                let mut removed = 0;
                for member in members {
                    if z.remove(&member).is_some() {
                        removed += 1;
                    }
                }
                Ok(removed)
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn zscore(&self, member: &str) -> Result<Option<f64>, String> {
        match self {
            DataType::SortedSet(z) => Ok(z.get(member).copied()),
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn zrange(&self, start: i64, stop: i64, with_scores: bool) -> Result<Vec<(String, Option<f64>)>, String> {
        match self {
            DataType::SortedSet(z) => {
                let mut sorted: Vec<(&String, &f64)> = z.iter().collect();
                sorted.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
                
                let len = sorted.len() as i64;
                let start = if start < 0 { (len + start).max(0) } else { start } as usize;
                let stop = if stop < 0 { (len + stop + 1).max(0) } else { stop + 1 } as usize;
                let stop = stop.min(sorted.len());
                
                if start >= sorted.len() {
                    Ok(vec![])
                } else {
                    Ok(sorted[start..stop]
                        .iter()
                        .map(|(member, score)| {
                            ((*member).clone(), if with_scores { Some(**score) } else { None })
                        })
                        .collect())
                }
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }
}

// JSON operations
impl DataType {
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            DataType::Json(j) => Some(j),
            _ => None,
        }
    }

    pub fn as_json_mut(&mut self) -> Option<&mut serde_json::Value> {
        match self {
            DataType::Json(j) => Some(j),
            _ => None,
        }
    }

    pub fn json_set(&mut self, path: &str, value: serde_json::Value) -> Result<(), String> {
        match self {
            DataType::Json(j) => {
                if path == "$" || path == "." {
                    *j = value;
                    Ok(())
                } else {
                    // Simple path implementation - in production, use jsonpath library
                    Err("Complex JSON paths not yet implemented".to_string())
                }
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn json_get(&self, path: &str) -> Result<Option<serde_json::Value>, String> {
        match self {
            DataType::Json(j) => {
                if path == "$" || path == "." {
                    Ok(Some(j.clone()))
                } else {
                    // Simple path implementation - in production, use jsonpath library
                    Err("Complex JSON paths not yet implemented".to_string())
                }
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }
}

// Stream operations
impl DataType {
    pub fn as_stream(&self) -> Option<&Vec<StreamEntry>> {
        match self {
            DataType::Stream(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_stream_mut(&mut self) -> Option<&mut Vec<StreamEntry>> {
        match self {
            DataType::Stream(s) => Some(s),
            _ => None,
        }
    }

    pub fn xadd(&mut self, id: Option<String>, fields: HashMap<String, String>) -> Result<String, String> {
        match self {
            DataType::Stream(s) => {
                let id = id.unwrap_or_else(|| {
                    let timestamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    format!("{}-0", timestamp)
                });
                
                let entry = StreamEntry {
                    id: id.clone(),
                    timestamp: SystemTime::now(),
                    fields,
                };
                
                s.push(entry);
                Ok(id)
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn xrange(&self, start: &str, end: &str, count: Option<usize>) -> Result<Vec<StreamEntry>, String> {
        match self {
            DataType::Stream(s) => {
                let mut result: Vec<StreamEntry> = s.iter()
                    .filter(|entry| entry.id.as_str() >= start && entry.id.as_str() <= end)
                    .cloned()
                    .collect();
                
                if let Some(count) = count {
                    result.truncate(count);
                }
                
                Ok(result)
            }
            _ => Err("Operation not supported on this type".to_string()),
        }
    }

    pub fn xlen(&self) -> Result<usize, String> {
        match self {
            DataType::Stream(s) => Ok(s.len()),
            _ => Err("Operation not supported on this type".to_string()),
        }
    }
}