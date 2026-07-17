//! forj — Fusion Ops Runtime for Joint Industrial Intelligence.
//!
//! Safe, concurrent, modular AI inference for Industry 4.0.
// Safety: the `realtime` module contains an unsafe block for Linux sched_setscheduler.
// All other modules must remain safe. We enforce this at code-review time.
#![deny(unsafe_code)]

extern crate async_trait;

pub mod config;
pub mod device;
pub mod error;
pub mod inference;
pub mod monitoring;
pub mod pipeline;
pub mod protocol;
pub mod realtime;
pub mod security;

pub mod prelude {
    pub use crate::device::traits::*;
    pub use crate::device::IndustrialDevice;
    pub use crate::error::{Error, Result};
    #[cfg(feature = "onnx")]
    pub use crate::inference::backends::onnx::OnnxBackend;
    pub use crate::inference::traits::*;
    pub use crate::pipeline::traits::*;
    #[cfg(feature = "modbus-tcp")]
    pub use crate::protocol::modbus::tcp::ModbusTcpClient;
    pub use crate::protocol::traits::ProtocolDriver;
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_links() {
        assert!(option_env!("CARGO_PKG_VERSION").is_some());
    }
}
