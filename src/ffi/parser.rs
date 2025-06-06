use std::ffi::{c_char, CStr};
use std::os::raw::c_int;
use std::ptr::NonNull;
use std::marker::PhantomData;
use crate::protocol::Request;
use crate::error::{Result, DiskDBError};

// FFI type definitions matching C structures
#[repr(C)]
pub struct StringView {
    pub data: *const c_char,
    pub len: usize,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CommandType {
    Unknown = 0,
    // String operations
    Get,
    Set,
    Incr,
    Decr,
    IncrBy,
    Append,
    // List operations
    LPush,
    RPush,
    LPop,
    RPop,
    LRange,
    LLen,
    // Set operations
    SAdd,
    SRem,
    SIsMember,
    SMembers,
    SCard,
    // Hash operations
    HSet,
    HGet,
    HDel,
    HGetAll,
    HExists,
    // Sorted set operations
    ZAdd,
    ZRem,
    ZScore,
    ZRange,
    ZCard,
    // JSON operations
    JsonSet,
    JsonGet,
    JsonDel,
    // Stream operations
    XAdd,
    XLen,
    XRange,
    // Utility operations
    Type,
    Exists,
    Del,
    Ping,
    Echo,
    FlushDb,
    Info,
}

#[repr(C)]
pub struct ParsedRequest {
    pub cmd_type: CommandType,
    pub key: StringView,
    pub args: [StringView; 128],
    pub arg_count: c_int,
    pub numeric: ParsedRequestNumeric,
    pub error: *const c_char,
}

#[repr(C)]
pub union ParsedRequestNumeric {
    pub integer_arg: i64,
    pub float_arg: f64,
}

// Opaque arena type
#[repr(C)]
pub struct Arena {
    _private: [u8; 0],
}

// External C functions
extern "C" {
    pub fn parser_init_thread_arena(size: usize) -> *mut Arena;
    pub fn parser_cleanup_thread_arena(arena: *mut Arena);
    pub fn parser_reset_arena(arena: *mut Arena);
    pub fn parse_request(input: *const c_char, len: usize, arena: *mut Arena) -> *mut ParsedRequest;
    pub fn validate_request(req: *const ParsedRequest) -> c_int;
}

// Safe Rust wrapper around the C parser
pub struct SafeParser {
    arena: NonNull<Arena>,
    _phantom: PhantomData<*const ()>,
}

// SAFETY: The parser uses thread-local arena, so it's safe to send between threads
unsafe impl Send for SafeParser {}
// SAFETY: The parser doesn't have shared mutable state
unsafe impl Sync for SafeParser {}

impl SafeParser {
    /// Create a new parser with thread-local arena
    pub fn new(arena_size: usize) -> Result<Self> {
        unsafe {
            let arena = parser_init_thread_arena(arena_size);
            if arena.is_null() {
                return Err(DiskDBError::Database("Failed to initialize parser arena".into()));
            }
            
            Ok(SafeParser {
                arena: NonNull::new_unchecked(arena),
                _phantom: PhantomData,
            })
        }
    }
    
    /// Parse a request using zero-copy C parser
    pub fn parse(&self, input: &str) -> Result<Request> {
        unsafe {
            // Reset arena for this parse
            parser_reset_arena(self.arena.as_ptr());
            
            // Parse using C parser
            let parsed = parse_request(
                input.as_ptr() as *const c_char,
                input.len(),
                self.arena.as_ptr()
            );
            
            if parsed.is_null() {
                return Err(DiskDBError::Protocol("Parse error: null result".into()));
            }
            
            let parsed_ref = &*parsed;
            
            // Check for parse errors
            if !parsed_ref.error.is_null() {
                let error_str = CStr::from_ptr(parsed_ref.error).to_string_lossy();
                return Err(DiskDBError::Protocol(format!("Parse error: {}", error_str)));
            }
            
            // Validate request
            if validate_request(parsed) == 0 {
                return Err(DiskDBError::Protocol("Invalid request".into()));
            }
            
            // Convert C parsed request to Rust Request
            self.convert_to_rust_request(parsed_ref)
        }
    }
    
