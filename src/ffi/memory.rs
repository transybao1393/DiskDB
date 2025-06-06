use std::ffi::{c_char, c_void};
use std::ptr::NonNull;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::slice;
use crate::error::{Result, DiskDBError};

// FFI bindings
#[repr(C)]
pub struct MemoryPoolConfig {
    pub initial_pool_size: usize,
    pub max_pool_size: usize,
    pub thread_cache_size: usize,
    pub enable_statistics: bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct MemoryStats {
    pub allocations: u64,
    pub deallocations: u64,
    pub bytes_allocated: u64,
    pub bytes_freed: u64,
    pub pool_hits: u64,
    pub pool_misses: u64,
    pub active_objects: u64,
}

extern "C" {
    fn memory_pool_init(config: *const MemoryPoolConfig) -> i32;
    fn memory_pool_shutdown();
    fn pool_alloc(size: usize) -> *mut c_void;
    fn pool_calloc(count: usize, size: usize) -> *mut c_void;
    fn pool_free(ptr: *mut c_void, size: usize);
    fn pool_realloc(ptr: *mut c_void, old_size: usize, new_size: usize) -> *mut c_void;
    fn pool_strdup(str: *const c_char) -> *mut c_char;
    fn pool_get_stats(stats: *mut MemoryStats);
    fn pool_reset_stats();
    fn tls_pool_clear();
}

// Initialize memory pool system
static INIT: std::sync::Once = std::sync::Once::new();

pub fn init_memory_pool() -> Result<()> {
    let mut result = Ok(());
    
    INIT.call_once(|| {
        let config = MemoryPoolConfig {
            initial_pool_size: 1024 * 1024,     // 1MB
            max_pool_size: 16 * 1024 * 1024,    // 16MB
            thread_cache_size: 8,
            enable_statistics: true,
        };
        
        unsafe {
            if memory_pool_init(&config) != 0 {
                result = Err(DiskDBError::Database("Failed to initialize memory pool".into()));
            }
        }
    });
    
    result
}

// RAII wrapper for pooled memory
pub struct PooledBox<T> {
    ptr: NonNull<T>,
    size: usize,
    _phantom: PhantomData<T>,
}

impl<T> PooledBox<T> {
    /// Allocate new object from pool
    pub fn new(value: T) -> Result<Self> {
        init_memory_pool()?;
        
        unsafe {
            let size = std::mem::size_of::<T>();
            let ptr = pool_alloc(size) as *mut T;
            if ptr.is_null() {
                return Err(DiskDBError::Database("Memory allocation failed".into()));
            }
            
            // Write value
            ptr.write(value);
            
            Ok(PooledBox {
                ptr: NonNull::new_unchecked(ptr),
                size,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Create uninitialized box
    pub unsafe fn new_uninit() -> Result<Self> {
        init_memory_pool()?;
        
        let size = std::mem::size_of::<T>();
        let ptr = pool_alloc(size) as *mut T;
        if ptr.is_null() {
            return Err(DiskDBError::Database("Memory allocation failed".into()));
        }
        
        Ok(PooledBox {
            ptr: NonNull::new_unchecked(ptr),
            size,
            _phantom: PhantomData,
        })
    }
}

impl<T> Deref for PooledBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for PooledBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for PooledBox<T> {
    fn drop(&mut self) {
        unsafe {
            // Drop the value
            std::ptr::drop_in_place(self.ptr.as_ptr());
            // Return memory to pool
            pool_free(self.ptr.as_ptr() as *mut c_void, self.size);
        }
    }
}

// RAII wrapper for pooled Vec
pub struct PooledVec<T> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
    _phantom: PhantomData<T>,
}

impl<T> PooledVec<T> {
    /// Create new empty vec
    pub fn new() -> Self {
        PooledVec {
            ptr: NonNull::dangling(),
            len: 0,
            capacity: 0,
            _phantom: PhantomData,
        }
    }
    
    /// Create vec with capacity
    pub fn with_capacity(capacity: usize) -> Result<Self> {
        if capacity == 0 {
            return Ok(Self::new());
        }
        
        init_memory_pool()?;
        
        unsafe {
            let size = capacity * std::mem::size_of::<T>();
            let ptr = pool_alloc(size) as *mut T;
            if ptr.is_null() {
                return Err(DiskDBError::Database("Memory allocation failed".into()));
            }
            
            Ok(PooledVec {
                ptr: NonNull::new_unchecked(ptr),
                len: 0,
                capacity,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Push element
    pub fn push(&mut self, value: T) -> Result<()> {
        if self.len == self.capacity {
            self.grow()?;
        }
        
        unsafe {
            let ptr = self.ptr.as_ptr().add(self.len);
            ptr.write(value);
            self.len += 1;
        }
        
        Ok(())
    }
    
    /// Pop element
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                let ptr = self.ptr.as_ptr().add(self.len);
                Some(ptr.read())
            }
        }
    }
    
    /// Get length
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// Clear vec
    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }
    
    /// Convert to slice
    pub fn as_slice(&self) -> &[T] {
        if self.len == 0 {
            &[]
        } else {
            unsafe {
                slice::from_raw_parts(self.ptr.as_ptr(), self.len)
            }
        }
    }
    
    /// Convert to mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.len == 0 {
            &mut []
        } else {
            unsafe {
                slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
            }
        }
    }
    
    // Grow capacity
    fn grow(&mut self) -> Result<()> {
        let new_capacity = if self.capacity == 0 {
            4
        } else {
            self.capacity * 2
        };
        
        unsafe {
            let old_size = self.capacity * std::mem::size_of::<T>();
            let new_size = new_capacity * std::mem::size_of::<T>();
            
            let new_ptr = if self.capacity == 0 {
                pool_alloc(new_size)
            } else {
                pool_realloc(self.ptr.as_ptr() as *mut c_void, old_size, new_size)
            } as *mut T;
            
            if new_ptr.is_null() {
                return Err(DiskDBError::Database("Memory reallocation failed".into()));
            }
            
            self.ptr = NonNull::new_unchecked(new_ptr);
            self.capacity = new_capacity;
        }
        
        Ok(())
    }
}

impl<T> Drop for PooledVec<T> {
    fn drop(&mut self) {
        if self.capacity > 0 {
            // Drop all elements
            self.clear();
            
            unsafe {
                let size = self.capacity * std::mem::size_of::<T>();
                pool_free(self.ptr.as_ptr() as *mut c_void, size);
            }
        }
    }
}

impl<T> Deref for PooledVec<T> {
    type Target = [T];
    
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> DerefMut for PooledVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

// Pooled string
#[derive(Clone)]
pub struct PooledString {
    ptr: NonNull<c_char>,
    len: usize,
}

impl PooledString {
    /// Create from string slice
    pub fn from_str(s: &str) -> Result<Self> {
        init_memory_pool()?;
        
        unsafe {
            let c_str = std::ffi::CString::new(s)
                .map_err(|_| DiskDBError::Database("Invalid string".into()))?;
            let ptr = pool_strdup(c_str.as_ptr());
            
            if ptr.is_null() {
                return Err(DiskDBError::Database("Memory allocation failed".into()));
            }
            
            Ok(PooledString {
                ptr: NonNull::new_unchecked(ptr),
                len: s.len(),
            })
        }
    }
    
