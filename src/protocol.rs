use crate::error::{DiskDBError, Result};
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
    XAdd { key: String, id: String, fields: Vec<(String, String)> },
    XRange { key: String, start: String, end: String, count: Option<usize> },
    XLen { key: String },
    
    // Utility operations
    Type { key: String },
    Del { keys: Vec<String> },
    Exists { keys: Vec<String> },
    Ping,
    Echo { message: String },
    FlushDb,
    Info,
}

#[derive(Debug)]
pub enum Response {
    Ok,
    String(Option<String>),
    Integer(i64),
    Array(Vec<Response>),
    Null,
    Error(String),
}

impl Response {
    /// Parse a response from a string
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        
        if trimmed == "OK" {
            Ok(Response::Ok)
        } else if trimmed == "PONG" {
            Ok(Response::String(Some("PONG".to_string())))
        } else if trimmed.starts_with("ERROR:") {
            Ok(Response::Error(trimmed[6..].trim().to_string()))
        } else if trimmed.starts_with("STRING:") {
            let value = trimmed[7..].trim();
            if value == "(nil)" {
                Ok(Response::String(None))
            } else {
                Ok(Response::String(Some(value.to_string())))
            }
        } else if trimmed.starts_with("INTEGER:") {
            let value = trimmed[8..].trim().parse::<i64>()
                .map_err(|_| DiskDBError::Protocol("Invalid integer response".to_string()))?;
            Ok(Response::Integer(value))
        } else if trimmed.starts_with("ARRAY:") {
            // Simple array parsing - in production, use proper RESP parser
            Ok(Response::Array(vec![]))
        } else if trimmed == "(nil)" {
            Ok(Response::Null)
        } else {
            // Try to parse as a simple string response
            Ok(Response::String(Some(trimmed.to_string())))
        }
    }
}

impl Request {
    /// Convert request to string for network transmission
    pub fn to_string(&self) -> String {
        match self {
            Request::Get { key } => format!("GET {}", key),
            Request::Set { key, value } => format!("SET {} {}", key, value),
            Request::Del { keys } => format!("DEL {}", keys.join(" ")),
            Request::Exists { keys } => format!("EXISTS {}", keys.join(" ")),
            Request::Type { key } => format!("TYPE {}", key),
            Request::Incr { key } => format!("INCR {}", key),
            Request::Decr { key } => format!("DECR {}", key),
            Request::IncrBy { key, delta } => format!("INCRBY {} {}", key, delta),
            Request::DecrBy { key, delta } => format!("DECRBY {} {}", key, delta),
            Request::Append { key, value } => format!("APPEND {} {}", key, value),
            Request::LPush { key, values } => format!("LPUSH {} {}", key, values.join(" ")),
            Request::RPush { key, values } => format!("RPUSH {} {}", key, values.join(" ")),
            Request::LPop { key } => format!("LPOP {}", key),
            Request::RPop { key } => format!("RPOP {}", key),
            Request::LRange { key, start, stop } => format!("LRANGE {} {} {}", key, start, stop),
            Request::LLen { key } => format!("LLEN {}", key),
            Request::SAdd { key, members } => format!("SADD {} {}", key, members.join(" ")),
            Request::SRem { key, members } => format!("SREM {} {}", key, members.join(" ")),
            Request::SMembers { key } => format!("SMEMBERS {}", key),
            Request::SIsMember { key, member } => format!("SISMEMBER {} {}", key, member),
            Request::SCard { key } => format!("SCARD {}", key),
            Request::HSet { key, field, value } => format!("HSET {} {} {}", key, field, value),
            Request::HGet { key, field } => format!("HGET {} {}", key, field),
            Request::HDel { key, fields } => format!("HDEL {} {}", key, fields.join(" ")),
            Request::HGetAll { key } => format!("HGETALL {}", key),
            Request::HExists { key, field } => format!("HEXISTS {} {}", key, field),
            Request::ZAdd { key, members } => {
                let pairs: Vec<String> = members.iter()
                    .map(|(score, member)| format!("{} {}", score, member))
                    .collect();
                format!("ZADD {} {}", key, pairs.join(" "))
            }
            Request::ZRem { key, members } => format!("ZREM {} {}", key, members.join(" ")),
            Request::ZScore { key, member } => format!("ZSCORE {} {}", key, member),
            Request::ZRange { key, start, stop, with_scores } => {
                if *with_scores {
                    format!("ZRANGE {} {} {} WITHSCORES", key, start, stop)
                } else {
                    format!("ZRANGE {} {} {}", key, start, stop)
                }
            }
            Request::ZCard { key } => format!("ZCARD {}", key),
            Request::JsonSet { key, path, value } => format!("JSON.SET {} {} {}", key, path, value),
            Request::JsonGet { key, path } => format!("JSON.GET {} {}", key, path),
            Request::JsonDel { key, path } => format!("JSON.DEL {} {}", key, path),
            Request::XAdd { key, id, fields } => {
                let field_pairs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{} {}", k, v))
                    .collect();
                let id_str = id;
                format!("XADD {} {} {}", key, id_str, field_pairs.join(" "))
            }
            Request::XRange { key, start, end, count } => {
                if let Some(c) = count {
                    format!("XRANGE {} {} {} COUNT {}", key, start, end, c)
                } else {
                    format!("XRANGE {} {} {}", key, start, end)
                }
            }
            Request::XLen { key } => format!("XLEN {}", key),
            Request::Ping => "PING".to_string(),
            Request::Echo { message } => format!("ECHO {}", message),
            Request::FlushDb => "FLUSHDB".to_string(),
            Request::Info => "INFO".to_string(),
        }
    }
}

impl Request {
    pub fn parse(input: &str) -> Result<Self> {
        // Use C parser if feature is enabled
        #[cfg(feature = "c_parser")]
        {
            return crate::ffi::parser::parse_request_fast(input);
        }
        
        // Fall back to Rust parser
        #[cfg(not(feature = "c_parser"))]
        {
            Self::parse_rust(input)
        }
    }
    
    pub fn parse_rust(input: &str) -> Result<Self> {
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
                let id = parts[2].to_string();
                let mut fields = Vec::new();
                for i in (3..parts.len()).step_by(2) {
                    if i + 1 < parts.len() {
                        fields.push((parts[i].to_string(), parts[i + 1].to_string()));
                    }
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
            "PING" => Ok(Request::Ping),
            "ECHO" => {
                if parts.len() < 2 {
                    return Err(DiskDBError::Protocol("ECHO requires a message".to_string()));
                }
                Ok(Request::Echo { message: parts[1..].join(" ") })
            }
            "FLUSHDB" => Ok(Request::FlushDb),
            "INFO" => Ok(Request::Info),
            
            cmd => Err(DiskDBError::InvalidCommand(cmd.to_string())),
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Ok => writeln!(f, "OK"),
            Response::String(Some(val)) => writeln!(f, "{}", val),
            Response::String(None) => writeln!(f, "(nil)"),
            Response::Integer(val) => writeln!(f, "{}", val),
            Response::Array(arr) => {
                if arr.is_empty() {
                    writeln!(f, "(empty array)")
                } else {
                    for (i, item) in arr.iter().enumerate() {
                        if i > 0 {
                            write!(f, "\n")?;
                        }
                        write!(f, "{}", item)?;
                    }
                    writeln!(f)
                }
            }
            Response::Null => writeln!(f, "(nil)"),
            Response::Error(msg) => writeln!(f, "ERROR: {}", msg),
        }
    }
}