use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_port: u16,
    pub database_path: PathBuf,
    pub use_tls: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub max_connections: usize,
    pub thread_pool_size: usize,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(port) = std::env::var("DISKDB_PORT") {
            if let Ok(p) = port.parse() {
                config.server_port = p;
            }
        }
        
        if let Ok(path) = std::env::var("DISKDB_PATH") {
            config.database_path = PathBuf::from(path);
        }
        
        if let Ok(tls) = std::env::var("DISKDB_USE_TLS") {
            config.use_tls = tls.to_lowercase() == "true" || tls == "1";
        }
        
        if let Ok(cert) = std::env::var("DISKDB_CERT_PATH") {
            config.cert_path = Some(PathBuf::from(cert));
        }
        
        if let Ok(key) = std::env::var("DISKDB_KEY_PATH") {
            config.key_path = Some(PathBuf::from(key));
        }
        
        if let Ok(max_conn) = std::env::var("DISKDB_MAX_CONNECTIONS") {
            if let Ok(m) = max_conn.parse() {
                config.max_connections = m;
            }
        }
        
        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_port: 6380,
            database_path: PathBuf::from("diskdb"),
            use_tls: false,
            cert_path: None,
            key_path: None,
            max_connections: 1000,
            thread_pool_size: num_cpus::get(),
        }
    }
}