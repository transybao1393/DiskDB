#include "../include/parser.h"
#include "../include/arena.h"
#include <string.h>
#include <ctype.h>
#include <stdlib.h>
#include <stdio.h>

// Command lookup table for O(1) command detection
typedef struct {
    const char* name;
    CommandType type;
    int min_args;
    int max_args;
} CommandInfo;

static const CommandInfo commands[] = {
    // String operations
    {"GET", CMD_GET, 1, 1},
    {"SET", CMD_SET, 2, 2},
    {"INCR", CMD_INCR, 1, 1},
    {"DECR", CMD_DECR, 1, 1},
    {"INCRBY", CMD_INCRBY, 2, 2},
    {"APPEND", CMD_APPEND, 2, 2},
    // List operations
    {"LPUSH", CMD_LPUSH, 2, MAX_ARGS},
    {"RPUSH", CMD_RPUSH, 2, MAX_ARGS},
    {"LPOP", CMD_LPOP, 1, 1},
    {"RPOP", CMD_RPOP, 1, 1},
    {"LRANGE", CMD_LRANGE, 3, 3},
    {"LLEN", CMD_LLEN, 1, 1},
    // Set operations
    {"SADD", CMD_SADD, 2, MAX_ARGS},
    {"SREM", CMD_SREM, 2, MAX_ARGS},
    {"SISMEMBER", CMD_SISMEMBER, 2, 2},
    {"SMEMBERS", CMD_SMEMBERS, 1, 1},
    {"SCARD", CMD_SCARD, 1, 1},
    // Hash operations
    {"HSET", CMD_HSET, 3, 3},
    {"HGET", CMD_HGET, 2, 2},
    {"HDEL", CMD_HDEL, 2, MAX_ARGS},
    {"HGETALL", CMD_HGETALL, 1, 1},
    {"HEXISTS", CMD_HEXISTS, 2, 2},
    // Sorted set operations
    {"ZADD", CMD_ZADD, 3, MAX_ARGS},
    {"ZREM", CMD_ZREM, 2, MAX_ARGS},
    {"ZSCORE", CMD_ZSCORE, 2, 2},
    {"ZRANGE", CMD_ZRANGE, 3, 4},
    {"ZCARD", CMD_ZCARD, 1, 1},
    // JSON operations
    {"JSON.SET", CMD_JSON_SET, 3, 3},
    {"JSON.GET", CMD_JSON_GET, 2, 2},
    {"JSON.DEL", CMD_JSON_DEL, 2, 2},
    // Stream operations
    {"XADD", CMD_XADD, 3, MAX_ARGS},
    {"XLEN", CMD_XLEN, 1, 1},
    {"XRANGE", CMD_XRANGE, 3, 5},
    // Utility operations
    {"TYPE", CMD_TYPE, 1, 1},
    {"EXISTS", CMD_EXISTS, 1, MAX_ARGS},
    {"DEL", CMD_DEL, 1, MAX_ARGS},
    {"PING", CMD_PING, 0, 1},
    {"ECHO", CMD_ECHO, 1, 1},
    {"FLUSHDB", CMD_FLUSHDB, 0, 0},
    {"INFO", CMD_INFO, 0, 1},
    {NULL, CMD_UNKNOWN, 0, 0}
};

// Fast command lookup using string comparison
static CommandType get_command_type_internal(const char* cmd, size_t len) {
    // Convert to uppercase for comparison (in-place would be faster but modifies input)
    char upper[32];
    if (len >= sizeof(upper)) return CMD_UNKNOWN;
    
    for (size_t i = 0; i < len; i++) {
        upper[i] = toupper(cmd[i]);
    }
    upper[len] = '\0';
    
    // Linear search (could be optimized with perfect hash)
    for (int i = 0; commands[i].name != NULL; i++) {
        if (strcmp(upper, commands[i].name) == 0) {
            return commands[i].type;
        }
    }
    
    return CMD_UNKNOWN;
}

// Skip whitespace and return pointer to next non-whitespace
static inline const char* skip_whitespace(const char* p, const char* end) {
    while (p < end && (*p == ' ' || *p == '\t' || *p == '\r')) {
        p++;
    }
    return p;
}

// Find next whitespace or end of line
static inline const char* find_whitespace(const char* p, const char* end) {
    while (p < end && *p != ' ' && *p != '\t' && *p != '\r' && *p != '\n') {
        p++;
    }
    return p;
}

