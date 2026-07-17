use crate::error::{protocol_connect, Error, Result};
use crate::protocol::traits::{Health, ProtocolAddr, ProtocolConnection, ProtocolDriver};
use async_trait::async_trait;
use std::net::SocketAddr;

pub struct ModbusTcpClient;

fn to_socket_addr(addr: &ProtocolAddr) -> SocketAddr {
    format!("{}:{}", addr.host, addr.port)
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 502)))
}

#[async_trait]
impl ProtocolDriver for ModbusTcpClient {
    type Conn = ModbusTcpConnection;
    fn name() -> &'static str {
        "modbus-tcp"
    }

    async fn connect(addr: ProtocolAddr) -> Result<Self::Conn> {
        let socket = to_socket_addr(&addr);
        #[cfg(feature = "modbus-tcp")]
        {
            tokio_modbus::prelude::tcp::connect_slave(socket, tokio_modbus::prelude::Slave(1))
                .await
                .map_err(|e| protocol_connect(format!("{socket}: {e}")))?;
        }
        Ok(ModbusTcpConnection { socket })
    }
}

pub struct ModbusTcpConnection {
    socket: SocketAddr,
}

#[async_trait]
impl ProtocolConnection for ModbusTcpConnection {
    async fn read(&self, addr: u32, cnt: u16) -> Result<Vec<u8>> {
        #[cfg(feature = "modbus-tcp")]
        {
            let mut ctx = tokio_modbus::prelude::tcp::connect_slave(
                self.socket,
                tokio_modbus::prelude::Slave(1),
            )
            .await
            .map_err(|e| protocol_connect(format!("{}: {e}", self.socket)))?;
            let regs =
                tokio_modbus::prelude::Reader::read_holding_registers(&mut ctx, addr as u16, cnt)
                    .await
                    .map_err(|e| -> crate::error::Error {
                        Error::ProtocolRead {
                            address: format!("{}:{}", self.socket, addr),
                            reason: e.to_string(),
                        }
                    })?
                    .unwrap_or_default();
            let mut out = Vec::with_capacity(regs.len() * 2);
            for &r in &regs {
                out.extend_from_slice(&r.to_be_bytes());
            }
            return Ok(out);
        }
        #[cfg(not(feature = "modbus-tcp"))]
        Err(Error::UnsupportedProtocol("modbus-tcp".into()))
    }

    async fn write(&self, addr: u32, data: &[u8]) -> Result<()> {
        #[cfg(feature = "modbus-tcp")]
        {
            let mut ctx = tokio_modbus::prelude::tcp::connect_slave(
                self.socket,
                tokio_modbus::prelude::Slave(1),
            )
            .await
            .map_err(|e| protocol_connect(format!("{}: {e}", self.socket)))?;
            let regs: Vec<u16> = data
                .chunks(2)
                .map(|c| {
                    if c.len() == 2 {
                        u16::from_be_bytes([c[0], c[1]])
                    } else {
                        u16::from_be_bytes([c[0], 0])
                    }
                })
                .collect();
            let _ = tokio_modbus::prelude::Writer::write_multiple_registers(
                &mut ctx,
                addr as u16,
                &regs,
            )
            .await
            .map_err(|e| Error::ProtocolWrite {
                address: self.socket.to_string(),
                reason: e.to_string(),
            })?;
            return Ok(());
        }
        #[cfg(not(feature = "modbus-tcp"))]
        Err(Error::UnsupportedProtocol("modbus-tcp".into()))
    }

    async fn health(&self) -> Result<Health> {
        #[cfg(feature = "modbus-tcp")]
        {
            let start = std::time::Instant::now();
            let mut ctx = tokio_modbus::prelude::tcp::connect_slave(
                self.socket,
                tokio_modbus::prelude::Slave(1),
            )
            .await
            .map_err(|e| Error::ProtocolConnect(e.to_string()))?;
            let _ = tokio_modbus::prelude::Reader::read_holding_registers(&mut ctx, 0, 1)
                .await
                .map_err(|_| Error::Other("health check failed".into()))?;
            return Ok(Health {
                connected: true,
                latency_ms: start.elapsed().as_millis() as u64,
            });
        }
        #[cfg(not(feature = "modbus-tcp"))]
        Err(Error::UnsupportedProtocol("modbus-tcp".into()))
    }

    async fn disconnect(self) -> Result<()> {
        Ok(())
    }
}
