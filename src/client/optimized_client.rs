use crate::client::connection_pool::ConnectionPool;
use crate::error::{Result, DiskDBError};
use crate::protocol::{Request, Response};
use crate::network::buffer_pool::GLOBAL_BUFFER_POOL;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Optimized DiskDB client with connection pooling and pipelining
pub struct OptimizedClient {
    pool: Arc<ConnectionPool>,
    pipeline_enabled: bool,
    pipeline_buffer: Arc<Mutex<Vec<Request>>>,
    max_pipeline_size: usize,
}

impl OptimizedClient {
    /// Create a new optimized client
    pub async fn connect(addr: &str) -> Result<Self> {
        let addr: SocketAddr = addr.parse()
            .map_err(|e| DiskDBError::Config(format!("Invalid address: {}", e)))?;
        
        let pool = Arc::new(ConnectionPool::new(addr));
        
        Ok(Self {
            pool,
            pipeline_enabled: true,
            pipeline_buffer: Arc::new(Mutex::new(Vec::with_capacity(100))),
            max_pipeline_size: 100,
        })
    }
    
    /// Create client with custom pool configuration
    pub async fn connect_with_pool(addr: &str, pool_size: usize, min_connections: usize) -> Result<Self> {
        let addr: SocketAddr = addr.parse()
            .map_err(|e| DiskDBError::Config(format!("Invalid address: {}", e)))?;
        
        let pool = Arc::new(ConnectionPool::with_config(addr, pool_size, min_connections));
        
        Ok(Self {
            pool,
            pipeline_enabled: true,
            pipeline_buffer: Arc::new(Mutex::new(Vec::with_capacity(100))),
            max_pipeline_size: 100,
        })
    }
    
    /// Execute a single command
    pub async fn execute(&self, request: Request) -> Result<Response> {
        if self.pipeline_enabled {
            // Add to pipeline
            let mut buffer = self.pipeline_buffer.lock().await;
            buffer.push(request.clone());
            
            // Execute pipeline if full or if this is a special command
            if buffer.len() >= self.max_pipeline_size || self.should_flush(&request) {
                drop(buffer);
                let responses = self.flush_pipeline().await?;
                // Return the last response (current request)
                responses.into_iter().last()
                    .ok_or_else(|| DiskDBError::Protocol("No response received".to_string()))
            } else {
                drop(buffer);
                // For now, execute immediately
                // In a real implementation, we might wait for more commands
                self.execute_single(request).await
            }
        } else {
            self.execute_single(request).await
        }
    }
    
    /// Execute a single request without pipelining
    async fn execute_single(&self, request: Request) -> Result<Response> {
        let mut conn = self.pool.get().await?;
        let stream = conn.stream_mut();
        
        // Get buffer from pool
        let mut write_buffer = GLOBAL_BUFFER_POOL.get(256).await;
        write_buffer.as_mut().extend_from_slice(request.to_string().as_bytes());
        write_buffer.as_mut().extend_from_slice(b"\n");
        
        // Send request
        match timeout(REQUEST_TIMEOUT, stream.write_all(write_buffer.as_mut())).await {
            Ok(Ok(_)) => {},
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => return Err(DiskDBError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Request timeout",
            ))),
        }
        
        // Read response
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        
        match timeout(REQUEST_TIMEOUT, reader.read_line(&mut line)).await {
            Ok(Ok(0)) => Err(DiskDBError::ConnectionClosed),
            Ok(Ok(_)) => Response::parse(&line),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err(DiskDBError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Response timeout",
            ))),
        }
    }
    
    /// Execute multiple requests in a pipeline
    pub async fn execute_pipeline(&self, requests: Vec<Request>) -> Result<Vec<Response>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut conn = self.pool.get().await?;
        let stream = conn.stream_mut();
        
        // Build request buffer
        let mut write_buffer = GLOBAL_BUFFER_POOL.get(4096).await;
        for request in &requests {
            write_buffer.as_mut().extend_from_slice(request.to_string().as_bytes());
            write_buffer.as_mut().extend_from_slice(b"\n");
        }
        
        // Send all requests
        match timeout(REQUEST_TIMEOUT, stream.write_all(write_buffer.as_mut())).await {
            Ok(Ok(_)) => {},
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => return Err(DiskDBError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Pipeline request timeout",
            ))),
        }
        
        // Read all responses
        let mut reader = BufReader::new(stream);
        let mut responses = Vec::with_capacity(requests.len());
        
        for _ in 0..requests.len() {
            let mut line = String::new();
            match timeout(REQUEST_TIMEOUT, reader.read_line(&mut line)).await {
                Ok(Ok(0)) => return Err(DiskDBError::ConnectionClosed),
                Ok(Ok(_)) => {
                    responses.push(Response::parse(&line)?);
                }
                Ok(Err(e)) => return Err(e.into()),
                Err(_) => return Err(DiskDBError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Pipeline response timeout",
                ))),
            }
        }
        
        Ok(responses)
    }
    
    /// Flush the pipeline buffer
    async fn flush_pipeline(&self) -> Result<Vec<Response>> {
        let mut buffer = self.pipeline_buffer.lock().await;
        if buffer.is_empty() {
            return Ok(Vec::new());
        }
        
        let requests: Vec<Request> = buffer.drain(..).collect();
        drop(buffer);
        
        self.execute_pipeline(requests).await
    }
    
    /// Check if request should trigger pipeline flush
    fn should_flush(&self, request: &Request) -> bool {
        matches!(request, 
            Request::FlushDb | 
            Request::Info | 
            Request::Ping
        )
    }
    
    /// Enable or disable pipelining
    pub fn set_pipeline_enabled(&mut self, enabled: bool) {
        self.pipeline_enabled = enabled;
    }
    
    /// Set maximum pipeline size
    pub fn set_max_pipeline_size(&mut self, size: usize) {
        self.max_pipeline_size = size;
    }
    
    /// Get connection pool statistics
    pub async fn pool_stats(&self) -> crate::client::connection_pool::PoolStats {
        self.pool.stats().await
    }
    
    /// Close all connections
    pub async fn close(&self) -> Result<()> {
        // Flush any pending pipeline requests
        let _ = self.flush_pipeline().await;
        Ok(())
    }
}

// Convenience methods for common operations
impl OptimizedClient {
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let response = self.execute(Request::Get { key: key.to_string() }).await?;
        match response {
            Response::String(value) => Ok(value),
            Response::Null => Ok(None),
            _ => Err(DiskDBError::Protocol("Unexpected response type".to_string())),
        }
    }
    
    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        let response = self.execute(Request::Set { 
            key: key.to_string(), 
            value: value.to_string() 
        }).await?;
        
        match response {
            Response::Ok => Ok(()),
            Response::Error(e) => Err(DiskDBError::Protocol(e)),
            _ => Err(DiskDBError::Protocol("Unexpected response type".to_string())),
        }
    }
    
    pub async fn ping(&self) -> Result<bool> {
        let response = self.execute(Request::Ping).await?;
        match response {
            Response::String(Some(s)) => Ok(s == "PONG"),
            _ => Ok(false),
        }
    }
}