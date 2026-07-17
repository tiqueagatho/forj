use async_trait::async_trait;
use forj::pipeline::traits::*;
use forj::prelude::*;

struct OpcuaSensor;

#[async_trait]
impl Source for OpcuaSensor {
    fn name(&self) -> &'static str {
        "opcua-temp"
    }
    async fn poll(&self) -> Result<Vec<u8>> {
        let temp: f32 = 85.0;
        Ok(temp.to_le_bytes().to_vec())
    }
}

struct OpcuaAlarm;

#[async_trait]
impl Sink for OpcuaAlarm {
    fn name(&self) -> &'static str {
        "opcua-alarm"
    }
    async fn send(&self, data: Vec<u8>) -> Result<()> {
        let alarm = data.first().copied().unwrap_or(0);
        println!("[OPC-UA] alarm = {alarm}");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let pipeline = std::sync::Arc::new(
        Pipeline::builder()
            .source_raw(OpcuaSensor)
            .postprocess(forj::inference::postprocessor::Threshold::new(80.0, 150.0))
            .sink_raw(OpcuaAlarm)
            .build()?,
    );
    let p = pipeline.clone();
    let _h = tokio::spawn(async move { p.run().await });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    Ok(())
}
