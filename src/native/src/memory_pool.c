#include "../include/memory_pool.h"
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <assert.h>

// Size classes for pooling
static const size_t size_classes[] = {
    16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192
};
#define NUM_SIZE_CLASSES (sizeof(size_classes) / sizeof(size_classes[0]))

// Global memory pool state
typedef struct {
    SlabAllocator* slabs[NUM_SIZE_CLASSES];
    pthread_mutex_t stats_lock;
    MemoryStats stats;
    bool initialized;
    MemoryPoolConfig config;
} GlobalMemoryPool;

static GlobalMemoryPool g_pool = {0};

// Thread-local cache
typedef struct {
    void* cache[NUM_SIZE_CLASSES][8];  // 8 objects per size class
    int count[NUM_SIZE_CLASSES];
} ThreadCache;

static __thread ThreadCache* tls_cache = NULL;
static __thread bool tls_initialized = false;

// Find appropriate size class
static int find_size_class(size_t size) {
    for (size_t i = 0; i < NUM_SIZE_CLASSES; i++) {
        if (size <= size_classes[i]) {
            return (int)i;
        }
    }
    return -1;
}

// Initialize thread-local cache
static void init_thread_cache(void) {
    if (tls_initialized) return;
    
    tls_cache = (ThreadCache*)calloc(1, sizeof(ThreadCache));
    tls_initialized = true;
}

int memory_pool_init(const MemoryPoolConfig* config) {
    if (g_pool.initialized) return 0;
    
    memset(&g_pool, 0, sizeof(g_pool));
    
    if (config) {
        g_pool.config = *config;
    } else {
        // Default config
        g_pool.config.initial_pool_size = 1024 * 1024;  // 1MB
        g_pool.config.max_pool_size = 16 * 1024 * 1024;  // 16MB
        g_pool.config.thread_cache_size = 8;
        g_pool.config.enable_statistics = true;
    }
    
    pthread_mutex_init(&g_pool.stats_lock, NULL);
    
    // Initialize slab allocators
    for (size_t i = 0; i < NUM_SIZE_CLASSES; i++) {
        size_t objects_per_slab = g_pool.config.initial_pool_size / size_classes[i];
        if (objects_per_slab < 64) objects_per_slab = 64;
        if (objects_per_slab > 1024) objects_per_slab = 1024;
        
        g_pool.slabs[i] = slab_create(size_classes[i], objects_per_slab);
        if (!g_pool.slabs[i]) {
            // Cleanup on failure
            for (size_t j = 0; j < i; j++) {
                slab_destroy(g_pool.slabs[j]);
            }
            return -1;
        }
    }
    
    g_pool.initialized = true;
    return 0;
}

void memory_pool_shutdown(void) {
    if (!g_pool.initialized) return;
    
    // Destroy all slab allocators
    for (size_t i = 0; i < NUM_SIZE_CLASSES; i++) {
        if (g_pool.slabs[i]) {
            slab_destroy(g_pool.slabs[i]);
            g_pool.slabs[i] = NULL;
        }
    }
    
    pthread_mutex_destroy(&g_pool.stats_lock);
    g_pool.initialized = false;
}

void* pool_alloc(size_t size) {
    if (!g_pool.initialized || size == 0) {
        return malloc(size);
    }
    
    init_thread_cache();
    
    int class_idx = find_size_class(size);
    if (class_idx < 0) {
        // Too large for pool
        if (g_pool.config.enable_statistics) {
            pthread_mutex_lock(&g_pool.stats_lock);
            g_pool.stats.allocations++;
            g_pool.stats.bytes_allocated += size;
            g_pool.stats.pool_misses++;
            pthread_mutex_unlock(&g_pool.stats_lock);
        }
        return malloc(size);
    }
    
    // Try thread-local cache first
    if (tls_cache && tls_cache->count[class_idx] > 0) {
        void* ptr = tls_cache->cache[class_idx][--tls_cache->count[class_idx]];
        
        if (g_pool.config.enable_statistics) {
            pthread_mutex_lock(&g_pool.stats_lock);
            g_pool.stats.allocations++;
            g_pool.stats.bytes_allocated += size_classes[class_idx];
            g_pool.stats.pool_hits++;
            g_pool.stats.active_objects++;
            pthread_mutex_unlock(&g_pool.stats_lock);
        }
        
        return ptr;
    }
    
    // Allocate from slab
    void* ptr = slab_alloc(g_pool.slabs[class_idx]);
    
    if (g_pool.config.enable_statistics) {
        pthread_mutex_lock(&g_pool.stats_lock);
        g_pool.stats.allocations++;
        g_pool.stats.bytes_allocated += size_classes[class_idx];
        if (ptr) {
            g_pool.stats.pool_hits++;
            g_pool.stats.active_objects++;
        } else {
            g_pool.stats.pool_misses++;
        }
        pthread_mutex_unlock(&g_pool.stats_lock);
    }
    
    // Fall back to malloc if slab allocation fails
    if (!ptr) {
        ptr = malloc(size);
    }
    
    return ptr;
}

void* pool_calloc(size_t count, size_t size) {
    size_t total = count * size;
    void* ptr = pool_alloc(total);
    if (ptr) {
        memset(ptr, 0, total);
    }
    return ptr;
}

