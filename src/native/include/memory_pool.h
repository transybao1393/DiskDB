#ifndef DISKDB_MEMORY_POOL_H
#define DISKDB_MEMORY_POOL_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Memory pool configuration
typedef struct {
    size_t initial_pool_size;    // Initial size for each pool
    size_t max_pool_size;        // Maximum size before falling back to malloc
    size_t thread_cache_size;    // Size of thread-local cache
    bool enable_statistics;      // Track allocation statistics
} MemoryPoolConfig;

// Slab allocator for fixed-size objects
typedef struct SlabAllocator SlabAllocator;

// Memory pool manager
typedef struct MemoryPool MemoryPool;

// Allocation statistics
typedef struct {
    uint64_t allocations;
    uint64_t deallocations;
    uint64_t bytes_allocated;
    uint64_t bytes_freed;
    uint64_t pool_hits;
    uint64_t pool_misses;
    uint64_t active_objects;
} MemoryStats;

// Common sizes for Redis-like operations
#define POOL_SIZE_16    16
#define POOL_SIZE_32    32
#define POOL_SIZE_64    64
#define POOL_SIZE_128   128
#define POOL_SIZE_256   256
#define POOL_SIZE_512   512
#define POOL_SIZE_1024  1024
#define POOL_SIZE_4096  4096

// Initialize global memory pool system
int memory_pool_init(const MemoryPoolConfig* config);

// Shutdown memory pool system
void memory_pool_shutdown(void);

// Allocate memory from pool
void* pool_alloc(size_t size);

// Allocate zeroed memory from pool
void* pool_calloc(size_t count, size_t size);

// Reallocate memory
void* pool_realloc(void* ptr, size_t old_size, size_t new_size);

// Free memory back to pool
void pool_free(void* ptr, size_t size);

// String operations with pooled memory
char* pool_strdup(const char* str);
char* pool_strndup(const char* str, size_t n);

// Get current memory statistics
void pool_get_stats(MemoryStats* stats);

// Reset statistics
void pool_reset_stats(void);

// Thread-local pool operations
void* tls_pool_alloc(size_t size);
void tls_pool_free(void* ptr, size_t size);
void tls_pool_clear(void);

// Slab allocator operations (for specific sizes)
SlabAllocator* slab_create(size_t object_size, size_t objects_per_slab);
void slab_destroy(SlabAllocator* slab);
void* slab_alloc(SlabAllocator* slab);
void slab_free(SlabAllocator* slab, void* ptr);
size_t slab_get_object_size(const SlabAllocator* slab);

// Memory pool for specific object types
MemoryPool* pool_create_typed(size_t object_size, size_t initial_count);
void pool_destroy_typed(MemoryPool* pool);
void* pool_get_object(MemoryPool* pool);
void pool_return_object(MemoryPool* pool, void* obj);

// Bulk operations
void* pool_alloc_bulk(size_t size, size_t count);
void pool_free_bulk(void* ptr, size_t size, size_t count);

#ifdef __cplusplus
}
#endif

#endif // DISKDB_MEMORY_POOL_H