    /// Get as string slice
    pub fn as_str(&self) -> &str {
        unsafe {
            let bytes = slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.len);
            std::str::from_utf8_unchecked(bytes)
        }
    }
}

impl Drop for PooledString {
    fn drop(&mut self) {
        unsafe {
            pool_free(self.ptr.as_ptr() as *mut c_void, self.len + 1);
        }
    }
}

impl Deref for PooledString {
    type Target = str;
    
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

// Implement standard traits for PooledString
impl std::fmt::Debug for PooledString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl std::fmt::Display for PooledString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq for PooledString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for PooledString {}

impl std::hash::Hash for PooledString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialOrd for PooledString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for PooledString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

// Implement Debug for PooledVec
impl<T: std::fmt::Debug> std::fmt::Debug for PooledVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

// Implement Clone for PooledVec
impl<T: Clone> Clone for PooledVec<T> {
    fn clone(&self) -> Self {
        if self.capacity == 0 {
            return Self::new();
        }
        
        let mut new_vec = Self::with_capacity(self.capacity).unwrap();
        for item in self.as_slice() {
            new_vec.push(item.clone()).unwrap();
        }
        new_vec
    }
}

// Implement Debug for PooledBox
impl<T: std::fmt::Debug> std::fmt::Debug for PooledBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

// Implement Clone for PooledBox where T: Clone
impl<T: Clone> Clone for PooledBox<T> {
    fn clone(&self) -> Self {
        Self::new((**self).clone()).unwrap()
    }
}

// Get memory statistics
pub fn get_memory_stats() -> MemoryStats {
    let mut stats = MemoryStats {
        allocations: 0,
        deallocations: 0,
        bytes_allocated: 0,
        bytes_freed: 0,
        pool_hits: 0,
        pool_misses: 0,
        active_objects: 0,
    };
    
    unsafe {
        pool_get_stats(&mut stats);
    }
    
    stats
}

// Reset statistics
pub fn reset_memory_stats() {
    unsafe {
        pool_reset_stats();
    }
}

// Clear thread-local cache
pub fn clear_thread_cache() {
    unsafe {
        tls_pool_clear();
    }
}

// Shutdown hook
pub struct MemoryPoolGuard;

impl Drop for MemoryPoolGuard {
    fn drop(&mut self) {
        unsafe {
            memory_pool_shutdown();
        }
    }
}

// Global guard to ensure cleanup
lazy_static::lazy_static! {
    static ref MEMORY_POOL_GUARD: MemoryPoolGuard = MemoryPoolGuard;
}

// Force initialization
pub fn ensure_initialized() {
    lazy_static::initialize(&MEMORY_POOL_GUARD);
}