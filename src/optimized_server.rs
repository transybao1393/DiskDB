use crate::commands::CommandExecutor;
use crate::config::Config;
use crate::error::Result;
use crate::network::{
    buffer_pool::GLOBAL_BUFFER_POOL,
    optimized_connection::{create_optimized_listener, OptimizedConnection},
};
use crate::storage::Storage;
use crate::tls::create_tls_acceptor;
use log::{error, info};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_native_tls::TlsAcceptor;

/// Optimized server with network I/O improvements
pub struct OptimizedServer {
    config: Config,
    storage: Arc<dyn Storage>,
    tls_acceptor: Option<TlsAcceptor>,
}

impl OptimizedServer {
    pub fn new(config: Config, storage: Arc<dyn Storage>) -> Result<Self> {
        let tls_acceptor = if config.use_tls {
            let cert_path = config.cert_path.as_ref()
                .ok_or_else(|| crate::error::DiskDBError::Protocol("TLS enabled but cert_path not provided".to_string()))?;
            let key_path = config.key_path.as_ref()
                .ok_or_else(|| crate::error::DiskDBError::Protocol("TLS enabled but key_path not provided".to_string()))?;
            
            Some(TlsAcceptor::from(create_tls_acceptor(cert_path, key_path)?))
        } else {
            None
        };

        Ok(Self {
            config,
            storage,
            tls_acceptor,
        })
    }

    pub async fn start(&self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.config.server_port);
        
        // Use io_uring on Linux if available
        #[cfg(all(target_os = "linux", feature = "io_uring"))]
        {
            if !self.config.use_tls {
                info!("Starting io_uring optimized server on {}", addr);
                let executor = Arc::new(CommandExecutor::new(self.storage.clone()));
                return crate::network::io_uring_server::create_io_uring_server(&addr, executor).await;
            }
        }
        
        // Use optimized TCP listener
        let listener = create_optimized_listener(&addr).await?;
        info!("Optimized server listening on {}", addr);
        
        if self.config.use_tls {
            info!("TLS enabled");
        }
        
        // Pre-allocate buffers
        info!("Pre-allocating network buffers...");
        GLOBAL_BUFFER_POOL.preallocate(200, 100, 20);

        let executor = Arc::new(CommandExecutor::new(self.storage.clone()));
        let buffer_pool = GLOBAL_BUFFER_POOL.clone();

        loop {
            let (stream, addr) = listener.accept().await?;
            let executor = executor.clone();
            let tls_acceptor = self.tls_acceptor.clone();
            let buffer_pool = buffer_pool.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(
                    stream, 
                    addr, 
                    executor, 
                    tls_acceptor,
                    buffer_pool,
                ).await {
                    error!("Error handling client {}: {}", addr, e);
                }
            });
        }
    }

    async fn handle_client(
        stream: TcpStream,
        addr: std::net::SocketAddr,
        executor: Arc<CommandExecutor>,
        tls_acceptor: Option<TlsAcceptor>,
        buffer_pool: Arc<crate::network::buffer_pool::BufferPool>,
    ) -> Result<()> {
        // Create optimized connection
        let mut connection = OptimizedConnection::accept(stream, addr).await?;
        
        // Handle TLS if enabled
        if let Some(acceptor) = tls_acceptor {
            match connection {
                OptimizedConnection::Plain(stream) => {
                    match acceptor.accept(stream).await {
                        Ok(tls_stream) => {
                            connection = OptimizedConnection::Tls(tls_stream);
                        }
                        Err(e) => {
                            error!("TLS handshake failed for {}: {}", addr, e);
                            return Err(e.into());
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        // Handle with optimizations
        connection.handle(executor, addr.to_string(), Some(buffer_pool)).await
    }
    
    /// Get server statistics
    pub fn stats(&self) -> ServerStats {
        let buffer_stats = GLOBAL_BUFFER_POOL.stats();
        
        ServerStats {
            buffer_pool_stats: buffer_stats,
            optimizations_enabled: OptimizationsEnabled {
                c_parser: cfg!(feature = "c_parser"),
                memory_pool: cfg!(feature = "memory_pool"),
                io_uring: cfg!(all(target_os = "linux", feature = "io_uring")),
                vectored_io: true,
                tcp_nodelay: true,
                buffer_pooling: true,
            },
        }
    }
}

#[derive(Debug)]
pub struct ServerStats {
    pub buffer_pool_stats: crate::network::buffer_pool::BufferPoolStats,
    pub optimizations_enabled: OptimizationsEnabled,
}

#[derive(Debug)]
pub struct OptimizationsEnabled {
    pub c_parser: bool,
    pub memory_pool: bool,
    pub io_uring: bool,
    pub vectored_io: bool,
    pub tcp_nodelay: bool,
    pub buffer_pooling: bool,
}