pub mod traits;

use crate::error::Result;
use async_trait::async_trait;

/// Core trait: every industrial device implements this.
#[async_trait]
pub trait IndustrialDevice: Send + Sync + 'static {
    type Reading: Send + 'static;
    type Command: Send + 'static;

    async fn read(&self) -> Result<Self::Reading>;
    async fn write(&self, cmd: Self::Command) -> Result<()>;
    async fn health(&self) -> Result<DeviceHealth>;
}

/// Erased version for pipeline storage.
#[async_trait]
pub trait DynIndustrialDevice: Send + Sync {
    async fn dyn_read(&self) -> Result<Vec<u8>>;
    async fn dyn_write(&self, cmd: Vec<u8>) -> Result<()>;
    async fn dyn_health(&self) -> Result<DeviceHealth>;
}

#[derive(Debug, Clone)]
pub struct DeviceHealth {
    pub connected: bool,
    pub uptime_secs: u64,
    pub last_error: Option<String>,
}

/// Blanket impl: any IndustrialDevice with Vec<u8> I/O becomes DynIndustrialDevice.
#[async_trait]
impl<T> DynIndustrialDevice for T
where
    T: IndustrialDevice<Reading = Vec<u8>, Command = Vec<u8>> + Sync,
{
    async fn dyn_read(&self) -> Result<Vec<u8>> {
        self.read().await
    }
    async fn dyn_write(&self, cmd: Vec<u8>) -> Result<()> {
        self.write(cmd).await
    }
    async fn dyn_health(&self) -> Result<DeviceHealth> {
        self.health().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDevice;

    #[async_trait]
    impl IndustrialDevice for TestDevice {
        type Reading = Vec<u8>;
        type Command = Vec<u8>;

        async fn read(&self) -> Result<Vec<u8>> {
            Ok(vec![0x01, 0x02])
        }
        async fn write(&self, cmd: Vec<u8>) -> Result<()> {
            assert!(!cmd.is_empty());
            Ok(())
        }
        async fn health(&self) -> Result<DeviceHealth> {
            Ok(DeviceHealth {
                connected: true,
                uptime_secs: 42,
                last_error: None,
            })
        }
    }

    #[tokio::test]
    async fn test_device_read() {
        let d = TestDevice;
        let reading = d.read().await.unwrap();
        assert_eq!(reading, vec![0x01, 0x02]);
    }

    #[tokio::test]
    async fn test_device_write() {
        let d = TestDevice;
        d.write(vec![0xFF]).await.unwrap();
    }

    #[tokio::test]
    async fn test_device_health() {
        let d = TestDevice;
        let h = d.health().await.unwrap();
        assert!(h.connected);
        assert_eq!(h.uptime_secs, 42);
        assert!(h.last_error.is_none());
    }

    #[tokio::test]
    async fn test_dyn_device() {
        let d: Box<dyn DynIndustrialDevice> = Box::new(TestDevice);
        let reading = d.dyn_read().await.unwrap();
        assert_eq!(reading, vec![0x01, 0x02]);
        d.dyn_write(vec![0xAB]).await.unwrap();
        let h = d.dyn_health().await.unwrap();
        assert!(h.connected);
    }

    #[test]
    fn device_health_debug_clone() {
        let h = DeviceHealth {
            connected: false,
            uptime_secs: 0,
            last_error: Some("fail".into()),
        };
        let h2 = h.clone();
        assert!(!h2.connected);
    }
}
