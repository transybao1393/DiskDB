use crate::commands::CommandExecutor;
use crate::error::Result;
use crate::protocol::{Request, Response};
use log::{error, info};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;

pub enum Connection {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl Connection {
    pub async fn handle(self, executor: Arc<CommandExecutor>, addr: String) -> Result<()> {
        info!("New connection from: {}", addr);
        
        match self {
            Connection::Plain(stream) => {
                let (reader, mut writer) = stream.into_split();
                let mut reader = BufReader::new(reader);
                let mut line = String::new();
                
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => break, // Connection closed
                        Ok(_) => {
                            if line.trim().is_empty() {
                                continue;
                            }

                            let response = match Request::parse(&line) {
                                Ok(request) => {
                                    match executor.execute(request).await {
                                        Ok(resp) => resp,
                                        Err(e) => Response::Error(e.to_string()),
                                    }
                                }
                                Err(e) => Response::Error(e.to_string()),
                            };

                            if let Err(e) = writer.write_all(response.to_string().as_bytes()).await {
                                error!("Failed to write response: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from stream: {}", e);
                            break;
                        }
                    }
                }
            }
            Connection::Tls(stream) => {
                let (reader, mut writer) = tokio::io::split(stream);
                let mut reader = BufReader::new(reader);
                let mut line = String::new();
                
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => break, // Connection closed
                        Ok(_) => {
                            if line.trim().is_empty() {
                                continue;
                            }

                            let response = match Request::parse(&line) {
                                Ok(request) => {
                                    match executor.execute(request).await {
                                        Ok(resp) => resp,
                                        Err(e) => Response::Error(e.to_string()),
                                    }
                                }
                                Err(e) => Response::Error(e.to_string()),
                            };

                            if let Err(e) = writer.write_all(response.to_string().as_bytes()).await {
                                error!("Failed to write response: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from stream: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        info!("Connection closed: {}", addr);
        Ok(())
    }
}