use crate::error::{DiskDBError, Result};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Request {
    // String operations
    Get { key: String },
    Set { key: String, value: String },
    Incr { key: String },
    Decr { key: String },
    IncrBy { key: String, delta: i64 },
    DecrBy { key: String, delta: i64 },
    Append { key: String, value: String },
    
    // List operations
    LPush { key: String, values: Vec<String> },
    RPush { key: String, values: Vec<String> },
    LPop { key: String },
    RPop { key: String },
    LRange { key: String, start: i64, stop: i64 },
    LLen { key: String },
    
    // Set operations
    SAdd { key: String, members: Vec<String> },
    SRem { key: String, members: Vec<String> },
    SMembers { key: String },
    SIsMember { key: String, member: String },
    SCard { key: String },
    
    // Hash operations
    HSet { key: String, field: String, value: String },
    HGet { key: String, field: String },
    HDel { key: String, fields: Vec<String> },
    HGetAll { key: String },
    HExists { key: String, field: String },
    
    // Sorted Set operations
    ZAdd { key: String, members: Vec<(f64, String)> },
    ZRem { key: String, members: Vec<String> },
    ZRange { key: String, start: i64, stop: i64, with_scores: bool },
    ZScore { key: String, member: String },
    ZCard { key: String },
    
    // JSON operations
    JsonSet { key: String, path: String, value: String },
    JsonGet { key: String, path: String },
    JsonDel { key: String, path: String },
    
    // Stream operations
    XAdd { key: String, id: Option<String>, fields: HashMap<String, String> },
    XRange { key: String, start: String, end: String, count: Option<usize> },
    XLen { key: String },
    
    // Utility operations
    Type { key: String },
    Del { keys: Vec<String> },
    Exists { keys: Vec<String> },
}

#[derive(Debug)]
pub enum Response {
    Ok,
    Value(String),
    Integer(i64),
    Array(Vec<String>),
    Null,
    Error(String),
}

impl Request {
    pub fn parse(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return Err(DiskDBError::Protocol("Empty command".to_string()));
        }
        
