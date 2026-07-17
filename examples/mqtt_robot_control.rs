use async_trait::async_trait;
use forj::pipeline::traits::*;
use forj::prelude::*;

struct MqttSensor;

#[async_trait]
impl Source for MqttSensor {
    fn name(&self) -> &'static str {
        "mqtt-sensor"
    }
    async fn poll(&self) -> Result<Vec<u8>> {
        Ok(vec![42u8])
    }
}

struct MqttRobot;

#[async_trait]
impl Sink for MqttRobot {
    fn name(&self) -> &'static str {
        "mqtt-robot"
    }
    async fn send(&self, data: Vec<u8>) -> Result<()> {
        println!("[MQTT] robot cmd = {data:?}");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let pipeline = std::sync::Arc::new(
        Pipeline::builder()
            .source_raw(MqttSensor)
            .sink_raw(MqttRobot)
            .build()?,
    );
    let p = pipeline.clone();
    let _h = tokio::spawn(async move { p.run().await });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    Ok(())
}
