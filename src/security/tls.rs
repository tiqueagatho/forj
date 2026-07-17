use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub ca_path: Option<String>,
    pub server_name: Option<String>,
    pub enable_mtls: bool,
}

pub fn validate_tls(cfg: &TlsConfig) -> Result<()> {
    if let Some(cert) = &cfg.cert_path {
        if !std::path::Path::new(cert).exists() {
            return Err(Error::Config(format!("TLS cert not found: {cert}")));
        }
    }
    if let Some(key) = &cfg.key_path {
        if !std::path::Path::new(key).exists() {
            return Err(Error::Config(format!("TLS key not found: {key}")));
        }
    }
    Ok(())
}
