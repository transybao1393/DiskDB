use crate::commands::CommandExecutor;
use crate::config::Config;
use crate::connection::Connection;
use crate::error::Result;
use crate::storage::Storage;
use crate::tls::create_tls_acceptor;
use log::{error, info};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::TlsAcceptor;

pub struct Server {
    config: Config,
    storage: Arc<dyn Storage>,
    tls_acceptor: Option<TlsAcceptor>,
}

impl Server {
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
        let listener = TcpListener::bind(&addr).await?;
        info!("Server listening on {}", addr);
        
        if self.config.use_tls {
            info!("TLS enabled");
        }

        let executor = Arc::new(CommandExecutor::new(self.storage.clone()));

        loop {
            let (stream, addr) = listener.accept().await?;
            let executor = executor.clone();
            let tls_acceptor = self.tls_acceptor.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(stream, addr.to_string(), executor, tls_acceptor).await {
                    error!("Error handling client {}: {}", addr, e);
                }
            });
        }
    }

    async fn handle_client(
        stream: TcpStream,
        addr: String,
        executor: Arc<CommandExecutor>,
        tls_acceptor: Option<TlsAcceptor>,
    ) -> Result<()> {
        let connection = if let Some(acceptor) = tls_acceptor {
            match acceptor.accept(stream).await {
                Ok(tls_stream) => Connection::Tls(tls_stream),
                Err(e) => {
                    error!("TLS handshake failed for {}: {}", addr, e);
                    return Err(e.into());
                }
            }
        } else {
            Connection::Plain(stream)
        };

        connection.handle(executor, addr).await
    }
}