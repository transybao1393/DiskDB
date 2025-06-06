#include "../include/arena.h"
#include <stdlib.h>
#include <string.h>
#include <assert.h>

// Thread-local storage for arena
static __thread Arena* tls_arena = NULL;

// Alignment helper
#define ALIGN_UP(x, align) (((x) + (align) - 1) & ~((align) - 1))

Arena* arena_create(size_t size) {
    Arena* arena = (Arena*)malloc(sizeof(Arena));
    if (!arena) return NULL;
    
    arena->base = (char*)malloc(size);
    if (!arena->base) {
        free(arena);
        return NULL;
    }
    
    arena->size = size;
    arena->offset = 0;
    arena->generation = 0;
    
    return arena;
}

void arena_destroy(Arena* arena) {
    if (!arena) return;
    
    free(arena->base);
    free(arena);
}

void* arena_alloc(Arena* arena, size_t size) {
    if (!arena || size == 0) return NULL;
    
    // Align to 8 bytes for better performance
    size = ALIGN_UP(size, 8);
    
    if (arena->offset + size > arena->size) {
        // Arena is full
        return NULL;
    }
    
    void* ptr = arena->base + arena->offset;
    arena->offset += size;
    
    return ptr;
}

void* arena_alloc_aligned(Arena* arena, size_t size, size_t alignment) {
    if (!arena || size == 0 || alignment == 0) return NULL;
    
    // Ensure alignment is power of 2
    assert((alignment & (alignment - 1)) == 0);
    
    // Align current offset
    size_t aligned_offset = ALIGN_UP(arena->offset, alignment);
    
    if (aligned_offset + size > arena->size) {
        return NULL;
    }
    
    void* ptr = arena->base + aligned_offset;
    arena->offset = aligned_offset + size;
    
    return ptr;
}

void arena_reset(Arena* arena) {
    if (!arena) return;
    
    arena->offset = 0;
    arena->generation++;
    
    // Optional: clear memory for security
    // memset(arena->base, 0, arena->size);
}

size_t arena_remaining(const Arena* arena) {
    if (!arena) return 0;
    
    return arena->size - arena->offset;
}

Arena* get_thread_local_arena(void) {
    return tls_arena;
}

void set_thread_local_arena(Arena* arena) {
    tls_arena = arena;
}