    /// Convert C ParsedRequest to Rust Request
    fn convert_to_rust_request(&self, parsed: &ParsedRequest) -> Result<Request> {
        // Helper to convert StringView to String
        let string_view_to_string = |sv: &StringView| -> String {
            if sv.data.is_null() || sv.len == 0 {
                String::new()
            } else {
                unsafe {
                    let slice = std::slice::from_raw_parts(sv.data as *const u8, sv.len);
                    String::from_utf8_lossy(slice).into_owned()
                }
            }
        };
        
        // Helper to get argument at index
        let get_arg = |index: usize| -> String {
            if index < parsed.arg_count as usize {
                string_view_to_string(&parsed.args[index])
            } else {
                String::new()
            }
        };
        
        // Convert based on command type
        let request = match parsed.cmd_type {
            CommandType::Get => Request::Get { 
                key: get_arg(0) 
            },
            CommandType::Set => Request::Set { 
                key: get_arg(0), 
                value: get_arg(1) 
            },
            CommandType::Incr => Request::Incr { 
                key: get_arg(0) 
            },
            CommandType::Decr => Request::Decr { 
                key: get_arg(0) 
            },
            CommandType::IncrBy => Request::IncrBy { 
                key: get_arg(0), 
                delta: unsafe { parsed.numeric.integer_arg } 
            },
            CommandType::Append => Request::Append { 
                key: get_arg(0), 
                value: get_arg(1) 
            },
            CommandType::LPush => {
                let key = get_arg(0);
                let values: Vec<String> = (1..parsed.arg_count as usize)
                    .map(|i| get_arg(i))
                    .collect();
                Request::LPush { key, values }
            },
            CommandType::RPush => {
                let key = get_arg(0);
                let values: Vec<String> = (1..parsed.arg_count as usize)
                    .map(|i| get_arg(i))
                    .collect();
                Request::RPush { key, values }
            },
            CommandType::LPop => Request::LPop { 
                key: get_arg(0) 
            },
            CommandType::RPop => Request::RPop { 
                key: get_arg(0) 
            },
            CommandType::LRange => Request::LRange { 
                key: get_arg(0),
                start: get_arg(1).parse().unwrap_or(0),
                stop: get_arg(2).parse().unwrap_or(-1),
            },
            CommandType::LLen => Request::LLen { 
                key: get_arg(0) 
            },
            CommandType::SAdd => {
                let key = get_arg(0);
                let members: Vec<String> = (1..parsed.arg_count as usize)
                    .map(|i| get_arg(i))
                    .collect();
                Request::SAdd { key, members }
            },
            CommandType::SRem => {
                let key = get_arg(0);
                let members: Vec<String> = (1..parsed.arg_count as usize)
                    .map(|i| get_arg(i))
                    .collect();
                Request::SRem { key, members }
            },
            CommandType::SIsMember => Request::SIsMember { 
                key: get_arg(0), 
                member: get_arg(1) 
            },
            CommandType::SMembers => Request::SMembers { 
                key: get_arg(0) 
            },
            CommandType::SCard => Request::SCard { 
                key: get_arg(0) 
            },
            CommandType::HSet => Request::HSet { 
                key: get_arg(0),
                field: get_arg(1),
                value: get_arg(2),
            },
            CommandType::HGet => Request::HGet { 
                key: get_arg(0),
                field: get_arg(1),
            },
            CommandType::HDel => {
                let key = get_arg(0);
                let fields: Vec<String> = (1..parsed.arg_count as usize)
                    .map(|i| get_arg(i))
                    .collect();
                Request::HDel { key, fields }
            },
            CommandType::HGetAll => Request::HGetAll { 
                key: get_arg(0) 
            },
            CommandType::HExists => Request::HExists { 
                key: get_arg(0),
                field: get_arg(1),
            },
            CommandType::ZAdd => {
                let key = get_arg(0);
                let mut members = Vec::new();
                
                // Parse score-member pairs
                let mut i = 1;
                while i + 1 < parsed.arg_count as usize {
                    let score = get_arg(i).parse::<f64>()
                        .map_err(|_| DiskDBError::Protocol("Invalid score in ZADD".into()))?;
                    let member = get_arg(i + 1);
                    members.push((member, score));
                    i += 2;
                }
                
                Request::ZAdd { key, members: members.into_iter().map(|(m, s)| (s, m)).collect() }
            },
            CommandType::ZRem => {
                let key = get_arg(0);
                let members: Vec<String> = (1..parsed.arg_count as usize)
                    .map(|i| get_arg(i))
                    .collect();
                Request::ZRem { key, members }
            },
            CommandType::ZScore => Request::ZScore { 
                key: get_arg(0),
                member: get_arg(1),
            },
            CommandType::ZRange => {
                let key = get_arg(0);
                let start = get_arg(1).parse().unwrap_or(0);
                let stop = get_arg(2).parse().unwrap_or(-1);
                let with_scores = parsed.arg_count > 3 && 
                    get_arg(3).to_uppercase() == "WITHSCORES";
                Request::ZRange { key, start, stop, with_scores }
            },
            CommandType::ZCard => Request::ZCard { 
                key: get_arg(0) 
            },
            CommandType::JsonSet => Request::JsonSet { 
                key: get_arg(0),
                path: get_arg(1),
                value: get_arg(2),
            },
            CommandType::JsonGet => Request::JsonGet { 
                key: get_arg(0),
                path: get_arg(1),
            },
            CommandType::JsonDel => Request::JsonDel { 
                key: get_arg(0),
                path: get_arg(1),
            },
            CommandType::XAdd => {
                let key = get_arg(0);
                let id = get_arg(1);
                let mut fields = Vec::new();
                
                // Parse field-value pairs
                let mut i = 2;
                while i + 1 < parsed.arg_count as usize {
                    fields.push((get_arg(i), get_arg(i + 1)));
                    i += 2;
                }
                
                Request::XAdd { key, id, fields }
            },
            CommandType::XLen => Request::XLen { 
                key: get_arg(0) 
            },
            CommandType::XRange => {
                let key = get_arg(0);
                let start = get_arg(1);
                let end = get_arg(2);
                let count = if parsed.arg_count > 4 && get_arg(3).to_uppercase() == "COUNT" {
                    Some(get_arg(4).parse().unwrap_or(10))
                } else {
                    None
                };
                Request::XRange { key, start, end, count }
            },
            CommandType::Type => Request::Type { 
                key: get_arg(0) 
            },
            CommandType::Exists => {
                let keys: Vec<String> = (0..parsed.arg_count as usize)
                    .map(|i| get_arg(i))
                    .collect();
                Request::Exists { keys }
            },
            CommandType::Del => {
                let keys: Vec<String> = (0..parsed.arg_count as usize)
                    .map(|i| get_arg(i))
                    .collect();
                Request::Del { keys }
            },
            CommandType::Ping => Request::Ping,
            CommandType::Echo => Request::Echo { 
                message: get_arg(0) 
            },
            CommandType::FlushDb => Request::FlushDb,
            CommandType::Info => Request::Info,
            CommandType::Unknown => {
                return Err(DiskDBError::Protocol("Unknown command".into()));
            }
        };
        
        Ok(request)
    }
}

impl Drop for SafeParser {
    fn drop(&mut self) {
        unsafe {
            parser_cleanup_thread_arena(self.arena.as_ptr());
        }
    }
}

// Thread-local parser instance
thread_local! {
    static PARSER: std::cell::RefCell<Option<SafeParser>> = std::cell::RefCell::new(None);
}

/// Parse request using thread-local C parser
pub fn parse_request_fast(input: &str) -> Result<Request> {
    PARSER.with(|p| {
        let mut parser_ref = p.borrow_mut();
        if parser_ref.is_none() {
            // Initialize with 64KB arena
            match SafeParser::new(65536) {
                Ok(parser) => *parser_ref = Some(parser),
                Err(e) => return Err(e),
            }
        }
        
        parser_ref.as_ref().unwrap().parse(input)
    })
}