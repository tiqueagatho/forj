use crate::error::Result;
use async_trait::async_trait;

/// Address used to connect any protocol driver.
#[derive(Debug, Clone)]
pub struct ProtocolAddr {
    pub host: String,
    pub port: u16,
    pub params: Vec<(String, String)>,
}

impl ProtocolAddr {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            params: vec![],
        }
    }
}

/// Health status returned by every protocol driver.
#[derive(Debug, Clone)]
pub struct Health {
    pub connected: bool,
    pub latency_ms: u64,
}

/// Erased protocol driver usable through the registry.
#[async_trait]
pub trait ErasedDriver: Send + Sync {
    fn name(&self) -> &'static str;
    async fn connect(&self, addr: ProtocolAddr) -> Result<Box<dyn ErasedConnection>>;
}

/// Erased connection returned by a driver.
#[async_trait]
pub trait ErasedConnection: Send + Sync {
    async fn read(&self, addr: u32, cnt: u16) -> Result<Vec<u8>>;
    async fn write(&self, addr: u32, data: &[u8]) -> Result<()>;
    async fn health(&self) -> Result<Health>;
    async fn disconnect(&self) -> Result<()>;
}

/// Typed protocol driver trait.
#[async_trait]
pub trait ProtocolDriver: Send + Sync + Sized + 'static {
    type Conn: ProtocolConnection;
    fn name() -> &'static str;
    async fn connect(addr: ProtocolAddr) -> Result<Self::Conn>;
}

/// Typed protocol connection.
#[async_trait]
pub trait ProtocolConnection: Send + Sync + Sized + 'static {
    async fn read(&self, addr: u32, cnt: u16) -> Result<Vec<u8>>;
    async fn write(&self, addr: u32, data: &[u8]) -> Result<()>;
    async fn health(&self) -> Result<Health>;
    async fn disconnect(self) -> Result<()>;
}
