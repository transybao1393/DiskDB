#include "../include/memory_pool.h"
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <assert.h>

// Slab header for tracking allocations
typedef struct Slab {
    struct Slab* next;
    size_t used_count;
    size_t total_objects;
    uint8_t* bitmap;  // Allocation bitmap
    char data[];      // Object storage
} Slab;

struct SlabAllocator {
    size_t object_size;
    size_t objects_per_slab;
    size_t slab_size;
    
    Slab* partial_slabs;  // Slabs with free space
    Slab* full_slabs;     // Completely allocated slabs
    Slab* empty_slabs;    // Completely free slabs (cache)
    
    size_t empty_slab_count;
    size_t max_empty_slabs;
    
    pthread_mutex_t lock;
    
    // Statistics
    uint64_t allocations;
    uint64_t deallocations;
    uint64_t slab_allocations;
};

// Helper to find first free bit in bitmap
static int find_free_bit(uint8_t* bitmap, size_t bits) {
    size_t bytes = (bits + 7) / 8;
    for (size_t i = 0; i < bytes; i++) {
        if (bitmap[i] != 0xFF) {
            // Found a byte with free bit
            for (int bit = 0; bit < 8 && (i * 8 + bit) < bits; bit++) {
                if (!(bitmap[i] & (1 << bit))) {
                    return i * 8 + bit;
                }
            }
        }
    }
    return -1;
}

// Set bit in bitmap
static void set_bit(uint8_t* bitmap, int index) {
    bitmap[index / 8] |= (1 << (index % 8));
}

// Clear bit in bitmap
static void clear_bit(uint8_t* bitmap, int index) {
    bitmap[index / 8] &= ~(1 << (index % 8));
}

// Check if bit is set
static bool is_bit_set(uint8_t* bitmap, int index) {
    return (bitmap[index / 8] & (1 << (index % 8))) != 0;
}

// Allocate a new slab
static Slab* allocate_slab(SlabAllocator* allocator) {
    size_t bitmap_size = (allocator->objects_per_slab + 7) / 8;
    size_t header_size = sizeof(Slab) + bitmap_size;
    size_t total_size = header_size + allocator->slab_size;
    
    Slab* slab = (Slab*)malloc(total_size);
    if (!slab) return NULL;
    
    slab->next = NULL;
    slab->used_count = 0;
    slab->total_objects = allocator->objects_per_slab;
    slab->bitmap = (uint8_t*)((char*)slab + sizeof(Slab));
    memset(slab->bitmap, 0, bitmap_size);
    
    allocator->slab_allocations++;
    
    return slab;
}

// Move slab between lists
static void move_slab(Slab** from, Slab** to, Slab* slab) {
    // Remove from source list
    if (*from == slab) {
        *from = slab->next;
    } else {
        Slab* prev = *from;
        while (prev && prev->next != slab) {
            prev = prev->next;
        }
        if (prev) {
            prev->next = slab->next;
        }
    }
    
    // Add to destination list
    slab->next = *to;
    *to = slab;
}

SlabAllocator* slab_create(size_t object_size, size_t objects_per_slab) {
    if (object_size == 0 || objects_per_slab == 0) return NULL;
    
    SlabAllocator* allocator = (SlabAllocator*)calloc(1, sizeof(SlabAllocator));
    if (!allocator) return NULL;
    
    // Align object size to 8 bytes
    allocator->object_size = (object_size + 7) & ~7;
    allocator->objects_per_slab = objects_per_slab;
    allocator->slab_size = allocator->object_size * objects_per_slab;
    allocator->max_empty_slabs = 2;  // Keep up to 2 empty slabs
    
    pthread_mutex_init(&allocator->lock, NULL);
    
    return allocator;
}

void slab_destroy(SlabAllocator* allocator) {
    if (!allocator) return;
    
    pthread_mutex_lock(&allocator->lock);
    
    // Free all slabs
    Slab* slabs[] = {allocator->partial_slabs, allocator->full_slabs, allocator->empty_slabs};
    for (int i = 0; i < 3; i++) {
        Slab* slab = slabs[i];
        while (slab) {
            Slab* next = slab->next;
            free(slab);
            slab = next;
        }
    }
    
    pthread_mutex_unlock(&allocator->lock);
    pthread_mutex_destroy(&allocator->lock);
    
    free(allocator);
}

