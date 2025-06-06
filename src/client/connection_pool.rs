use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;
use crate::error::{Result, DiskDBError};

const DEFAULT_POOL_SIZE: usize = 10;
const DEFAULT_MIN_CONNECTIONS: usize = 2;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const IDLE_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

/// A pooled connection with metadata
struct PooledConnection {
    stream: TcpStream,
    created_at: Instant,
    last_used: Instant,
}

impl PooledConnection {
    fn new(stream: TcpStream) -> Self {
        let now = Instant::now();
        Self {
            stream,
            created_at: now,
            last_used: now,
        }
    }
    
    fn is_stale(&self) -> bool {
        self.last_used.elapsed() > IDLE_TIMEOUT
    }
    
    fn touch(&mut self) {
        self.last_used = Instant::now();
    }
}

/// Connection pool for DiskDB clients
pub struct ConnectionPool {
    addr: SocketAddr,
    connections: Arc<Mutex<VecDeque<PooledConnection>>>,
    semaphore: Arc<Semaphore>,
    max_size: usize,
    min_connections: usize,
}

impl ConnectionPool {
    /// Create a new connection pool
    pub fn new(addr: SocketAddr) -> Self {
        Self::with_config(addr, DEFAULT_POOL_SIZE, DEFAULT_MIN_CONNECTIONS)
    }
    
    /// Create a connection pool with custom configuration
    pub fn with_config(addr: SocketAddr, max_size: usize, min_connections: usize) -> Self {
        let pool = Self {
            addr,
            connections: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            semaphore: Arc::new(Semaphore::new(max_size)),
            max_size,
            min_connections: min_connections.min(max_size),
        };
        
        // Pre-warm the pool
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            pool_clone.warm_pool().await;
        });
        
        pool
    }
    
    /// Get a connection from the pool
    pub async fn get(&self) -> Result<PooledTcpStream> {
        // Acquire permit
        let permit = self.semaphore.clone().acquire_owned().await
            .map_err(|_| DiskDBError::Protocol("Failed to acquire connection permit".to_string()))?;
        
        // Try to get existing connection
        let mut connections = self.connections.lock().await;
        
        // Remove stale connections
        connections.retain(|conn| !conn.is_stale());
        
        if let Some(mut conn) = connections.pop_front() {
            conn.touch();
            drop(connections);
            
            // Verify connection is still alive
            if Self::is_connection_alive(&conn.stream).await {
                return Ok(PooledTcpStream {
                    stream: Some(conn.stream),
                    pool: self.connections.clone(),
                    _permit: permit,
                });
            }
        }
        
        // Create new connection
        let stream = self.create_connection().await?;
        Ok(PooledTcpStream {
            stream: Some(stream),
            pool: self.connections.clone(),
            _permit: permit,
        })
    }
    
    /// Create a new connection
    async fn create_connection(&self) -> Result<TcpStream> {
        match timeout(CONNECTION_TIMEOUT, TcpStream::connect(self.addr)).await {
            Ok(Ok(stream)) => {
                // Set TCP_NODELAY
                stream.set_nodelay(true)?;
                Ok(stream)
            }
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err(DiskDBError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Connection timeout",
            ))),
        }
    }
    
    /// Check if a connection is still alive
    async fn is_connection_alive(stream: &TcpStream) -> bool {
        // Try to read with zero timeout - if connection is closed, it will return immediately
        let mut buf = [0u8; 1];
        match timeout(Duration::from_millis(1), stream.peek(&mut buf)).await {
            Ok(Ok(0)) => false, // Connection closed
            Ok(Ok(_)) => true,  // Data available
            Ok(Err(_)) => false, // Error
            Err(_) => true,      // Timeout means connection is likely alive
        }
    }
    
    /// Pre-warm the connection pool
    async fn warm_pool(&self) {
        let mut connections = Vec::new();
        
        for _ in 0..self.min_connections {
            match self.create_connection().await {
                Ok(stream) => connections.push(PooledConnection::new(stream)),
                Err(e) => {
                    log::warn!("Failed to pre-warm connection: {}", e);
                    break;
                }
            }
        }
        
        if !connections.is_empty() {
            let mut pool = self.connections.lock().await;
            pool.extend(connections);
        }
    }
    
    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let connections = self.connections.lock().await;
        let active_connections = connections.len();
        let available_permits = self.semaphore.available_permits();
        
        PoolStats {
            active_connections,
            idle_connections: active_connections,
            total_capacity: self.max_size,
            available_permits,
        }
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            addr: self.addr,
            connections: self.connections.clone(),
            semaphore: self.semaphore.clone(),
            max_size: self.max_size,
            min_connections: self.min_connections,
        }
    }
}

/// A TCP stream that returns to the pool when dropped
pub struct PooledTcpStream {
    stream: Option<TcpStream>,
    pool: Arc<Mutex<VecDeque<PooledConnection>>>,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl PooledTcpStream {
    /// Get a reference to the inner stream
    pub fn stream(&self) -> &TcpStream {
        self.stream.as_ref().expect("Stream already taken")
    }
    
    /// Get a mutable reference to the inner stream
    pub fn stream_mut(&mut self) -> &mut TcpStream {
        self.stream.as_mut().expect("Stream already taken")
    }
    
    /// Take ownership of the stream (removes from pool)
    pub fn into_inner(mut self) -> TcpStream {
        self.stream.take().expect("Stream already taken")
    }
}

impl Drop for PooledTcpStream {
    fn drop(&mut self) {
        if let Some(stream) = self.stream.take() {
            // Return to pool if healthy
            let pool = self.pool.clone();
            tokio::spawn(async move {
                if ConnectionPool::is_connection_alive(&stream).await {
                    let mut connections = pool.lock().await;
                    connections.push_back(PooledConnection::new(stream));
                }
            });
        }
    }
}

#[derive(Debug)]
pub struct PoolStats {
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_capacity: usize,
    pub available_permits: usize,
}