void pool_free(void* ptr, size_t size) {
    if (!ptr) return;
    
    if (!g_pool.initialized) {
        free(ptr);
        return;
    }
    
    init_thread_cache();
    
    int class_idx = find_size_class(size);
    if (class_idx < 0) {
        // Too large for pool
        if (g_pool.config.enable_statistics) {
            pthread_mutex_lock(&g_pool.stats_lock);
            g_pool.stats.deallocations++;
            g_pool.stats.bytes_freed += size;
            pthread_mutex_unlock(&g_pool.stats_lock);
        }
        free(ptr);
        return;
    }
    
    // Try to add to thread-local cache
    if (tls_cache && tls_cache->count[class_idx] < 8) {
        tls_cache->cache[class_idx][tls_cache->count[class_idx]++] = ptr;
        
        if (g_pool.config.enable_statistics) {
            pthread_mutex_lock(&g_pool.stats_lock);
            g_pool.stats.deallocations++;
            g_pool.stats.bytes_freed += size_classes[class_idx];
            g_pool.stats.active_objects--;
            pthread_mutex_unlock(&g_pool.stats_lock);
        }
        
        return;
    }
    
    // Return to slab
    slab_free(g_pool.slabs[class_idx], ptr);
    
    if (g_pool.config.enable_statistics) {
        pthread_mutex_lock(&g_pool.stats_lock);
        g_pool.stats.deallocations++;
        g_pool.stats.bytes_freed += size_classes[class_idx];
        g_pool.stats.active_objects--;
        pthread_mutex_unlock(&g_pool.stats_lock);
    }
}

void* pool_realloc(void* ptr, size_t old_size, size_t new_size) {
    if (!ptr) return pool_alloc(new_size);
    if (new_size == 0) {
        pool_free(ptr, old_size);
        return NULL;
    }
    
    // If sizes are in same class, no need to reallocate
    int old_class = find_size_class(old_size);
    int new_class = find_size_class(new_size);
    
    if (old_class == new_class && old_class >= 0) {
        return ptr;
    }
    
    // Allocate new and copy
    void* new_ptr = pool_alloc(new_size);
    if (new_ptr) {
        size_t copy_size = old_size < new_size ? old_size : new_size;
        memcpy(new_ptr, ptr, copy_size);
        pool_free(ptr, old_size);
    }
    
    return new_ptr;
}

char* pool_strdup(const char* str) {
    if (!str) return NULL;
    
    size_t len = strlen(str) + 1;
    char* copy = (char*)pool_alloc(len);
    if (copy) {
        memcpy(copy, str, len);
    }
    return copy;
}

char* pool_strndup(const char* str, size_t n) {
    if (!str) return NULL;
    
    size_t len = strnlen(str, n);
    char* copy = (char*)pool_alloc(len + 1);
    if (copy) {
        memcpy(copy, str, len);
        copy[len] = '\0';
    }
    return copy;
}

void pool_get_stats(MemoryStats* stats) {
    if (!stats || !g_pool.initialized) return;
    
    pthread_mutex_lock(&g_pool.stats_lock);
    *stats = g_pool.stats;
    pthread_mutex_unlock(&g_pool.stats_lock);
}

void pool_reset_stats(void) {
    if (!g_pool.initialized) return;
    
    pthread_mutex_lock(&g_pool.stats_lock);
    memset(&g_pool.stats, 0, sizeof(g_pool.stats));
    pthread_mutex_unlock(&g_pool.stats_lock);
}

// Thread-local pool operations
void* tls_pool_alloc(size_t size) {
    return pool_alloc(size);
}

void tls_pool_free(void* ptr, size_t size) {
    pool_free(ptr, size);
}

void tls_pool_clear(void) {
    if (!tls_cache) return;
    
    // Return all cached objects to slabs
    for (size_t i = 0; i < NUM_SIZE_CLASSES; i++) {
        while (tls_cache->count[i] > 0) {
            void* ptr = tls_cache->cache[i][--tls_cache->count[i]];
            slab_free(g_pool.slabs[i], ptr);
        }
    }
}

// Typed memory pool implementation
struct MemoryPool {
    SlabAllocator* slab;
    size_t object_size;
    pthread_mutex_t lock;
};

MemoryPool* pool_create_typed(size_t object_size, size_t initial_count) {
    if (object_size == 0) return NULL;
    
    MemoryPool* pool = (MemoryPool*)malloc(sizeof(MemoryPool));
    if (!pool) return NULL;
    
    pool->object_size = object_size;
    pool->slab = slab_create(object_size, initial_count);
    if (!pool->slab) {
        free(pool);
        return NULL;
    }
    
    pthread_mutex_init(&pool->lock, NULL);
    return pool;
}

void pool_destroy_typed(MemoryPool* pool) {
    if (!pool) return;
    
    slab_destroy(pool->slab);
    pthread_mutex_destroy(&pool->lock);
    free(pool);
}

void* pool_get_object(MemoryPool* pool) {
    if (!pool) return NULL;
    
    pthread_mutex_lock(&pool->lock);
    void* obj = slab_alloc(pool->slab);
    pthread_mutex_unlock(&pool->lock);
    
    return obj;
}

void pool_return_object(MemoryPool* pool, void* obj) {
    if (!pool || !obj) return;
    
    pthread_mutex_lock(&pool->lock);
    slab_free(pool->slab, obj);
    pthread_mutex_unlock(&pool->lock);
}