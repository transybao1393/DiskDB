#ifndef DISKDB_PARSER_H
#define DISKDB_PARSER_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Maximum number of arguments in a command
#define MAX_ARGS 128
#define MAX_INLINE_STRING 32

// String view - points to existing memory without allocation
typedef struct {
    const char* data;
    size_t len;
} StringView;

// Command types matching Rust enum
typedef enum {
    CMD_UNKNOWN = 0,
    // String operations
    CMD_GET,
    CMD_SET,
    CMD_INCR,
    CMD_DECR,
    CMD_INCRBY,
    CMD_APPEND,
    // List operations
    CMD_LPUSH,
    CMD_RPUSH,
    CMD_LPOP,
    CMD_RPOP,
    CMD_LRANGE,
    CMD_LLEN,
    // Set operations
    CMD_SADD,
    CMD_SREM,
    CMD_SISMEMBER,
    CMD_SMEMBERS,
    CMD_SCARD,
    // Hash operations
    CMD_HSET,
    CMD_HGET,
    CMD_HDEL,
    CMD_HGETALL,
    CMD_HEXISTS,
    // Sorted set operations
    CMD_ZADD,
    CMD_ZREM,
    CMD_ZSCORE,
    CMD_ZRANGE,
    CMD_ZCARD,
    // JSON operations
    CMD_JSON_SET,
    CMD_JSON_GET,
    CMD_JSON_DEL,
    // Stream operations
    CMD_XADD,
    CMD_XLEN,
    CMD_XRANGE,
    // Utility operations
    CMD_TYPE,
    CMD_EXISTS,
    CMD_DEL,
    CMD_PING,
    CMD_ECHO,
    CMD_FLUSHDB,
    CMD_INFO
} CommandType;

// Parsed request structure - zero allocation
typedef struct {
    CommandType type;
    StringView key;
    StringView args[MAX_ARGS];
    int arg_count;
    
    // For numeric arguments (pre-parsed)
    union {
        int64_t integer_arg;
        double float_arg;
    } numeric;
    
    // Error information
    const char* error;
} ParsedRequest;

// Thread-local arena for temporary allocations
typedef struct Arena Arena;

// Initialize parser (call once per thread)
Arena* parser_init_thread_arena(size_t size);

// Clean up thread arena
void parser_cleanup_thread_arena(Arena* arena);

// Reset arena for next parse (very fast)
void parser_reset_arena(Arena* arena);

// Main parsing function - zero copy
ParsedRequest* parse_request(const char* input, size_t len, Arena* arena);

// Helper to convert command string to enum (optimized with perfect hash)
CommandType get_command_type(const char* cmd, size_t len);

// Get command name from type
const char* get_command_name(CommandType type);

// Validate parsed request
int validate_request(const ParsedRequest* req);

#ifdef __cplusplus
}
#endif

#endif // DISKDB_PARSER_H