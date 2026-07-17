//! Modbus Vision: camera → AI → PLC. Simulated.

use async_trait::async_trait;
use forj::pipeline::traits::*;
use forj::prelude::*;
use std::sync::Arc;

struct SimCamera {
    data: Arc<Vec<u8>>,
}

#[async_trait]
impl Source for SimCamera {
    fn name(&self) -> &'static str {
        "sim-camera"
    }
    async fn poll(&self) -> Result<Vec<u8>> {
        Ok(self.data.to_vec())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _img: Vec<u8> = (0..128 * 128 * 3).map(|_| 128u8).collect();

    let pipeline = std::sync::Arc::new(
        Pipeline::builder()
            .source_raw(SimCamera {
                data: Arc::new(vec![0u8; 1024]),
            })
            .preprocess(forj::inference::preprocessor::Normalize::new(0.0, 255.0))
            .sink_raw(SimPlc)
            .backpressure(BackpressureConfig::bounded(64))
            .build()?,
    );

    println!("Starting Modbus Vision pipeline...");
    let p = pipeline.clone();
    let _ = tokio::spawn(async move { p.run().await });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    Ok(())
}

struct SimPlc;

#[async_trait]
impl Sink for SimPlc {
    fn name(&self) -> &'static str {
        "sim-plc"
    }
    async fn send(&self, data: Vec<u8>) -> Result<()> {
        let d = data.first().copied().unwrap_or(0);
        println!("[PLC] register = {d}");
        Ok(())
    }
}
