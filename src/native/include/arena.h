#ifndef DISKDB_ARENA_H
#define DISKDB_ARENA_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Arena allocator for zero-copy parsing
typedef struct Arena {
    char* base;       // Base memory address
    size_t size;      // Total size
    size_t offset;    // Current offset
    uint64_t generation; // Generation counter for safety
} Arena;

// Create a new arena
Arena* arena_create(size_t size);

// Destroy arena
void arena_destroy(Arena* arena);

// Allocate memory from arena (very fast)
void* arena_alloc(Arena* arena, size_t size);

// Allocate aligned memory
void* arena_alloc_aligned(Arena* arena, size_t size, size_t alignment);

// Reset arena for reuse (O(1) operation)
void arena_reset(Arena* arena);

// Get remaining space
size_t arena_remaining(const Arena* arena);

// Thread-local arena management
Arena* get_thread_local_arena(void);
void set_thread_local_arena(Arena* arena);

#ifdef __cplusplus
}
#endif

#endif // DISKDB_ARENA_H