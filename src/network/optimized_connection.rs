use crate::commands::CommandExecutor;
use crate::error::{Result, DiskDBError};
use crate::network::buffer_pool::{BufferPool, GLOBAL_BUFFER_POOL};
use crate::protocol::{Request, Response};
use bytes::{BufMut, BytesMut};
use log::{error, info, trace};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_native_tls::TlsStream;

const READ_TIMEOUT: Duration = Duration::from_secs(30);
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_PIPELINE_DEPTH: usize = 100;

pub enum OptimizedConnection {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl OptimizedConnection {
    /// Create an optimized TCP connection with custom socket options
    pub async fn accept(stream: TcpStream, _addr: SocketAddr) -> Result<Self> {
        // Set TCP options for better performance
        let sock_ref = socket2::SockRef::from(&stream);
        
        // Enable TCP_NODELAY for low latency
        sock_ref.set_nodelay(true)?;
        
        // Set socket buffer sizes for better throughput
        let _ = sock_ref.set_recv_buffer_size(256 * 1024);
        let _ = sock_ref.set_send_buffer_size(256 * 1024);
        
        // Enable TCP keepalive
        sock_ref.set_keepalive(true)?;
        
        #[cfg(target_os = "linux")]
        {
            // Linux-specific optimizations
            use std::os::unix::io::AsRawFd;
            let fd = stream.as_raw_fd();
            
            // Enable TCP_QUICKACK for faster ACKs
            unsafe {
                let enable: i32 = 1;
                libc::setsockopt(
                    fd,
                    libc::IPPROTO_TCP,
                    12, // TCP_QUICKACK
                    &enable as *const _ as *const libc::c_void,
                    std::mem::size_of::<i32>() as libc::socklen_t,
                );
            }
        }
        
        Ok(OptimizedConnection::Plain(stream))
    }
    
    /// Handle the connection with optimizations
    pub async fn handle(
        self,
        executor: Arc<CommandExecutor>,
        addr: String,
        buffer_pool: Option<Arc<BufferPool>>,
    ) -> Result<()> {
        info!("Optimized connection from: {}", addr);
        
        let pool = buffer_pool.unwrap_or_else(|| GLOBAL_BUFFER_POOL.clone());
        
        match self {
            OptimizedConnection::Plain(stream) => {
                Self::handle_plain(stream, executor, addr, pool).await
            }
            OptimizedConnection::Tls(stream) => {
                Self::handle_tls(stream, executor, addr, pool).await
            }
        }
    }
    
    async fn handle_plain(
        stream: TcpStream,
        executor: Arc<CommandExecutor>,
        addr: String,
        buffer_pool: Arc<BufferPool>,
    ) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::with_capacity(64 * 1024, reader);
        
        // Pipeline support - collect multiple requests before responding
        let mut pipeline_buffer = Vec::with_capacity(MAX_PIPELINE_DEPTH);
        let mut response_buffer = buffer_pool.get(4096).await;
        
        loop {
            // Read with timeout
            let mut line = String::new();
            match timeout(READ_TIMEOUT, reader.read_line(&mut line)).await {
                Ok(Ok(0)) => break, // Connection closed
                Ok(Ok(_)) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    // Parse request
                    let request_result = Request::parse(&line);
                    pipeline_buffer.push((line.clone(), request_result));
                    
                    // Check if we should process the pipeline
                    if pipeline_buffer.len() >= MAX_PIPELINE_DEPTH || 
                       Self::should_flush_pipeline(&pipeline_buffer) {
                        Self::process_pipeline(
                            &mut pipeline_buffer,
                            &executor,
                            response_buffer.as_mut(),
                            &mut writer,
                            &buffer_pool,
                        ).await?;
                    }
                }
                Ok(Err(e)) => {
                    error!("Read error from {}: {}", addr, e);
                    break;
                }
                Err(_) => {
                    error!("Read timeout from {}", addr);
                    break;
                }
            }
        }
        
        // Process any remaining requests
        if !pipeline_buffer.is_empty() {
            Self::process_pipeline(
                &mut pipeline_buffer,
                &executor,
                response_buffer.as_mut(),
                &mut writer,
                &buffer_pool,
            ).await?;
        }
        