ParsedRequest* parse_request(const char* input, size_t len, Arena* arena) {
    if (!input || len == 0 || !arena) {
        return NULL;
    }
    
    // Allocate parsed request from arena
    ParsedRequest* req = (ParsedRequest*)arena_alloc(arena, sizeof(ParsedRequest));
    if (!req) return NULL;
    
    // Initialize
    memset(req, 0, sizeof(ParsedRequest));
    req->type = CMD_UNKNOWN;
    req->arg_count = 0;
    
    const char* p = input;
    const char* end = input + len;
    
    // Skip leading whitespace
    p = skip_whitespace(p, end);
    if (p >= end) {
        req->error = "Empty command";
        return req;
    }
    
    // Extract command
    const char* cmd_start = p;
    p = find_whitespace(p, end);
    size_t cmd_len = p - cmd_start;
    
    // Identify command type
    req->type = get_command_type_internal(cmd_start, cmd_len);
    if (req->type == CMD_UNKNOWN) {
        req->error = "Unknown command";
        return req;
    }
    
    // Get command info
    const CommandInfo* info = NULL;
    for (int i = 0; commands[i].name != NULL; i++) {
        if (commands[i].type == req->type) {
            info = &commands[i];
            break;
        }
    }
    
    // Parse arguments
    while (p < end && req->arg_count < MAX_ARGS) {
        // Skip whitespace
        p = skip_whitespace(p, end);
        if (p >= end || *p == '\n') break;
        
        // Find argument boundaries
        const char* arg_start = p;
        
        // Check if this is a quoted string
        if (*p == '"' || *p == '\'') {
            char quote = *p;
            p++; // Skip opening quote
            arg_start = p;
            
            // Find closing quote
            while (p < end && *p != quote) {
                if (*p == '\\' && p + 1 < end) {
                    p += 2; // Skip escaped character
                } else {
                    p++;
                }
            }
            
            if (p >= end) {
                req->error = "Unclosed quote";
                return req;
            }
            
            // Store argument without quotes
            req->args[req->arg_count].data = arg_start;
            req->args[req->arg_count].len = p - arg_start;
            req->arg_count++;
            
            p++; // Skip closing quote
        } else {
            // Regular argument
            p = find_whitespace(p, end);
            
            req->args[req->arg_count].data = arg_start;
            req->args[req->arg_count].len = p - arg_start;
            req->arg_count++;
        }
    }
    
    // Set key for commands that have one
    if (req->arg_count > 0 && info && info->min_args >= 1) {
        req->key = req->args[0];
    }
    
    // Validate argument count
    if (req->arg_count < info->min_args) {
        req->error = "Too few arguments";
        return req;
    }
    if (info->max_args < MAX_ARGS && req->arg_count > info->max_args) {
        req->error = "Too many arguments";
        return req;
    }
    
    // Pre-parse numeric arguments for specific commands
    switch (req->type) {
        case CMD_INCRBY:
            if (req->arg_count >= 2) {
                char* endptr;
                req->numeric.integer_arg = strtoll(req->args[1].data, &endptr, 10);
                if (endptr != req->args[1].data + req->args[1].len) {
                    req->error = "Invalid integer";
                }
            }
            break;
        
        case CMD_ZADD:
            // For ZADD, we'll let Rust handle the score parsing
            break;
            
        case CMD_LRANGE:
            // Pre-parse start and stop indices
            if (req->arg_count >= 3) {
                // We'll let Rust handle this for now
            }
            break;
            
        default:
            break;
    }
    
    return req;
}

CommandType get_command_type(const char* cmd, size_t len) {
    return get_command_type_internal(cmd, len);
}

const char* get_command_name(CommandType type) {
    for (int i = 0; commands[i].name != NULL; i++) {
        if (commands[i].type == type) {
            return commands[i].name;
        }
    }
    return "UNKNOWN";
}

int validate_request(const ParsedRequest* req) {
    if (!req) return 0;
    if (req->error != NULL) return 0;
    if (req->type == CMD_UNKNOWN) return 0;
    
    return 1;
}

// Thread-local arena initialization
static __thread Arena* tls_parser_arena = NULL;

Arena* parser_init_thread_arena(size_t size) {
    if (tls_parser_arena) {
        arena_destroy(tls_parser_arena);
    }
    
    tls_parser_arena = arena_create(size);
    return tls_parser_arena;
}

void parser_cleanup_thread_arena(Arena* arena) {
    if (arena == tls_parser_arena) {
        tls_parser_arena = NULL;
    }
    arena_destroy(arena);
}

void parser_reset_arena(Arena* arena) {
    arena_reset(arena);
}