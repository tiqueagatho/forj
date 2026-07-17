use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    pub plc: Option<PlcConfig>,
    pub sensor: Option<SensorConfig>,
    pub inference: Option<InferenceConfig>,
    pub pipeline: Option<PipelineConfig>,
    pub security: Option<SecurityConfig>,
    pub monitoring: Option<MonitoringConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlcConfig {
    pub protocol: String,
    pub ip: String,
    pub port: Option<u16>,
    pub slave_id: Option<u8>,
    pub register_map: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub sensor_type: String,
    pub connection: String,
    pub sampling_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub backend: String,
    pub model: String,
    pub device: Option<String>,
    pub batch_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub workers: Option<usize>,
    pub buffer_size: Option<usize>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub tls: Option<crate::security::tls::TlsConfig>,
    pub auth_mode: Option<String>,
    pub audit_log: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub tracing: Option<bool>,
    pub otel_endpoint: Option<String>,
    pub metrics_port: Option<u16>,
}

pub fn load_config(path: &str) -> Result<RuntimeConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::Config(format!("cannot read {path}: {e}")))?;
    let cfg: RuntimeConfig =
        toml::from_str(&content).map_err(|e| Error::Config(format!("parse {path}: {e}")))?;
    Ok(cfg)
}