void* slab_alloc(SlabAllocator* allocator) {
    if (!allocator) return NULL;
    
    pthread_mutex_lock(&allocator->lock);
    
    // Try to allocate from partial slab
    Slab* slab = allocator->partial_slabs;
    
    if (!slab) {
        // No partial slabs, try empty slabs
        if (allocator->empty_slabs) {
            slab = allocator->empty_slabs;
            allocator->empty_slabs = slab->next;
            slab->next = allocator->partial_slabs;
            allocator->partial_slabs = slab;
            allocator->empty_slab_count--;
        } else {
            // Allocate new slab
            slab = allocate_slab(allocator);
            if (!slab) {
                pthread_mutex_unlock(&allocator->lock);
                return NULL;
            }
            slab->next = allocator->partial_slabs;
            allocator->partial_slabs = slab;
        }
    }
    
    // Find free object in slab
    int index = find_free_bit(slab->bitmap, slab->total_objects);
    assert(index >= 0);  // Should always find free bit in partial slab
    
    set_bit(slab->bitmap, index);
    slab->used_count++;
    
    // Calculate object address
    size_t bitmap_size = (allocator->objects_per_slab + 7) / 8;
    char* data_start = (char*)slab + sizeof(Slab) + bitmap_size;
    void* object = data_start + (index * allocator->object_size);
    
    // Move to full list if necessary
    if (slab->used_count == slab->total_objects) {
        move_slab(&allocator->partial_slabs, &allocator->full_slabs, slab);
    }
    
    allocator->allocations++;
    pthread_mutex_unlock(&allocator->lock);
    
    return object;
}

void slab_free(SlabAllocator* allocator, void* ptr) {
    if (!allocator || !ptr) return;
    
    pthread_mutex_lock(&allocator->lock);
    
    // Find which slab contains this pointer
    Slab* slab = NULL;
    Slab** list = NULL;
    
    // Check all slab lists
    Slab* lists[] = {allocator->partial_slabs, allocator->full_slabs};
    Slab** list_ptrs[] = {&allocator->partial_slabs, &allocator->full_slabs};
    
    for (int i = 0; i < 2; i++) {
        Slab* current = lists[i];
        while (current) {
            size_t bitmap_size = (allocator->objects_per_slab + 7) / 8;
            char* data_start = (char*)current + sizeof(Slab) + bitmap_size;
            char* data_end = data_start + allocator->slab_size;
            
            if ((char*)ptr >= data_start && (char*)ptr < data_end) {
                slab = current;
                list = list_ptrs[i];
                break;
            }
            current = current->next;
        }
        if (slab) break;
    }
    
    if (!slab) {
        // Pointer not from this allocator
        pthread_mutex_unlock(&allocator->lock);
        return;
    }
    
    // Calculate object index
    size_t bitmap_size = (allocator->objects_per_slab + 7) / 8;
    char* data_start = (char*)slab + sizeof(Slab) + bitmap_size;
    int index = ((char*)ptr - data_start) / allocator->object_size;
    
    assert(index >= 0 && (size_t)index < slab->total_objects);
    assert(is_bit_set(slab->bitmap, index));
    
    clear_bit(slab->bitmap, index);
    slab->used_count--;
    
    // Move slab between lists if necessary
    if (slab->used_count == 0) {
        // Slab is now empty
        if (allocator->empty_slab_count < allocator->max_empty_slabs) {
            move_slab(list, &allocator->empty_slabs, slab);
            allocator->empty_slab_count++;
        } else {
            // Too many empty slabs, free this one
            if (*list == slab) {
                *list = slab->next;
            } else {
                Slab* prev = *list;
                while (prev && prev->next != slab) {
                    prev = prev->next;
                }
                if (prev) {
                    prev->next = slab->next;
                }
            }
            free(slab);
        }
    } else if (slab->used_count == slab->total_objects - 1 && list == &allocator->full_slabs) {
        // Was full, now partial
        move_slab(&allocator->full_slabs, &allocator->partial_slabs, slab);
    }
    
    allocator->deallocations++;
    pthread_mutex_unlock(&allocator->lock);
}

size_t slab_get_object_size(const SlabAllocator* allocator) {
    return allocator ? allocator->object_size : 0;
}