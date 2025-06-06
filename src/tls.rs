use crate::error::Result;
use native_tls::{Identity, TlsAcceptor};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn create_tls_acceptor(cert_path: &Path, key_path: &Path) -> Result<TlsAcceptor> {
    let mut cert_file = File::open(cert_path)?;
    let mut cert_contents = Vec::new();
    cert_file.read_to_end(&mut cert_contents)?;

    let mut key_file = File::open(key_path)?;
    let mut key_contents = Vec::new();
    key_file.read_to_end(&mut key_contents)?;

    let identity = Identity::from_pkcs12(&cert_contents, "")
        .or_else(|_| {
            let cert_pem = String::from_utf8_lossy(&cert_contents);
            let key_pem = String::from_utf8_lossy(&key_contents);
            Identity::from_pkcs8(cert_pem.as_bytes(), key_pem.as_bytes())
        })?;

    let acceptor = TlsAcceptor::new(identity)?;
    Ok(acceptor)
}