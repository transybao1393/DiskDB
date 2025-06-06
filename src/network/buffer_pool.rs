use bytes::{Bytes, BytesMut};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

/// Buffer sizes for different use cases
#[derive(Debug, Clone, Copy)]
pub enum BufferSize {
    Small = 512,      // For small responses
    Medium = 4096,    // For typical responses
    Large = 65536,    // For bulk operations
}

impl BufferSize {
    fn as_usize(self) -> usize {
        self as usize
    }
    
    fn from_size(size: usize) -> Self {
        if size <= BufferSize::Small.as_usize() {
            BufferSize::Small
        } else if size <= BufferSize::Medium.as_usize() {
            BufferSize::Medium
        } else {
            BufferSize::Large
        }
    }
}

/// A pool of reusable buffers to reduce allocation overhead
pub struct BufferPool {
    small_pool: Arc<Mutex<VecDeque<BytesMut>>>,
    medium_pool: Arc<Mutex<VecDeque<BytesMut>>>,
    large_pool: Arc<Mutex<VecDeque<BytesMut>>>,
    
    // Limits to prevent unbounded growth
    max_small: usize,
    max_medium: usize,
    max_large: usize,
    
    // Semaphores to limit total memory usage
    small_sem: Arc<Semaphore>,
    medium_sem: Arc<Semaphore>,
    large_sem: Arc<Semaphore>,
}

impl BufferPool {
    /// Create a new buffer pool with default limits
    pub fn new() -> Self {
        Self::with_limits(1000, 500, 100)
    }
    
    /// Create a buffer pool with custom limits
    pub fn with_limits(max_small: usize, max_medium: usize, max_large: usize) -> Self {
        Self {
            small_pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_small))),
            medium_pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_medium))),
            large_pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_large))),
            max_small,
            max_medium,
            max_large,
            small_sem: Arc::new(Semaphore::new(max_small)),
            medium_sem: Arc::new(Semaphore::new(max_medium)),
            large_sem: Arc::new(Semaphore::new(max_large)),
        }
    }
    
    /// Get a buffer of at least the specified size
    pub async fn get(&self, min_size: usize) -> PooledBuffer {
        let size = BufferSize::from_size(min_size);
        
        let (pool, _sem, _max_size) = match size {
            BufferSize::Small => (&self.small_pool, &self.small_sem, self.max_small),
            BufferSize::Medium => (&self.medium_pool, &self.medium_sem, self.max_medium),
            BufferSize::Large => (&self.large_pool, &self.large_sem, self.max_large),
        };
        
        // Try to get from pool first
        if let Ok(mut guard) = pool.try_lock() {
            if let Some(mut buffer) = guard.pop_front() {
                buffer.clear();
                return PooledBuffer {
                    buffer,
                    pool: pool.clone(),
                    size,
                };
            }
        }
        
        // Allocate new buffer if pool is empty
        let buffer = BytesMut::with_capacity(size.as_usize());
        PooledBuffer {
            buffer,
            pool: pool.clone(),
            size,
        }
    }
    
    /// Pre-allocate buffers
    pub fn preallocate(&self, small: usize, medium: usize, large: usize) {
        if let Ok(mut pool) = self.small_pool.lock() {
            for _ in 0..small.min(self.max_small) {
                pool.push_back(BytesMut::with_capacity(BufferSize::Small.as_usize()));
            }
        }
        
        if let Ok(mut pool) = self.medium_pool.lock() {
            for _ in 0..medium.min(self.max_medium) {
                pool.push_back(BytesMut::with_capacity(BufferSize::Medium.as_usize()));
            }
        }
        
        if let Ok(mut pool) = self.large_pool.lock() {
            for _ in 0..large.min(self.max_large) {
                pool.push_back(BytesMut::with_capacity(BufferSize::Large.as_usize()));
            }
        }
    }
    
    /// Get current pool statistics
    pub fn stats(&self) -> BufferPoolStats {
        let small_count = self.small_pool.lock().ok().map(|p| p.len()).unwrap_or(0);
        let medium_count = self.medium_pool.lock().ok().map(|p| p.len()).unwrap_or(0);
        let large_count = self.large_pool.lock().ok().map(|p| p.len()).unwrap_or(0);
        
        BufferPoolStats {
            small_buffers: small_count,
            medium_buffers: medium_count,
            large_buffers: large_count,
            small_capacity: self.max_small,
            medium_capacity: self.max_medium,
            large_capacity: self.max_large,
        }
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

/// A buffer that returns to the pool when dropped
pub struct PooledBuffer {
    buffer: BytesMut,
    pool: Arc<Mutex<VecDeque<BytesMut>>>,
    size: BufferSize,
}

impl PooledBuffer {
    /// Get the underlying buffer
    pub fn as_mut(&mut self) -> &mut BytesMut {
        &mut self.buffer
    }
    
    /// Freeze the buffer into immutable Bytes
    pub fn freeze(mut self) -> Bytes {
        let buffer = std::mem::take(&mut self.buffer);
        std::mem::forget(self); // Prevent drop from running
        buffer.freeze()
    }
    
    /// Get current length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        // Only return to pool if not too large and pool has space
        if self.buffer.capacity() <= self.size.as_usize() * 2 {
            if let Ok(mut pool) = self.pool.lock() {
                // Only keep if pool isn't full
                let max_size = match self.size {
                    BufferSize::Small => 1000,
                    BufferSize::Medium => 500,
                    BufferSize::Large => 100,
                };
                
                if pool.len() < max_size {
                    self.buffer.clear();
                    pool.push_back(std::mem::take(&mut self.buffer));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct BufferPoolStats {
    pub small_buffers: usize,
    pub medium_buffers: usize,
    pub large_buffers: usize,
    pub small_capacity: usize,
    pub medium_capacity: usize,
    pub large_capacity: usize,
}

// Global buffer pool
lazy_static::lazy_static! {
    pub static ref GLOBAL_BUFFER_POOL: Arc<BufferPool> = {
        let pool = BufferPool::new();
        // Pre-allocate some buffers
        pool.preallocate(100, 50, 10);
        Arc::new(pool)
    };
}