        info!("Optimized connection closed: {}", addr);
        Ok(())
    }
    
    async fn handle_tls(
        stream: TlsStream<TcpStream>,
        executor: Arc<CommandExecutor>,
        addr: String,
        buffer_pool: Arc<BufferPool>,
    ) -> Result<()> {
        // Similar to plain but with TLS stream
        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::with_capacity(64 * 1024, reader);
        
        let mut pipeline_buffer = Vec::with_capacity(MAX_PIPELINE_DEPTH);
        let mut response_buffer = buffer_pool.get(4096).await;
        
        loop {
            let mut line = String::new();
            match timeout(READ_TIMEOUT, reader.read_line(&mut line)).await {
                Ok(Ok(0)) => break,
                Ok(Ok(_)) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    let request_result = Request::parse(&line);
                    pipeline_buffer.push((line.clone(), request_result));
                    
                    if pipeline_buffer.len() >= MAX_PIPELINE_DEPTH || 
                       Self::should_flush_pipeline(&pipeline_buffer) {
                        // For TLS, we can't use vectored I/O efficiently
                        Self::process_pipeline_tls(
                            &mut pipeline_buffer,
                            &executor,
                            response_buffer.as_mut(),
                            &mut writer,
                        ).await?;
                    }
                }
                Ok(Err(e)) => {
                    error!("Read error from {}: {}", addr, e);
                    break;
                }
                Err(_) => {
                    error!("Read timeout from {}", addr);
                    break;
                }
            }
        }
        
        if !pipeline_buffer.is_empty() {
            Self::process_pipeline_tls(
                &mut pipeline_buffer,
                &executor,
                response_buffer.as_mut(),
                &mut writer,
            ).await?;
        }
        
        info!("TLS connection closed: {}", addr);
        Ok(())
    }
    
    fn should_flush_pipeline(pipeline: &[(String, Result<Request>)]) -> bool {
        // Flush if we have any errors or special commands
        pipeline.iter().any(|(_, result)| {
            match result {
                Err(_) => true,
                Ok(req) => matches!(req, 
                    Request::FlushDb | 
                    Request::Info | 
                    Request::Ping
                ),
            }
        })
    }
    
    async fn process_pipeline(
        pipeline: &mut Vec<(String, Result<Request>)>,
        executor: &Arc<CommandExecutor>,
        response_buffer: &mut BytesMut,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        _buffer_pool: &Arc<BufferPool>,
    ) -> Result<()> {
        response_buffer.clear();
        
        // Process all requests and build responses
        for (_, request_result) in pipeline.iter() {
            let response = match request_result {
                Ok(request) => {
                    match executor.execute(request.clone()).await {
                        Ok(resp) => resp,
                        Err(e) => Response::Error(e.to_string()),
                    }
                }
                Err(e) => Response::Error(e.to_string()),
            };
            
            // Write response to buffer
            response_buffer.put(response.to_string().as_bytes());
        }
        
        // Write all responses at once with timeout
        match timeout(WRITE_TIMEOUT, writer.write_all(response_buffer)).await {
            Ok(Ok(_)) => {
                trace!("Sent {} responses in batch", pipeline.len());
                pipeline.clear();
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Write error: {}", e);
                Err(e.into())
            }
            Err(_) => {
                error!("Write timeout");
                Err(DiskDBError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Write timeout",
                )))
            }
        }
    }
    
    async fn process_pipeline_tls<W>(
        pipeline: &mut Vec<(String, Result<Request>)>,
        executor: &Arc<CommandExecutor>,
        response_buffer: &mut BytesMut,
        writer: &mut W,
    ) -> Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        response_buffer.clear();
        
        for (_, request_result) in pipeline.iter() {
            let response = match request_result {
                Ok(request) => {
                    match executor.execute(request.clone()).await {
                        Ok(resp) => resp,
                        Err(e) => Response::Error(e.to_string()),
                    }
                }
                Err(e) => Response::Error(e.to_string()),
            };
            
            response_buffer.put(response.to_string().as_bytes());
        }
        
        match timeout(WRITE_TIMEOUT, writer.write_all(response_buffer)).await {
            Ok(Ok(_)) => {
                trace!("Sent {} TLS responses", pipeline.len());
                pipeline.clear();
                Ok(())
            }
            Ok(Err(e)) => {
                error!("TLS write error: {}", e);
                Err(e.into())
            }
            Err(_) => {
                error!("TLS write timeout");
                Err(DiskDBError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Write timeout",
                )))
            }
        }
    }
}

/// Create an optimized TCP listener
pub async fn create_optimized_listener(addr: &str) -> Result<tokio::net::TcpListener> {
    let addr: SocketAddr = addr.parse()
        .map_err(|e| DiskDBError::Config(format!("Invalid address: {}", e)))?;
    
    // Create socket with custom options
    let socket = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP))?;
    
    // Set socket options before binding
    socket.set_reuse_address(true)?;
    
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    
    // Set socket buffer sizes
    let _ = socket.set_recv_buffer_size(256 * 1024);
    let _ = socket.set_send_buffer_size(256 * 1024);
    
    // Bind and listen
    socket.bind(&addr.into())?;
    socket.listen(1024)?;
    
    // Convert to tokio TcpListener
    socket.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(socket.into())?;
    
    Ok(listener)
}