        match parts[0].to_uppercase().as_str() {
            // String operations
            "GET" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("GET requires exactly one argument".to_string()));
                }
                Ok(Request::Get { key: parts[1].to_string() })
            }
            "SET" => {
                if parts.len() < 3 {
                    return Err(DiskDBError::Protocol("SET requires at least two arguments".to_string()));
                }
                let value = parts[2..].join(" ");
                Ok(Request::Set { 
                    key: parts[1].to_string(), 
                    value 
                })
            }
            "INCR" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("INCR requires exactly one argument".to_string()));
                }
                Ok(Request::Incr { key: parts[1].to_string() })
            }
            "DECR" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("DECR requires exactly one argument".to_string()));
                }
                Ok(Request::Decr { key: parts[1].to_string() })
            }
            "INCRBY" => {
                if parts.len() != 3 {
                    return Err(DiskDBError::Protocol("INCRBY requires exactly two arguments".to_string()));
                }
                let delta = parts[2].parse::<i64>()
                    .map_err(|_| DiskDBError::Protocol("Invalid integer".to_string()))?;
                Ok(Request::IncrBy { key: parts[1].to_string(), delta })
            }
            "APPEND" => {
                if parts.len() < 3 {
                    return Err(DiskDBError::Protocol("APPEND requires at least two arguments".to_string()));
                }
                let value = parts[2..].join(" ");
                Ok(Request::Append { key: parts[1].to_string(), value })
            }
            
            // List operations
            "LPUSH" => {
                if parts.len() < 3 {
                    return Err(DiskDBError::Protocol("LPUSH requires at least two arguments".to_string()));
                }
                Ok(Request::LPush {
                    key: parts[1].to_string(),
                    values: parts[2..].iter().map(|s| s.to_string()).collect(),
                })
            }
            "RPUSH" => {
                if parts.len() < 3 {
                    return Err(DiskDBError::Protocol("RPUSH requires at least two arguments".to_string()));
                }
                Ok(Request::RPush {
                    key: parts[1].to_string(),
                    values: parts[2..].iter().map(|s| s.to_string()).collect(),
                })
            }
            "LPOP" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("LPOP requires exactly one argument".to_string()));
                }
                Ok(Request::LPop { key: parts[1].to_string() })
            }
            "RPOP" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("RPOP requires exactly one argument".to_string()));
                }
                Ok(Request::RPop { key: parts[1].to_string() })
            }
            "LRANGE" => {
                if parts.len() != 4 {
                    return Err(DiskDBError::Protocol("LRANGE requires exactly three arguments".to_string()));
                }
                let start = parts[2].parse::<i64>()
                    .map_err(|_| DiskDBError::Protocol("Invalid start index".to_string()))?;
                let stop = parts[3].parse::<i64>()
                    .map_err(|_| DiskDBError::Protocol("Invalid stop index".to_string()))?;
                Ok(Request::LRange { 
                    key: parts[1].to_string(), 
                    start, 
                    stop 
                })
            }
            "LLEN" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("LLEN requires exactly one argument".to_string()));
                }
                Ok(Request::LLen { key: parts[1].to_string() })
            }
            
            // Set operations
            "SADD" => {
                if parts.len() < 3 {
                    return Err(DiskDBError::Protocol("SADD requires at least two arguments".to_string()));
                }
                Ok(Request::SAdd {
                    key: parts[1].to_string(),
                    members: parts[2..].iter().map(|s| s.to_string()).collect(),
                })
            }
            "SREM" => {
                if parts.len() < 3 {
                    return Err(DiskDBError::Protocol("SREM requires at least two arguments".to_string()));
                }
                Ok(Request::SRem {
                    key: parts[1].to_string(),
                    members: parts[2..].iter().map(|s| s.to_string()).collect(),
                })
            }
            "SMEMBERS" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("SMEMBERS requires exactly one argument".to_string()));
                }
                Ok(Request::SMembers { key: parts[1].to_string() })
            }
            "SISMEMBER" => {
                if parts.len() != 3 {
                    return Err(DiskDBError::Protocol("SISMEMBER requires exactly two arguments".to_string()));
                }
                Ok(Request::SIsMember {
                    key: parts[1].to_string(),
                    member: parts[2].to_string(),
                })
            }
            "SCARD" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("SCARD requires exactly one argument".to_string()));
                }
                Ok(Request::SCard { key: parts[1].to_string() })
            }
            
            // Hash operations
            "HSET" => {
                if parts.len() != 4 {
                    return Err(DiskDBError::Protocol("HSET requires exactly three arguments".to_string()));
                }
                Ok(Request::HSet {
                    key: parts[1].to_string(),
                    field: parts[2].to_string(),
                    value: parts[3].to_string(),
                })
            }
            "HGET" => {
                if parts.len() != 3 {
                    return Err(DiskDBError::Protocol("HGET requires exactly two arguments".to_string()));
                }
                Ok(Request::HGet {
                    key: parts[1].to_string(),
                    field: parts[2].to_string(),
                })
            }
            "HDEL" => {
                if parts.len() < 3 {
                    return Err(DiskDBError::Protocol("HDEL requires at least two arguments".to_string()));
                }
                Ok(Request::HDel {
                    key: parts[1].to_string(),
                    fields: parts[2..].iter().map(|s| s.to_string()).collect(),
                })
            }
            "HGETALL" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("HGETALL requires exactly one argument".to_string()));
                }
                Ok(Request::HGetAll { key: parts[1].to_string() })
            }
            "HEXISTS" => {
                if parts.len() != 3 {
                    return Err(DiskDBError::Protocol("HEXISTS requires exactly two arguments".to_string()));
                }
                Ok(Request::HExists {
                    key: parts[1].to_string(),
                    field: parts[2].to_string(),
                })
            }
            
            // Sorted Set operations
            "ZADD" => {
                if parts.len() < 4 || (parts.len() - 2) % 2 != 0 {
                    return Err(DiskDBError::Protocol("ZADD requires key and score/member pairs".to_string()));
                }
                let mut members = Vec::new();
                for i in (2..parts.len()).step_by(2) {
                    let score = parts[i].parse::<f64>()
                        .map_err(|_| DiskDBError::Protocol("Invalid score".to_string()))?;
                    let member = parts[i + 1].to_string();
                    members.push((score, member));
                }
                Ok(Request::ZAdd {
                    key: parts[1].to_string(),
                    members,
                })
            }
            "ZREM" => {
                if parts.len() < 3 {
                    return Err(DiskDBError::Protocol("ZREM requires at least two arguments".to_string()));
                }
                Ok(Request::ZRem {
                    key: parts[1].to_string(),
                    members: parts[2..].iter().map(|s| s.to_string()).collect(),
                })
            }
            "ZRANGE" => {
                if parts.len() < 4 || parts.len() > 5 {
                    return Err(DiskDBError::Protocol("ZRANGE requires 3-4 arguments".to_string()));
                }
                let start = parts[2].parse::<i64>()
                    .map_err(|_| DiskDBError::Protocol("Invalid start index".to_string()))?;
                let stop = parts[3].parse::<i64>()
                    .map_err(|_| DiskDBError::Protocol("Invalid stop index".to_string()))?;
                let with_scores = parts.len() == 5 && parts[4].to_uppercase() == "WITHSCORES";
                Ok(Request::ZRange {
                    key: parts[1].to_string(),
                    start,
                    stop,
                    with_scores,
                })
            }
            "ZSCORE" => {
                if parts.len() != 3 {
                    return Err(DiskDBError::Protocol("ZSCORE requires exactly two arguments".to_string()));
                }
                Ok(Request::ZScore {
                    key: parts[1].to_string(),
                    member: parts[2].to_string(),
                })
            }
            "ZCARD" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("ZCARD requires exactly one argument".to_string()));
                }
                Ok(Request::ZCard { key: parts[1].to_string() })
            }
            
            // JSON operations
            "JSON.SET" => {
                if parts.len() < 4 {
                    return Err(DiskDBError::Protocol("JSON.SET requires at least three arguments".to_string()));
                }
                let value = parts[3..].join(" ");
                Ok(Request::JsonSet {
                    key: parts[1].to_string(),
                    path: parts[2].to_string(),
                    value,
                })
            }
            "JSON.GET" => {
                if parts.len() != 3 {
                    return Err(DiskDBError::Protocol("JSON.GET requires exactly two arguments".to_string()));
                }
                Ok(Request::JsonGet {
                    key: parts[1].to_string(),
                    path: parts[2].to_string(),
                })
            }
            "JSON.DEL" => {
                if parts.len() != 3 {
                    return Err(DiskDBError::Protocol("JSON.DEL requires exactly two arguments".to_string()));
                }
                Ok(Request::JsonDel {
                    key: parts[1].to_string(),
                    path: parts[2].to_string(),
                })
            }
            
            // Stream operations
            "XADD" => {
                if parts.len() < 5 || (parts.len() - 3) % 2 != 0 {
                    return Err(DiskDBError::Protocol("XADD requires key, id, and field/value pairs".to_string()));
                }
                let id = if parts[2] == "*" { None } else { Some(parts[2].to_string()) };
                let mut fields = HashMap::new();
                for i in (3..parts.len()).step_by(2) {
                    fields.insert(parts[i].to_string(), parts[i + 1].to_string());
                }
                Ok(Request::XAdd {
                    key: parts[1].to_string(),
                    id,
                    fields,
                })
            }
            "XRANGE" => {
                if parts.len() < 4 || parts.len() > 6 {
                    return Err(DiskDBError::Protocol("XRANGE requires 3-5 arguments".to_string()));
                }
                let count = if parts.len() >= 6 && parts[4].to_uppercase() == "COUNT" {
                    Some(parts[5].parse::<usize>()
                        .map_err(|_| DiskDBError::Protocol("Invalid count".to_string()))?)
                } else {
                    None
                };
                Ok(Request::XRange {
                    key: parts[1].to_string(),
                    start: parts[2].to_string(),
                    end: parts[3].to_string(),
                    count,
                })
            }
            "XLEN" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("XLEN requires exactly one argument".to_string()));
                }
                Ok(Request::XLen { key: parts[1].to_string() })
            }
            
            // Utility operations
            "TYPE" => {
                if parts.len() != 2 {
                    return Err(DiskDBError::Protocol("TYPE requires exactly one argument".to_string()));
                }
                Ok(Request::Type { key: parts[1].to_string() })
            }
            "DEL" => {
                if parts.len() < 2 {
                    return Err(DiskDBError::Protocol("DEL requires at least one argument".to_string()));
                }
                Ok(Request::Del {
                    keys: parts[1..].iter().map(|s| s.to_string()).collect(),
                })
            }
            "EXISTS" => {
                if parts.len() < 2 {
                    return Err(DiskDBError::Protocol("EXISTS requires at least one argument".to_string()));
                }
                Ok(Request::Exists {
                    keys: parts[1..].iter().map(|s| s.to_string()).collect(),
                })
            }
            
            cmd => Err(DiskDBError::InvalidCommand(cmd.to_string())),
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Ok => writeln!(f, "OK"),
            Response::Value(val) => writeln!(f, "{}", val),
            Response::Integer(val) => writeln!(f, "{}", val),
            Response::Array(arr) => {
                if arr.is_empty() {
                    writeln!(f, "(empty array)")
                } else {
                    writeln!(f, "{}", arr.join("\n"))
                }
            }
            Response::Null => writeln!(f, "(nil)"),
            Response::Error(msg) => writeln!(f, "ERROR: {}", msg),
        }
    }
}