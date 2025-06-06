use crate::commands::CommandExecutor;
use crate::error::{Result, DiskDBError};
use crate::network::buffer_pool::GLOBAL_BUFFER_POOL;
use crate::protocol::{Request, Response};
use bytes::BytesMut;
use log::{error, info, trace};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_uring::net::{TcpListener, TcpStream};
use tokio_uring::buf::BoundedBuf;

const RING_SIZE: u32 = 256;
const MAX_CONNECTIONS: usize = 10000;
const BUFFER_SIZE: usize = 4096;

/// io_uring-based server for maximum performance on Linux
pub struct IoUringServer {
    addr: SocketAddr,
    executor: Arc<CommandExecutor>,
}

#[derive(Debug)]
struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    read_buf: Vec<u8>,
    write_buf: BytesMut,
    pending_requests: Vec<String>,
}

impl IoUringServer {
    pub fn new(addr: &str, executor: Arc<CommandExecutor>) -> Result<Self> {
        let addr = addr.parse()
            .map_err(|e| DiskDBError::Config(format!("Invalid address: {}", e)))?;
        
        Ok(Self {
            addr,
            executor,
        })
    }
    
    /// Start the io_uring-based server
    pub async fn start(self) -> Result<()> {
        // Initialize tokio_uring runtime
        tokio_uring::start(async move {
            if let Err(e) = self.run_server().await {
                error!("io_uring server error: {}", e);
            }
        });
        
        Ok(())
    }
    
    async fn run_server(self) -> Result<()> {
        let listener = TcpListener::bind(self.addr)?;
        info!("io_uring server listening on {}", self.addr);
        
        let mut connections: HashMap<u64, Connection> = HashMap::new();
        let mut next_id = 0u64;
        
        loop {
            // Accept new connections
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let id = next_id;
                    next_id += 1;
                    
                    info!("New io_uring connection from: {} (id: {})", addr, id);
                    
                    // Set TCP_NODELAY
                    let _ = stream.set_nodelay(true);
                    
                    let conn = Connection {
                        stream,
                        addr,
                        read_buf: vec![0u8; BUFFER_SIZE],
                        write_buf: BytesMut::with_capacity(BUFFER_SIZE),
                        pending_requests: Vec::new(),
                    };
                    
                    connections.insert(id, conn);
                    
                    // Start handling the connection
                    tokio_uring::spawn(Self::handle_connection(
                        id,
                        connections.remove(&id).unwrap(),
                        self.executor.clone(),
                    ));
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                    continue;
                }
            }
            
            // Limit connections
            if connections.len() >= MAX_CONNECTIONS {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }
    }
    
    async fn handle_connection(
        id: u64,
        mut conn: Connection,
        executor: Arc<CommandExecutor>,
    ) {
        trace!("Starting io_uring handler for connection {}", id);
        
        loop {
            // Read data using io_uring
            let (res, buf) = conn.stream.read(conn.read_buf).await;
            conn.read_buf = buf;
            
            match res {
                Ok(0) => {
                    info!("Connection {} closed", id);
                    break;
                }
                Ok(n) => {
                    trace!("Read {} bytes from connection {}", n, id);
                    
                    // Process the data
                    let data = &conn.read_buf[..n];
                    if let Ok(str_data) = std::str::from_utf8(data) {
                        // Parse lines
                        for line in str_data.lines() {
                            if line.is_empty() {
                                continue;
                            }
                            
                            conn.pending_requests.push(line.to_string());
                        }
                        
                        // Process requests if we have any complete ones
                        if !conn.pending_requests.is_empty() {
                            Self::process_requests(
                                &mut conn,
                                &executor,
                            ).await;
                        }
                    }
                }
                Err(e) => {
                    error!("Read error on connection {}: {}", id, e);
                    break;
                }
            }
        }
        
        trace!("io_uring handler for connection {} finished", id);
    }
    
    async fn process_requests(
        conn: &mut Connection,
        executor: &Arc<CommandExecutor>,
    ) {
        conn.write_buf.clear();
        
        // Process all pending requests
        for request_str in &conn.pending_requests {
            let response = match Request::parse(request_str) {
                Ok(request) => {
                    match executor.execute(request).await {
                        Ok(resp) => resp,
                        Err(e) => Response::Error(e.to_string()),
                    }
                }
                Err(e) => Response::Error(e.to_string()),
            };
            
            // Append response to write buffer
            conn.write_buf.extend_from_slice(response.to_string().as_bytes());
        }
        
        // Clear processed requests
        conn.pending_requests.clear();
        
        // Write response using io_uring
        let write_data = conn.write_buf.split().freeze();
        let (res, _) = conn.stream.write_all(write_data).await;
        
        if let Err(e) = res {
            error!("Write error: {}", e);
        }
    }
}

/// Zero-copy buffer for io_uring operations
#[derive(Clone)]
struct IoUringBuffer {
    data: Vec<u8>,
}

impl BoundedBuf for IoUringBuffer {
    type Buf = Vec<u8>;
    type Bounds = (usize, usize);
    
    fn slice(self, bounds: Self::Bounds) -> Self::Buf {
        let mut data = self.data;
        data.truncate(bounds.1);
        data.drain(..bounds.0);
        data
    }
    
    fn slice_full(self) -> Self::Buf {
        self.data
    }
    
    fn into_inner(self) -> Self::Buf {
        self.data
    }
    
    fn bounds(&self) -> Self::Bounds {
        (0, self.data.len())
    }
}

/// Create an io_uring optimized server
pub async fn create_io_uring_server(
    addr: &str,
    executor: Arc<CommandExecutor>,
) -> Result<()> {
    let server = IoUringServer::new(addr, executor)?;
    server.start().await
}