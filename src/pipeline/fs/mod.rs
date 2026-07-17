use crate::error::Result;
use crate::pipeline::Source;
use async_trait::async_trait;

pub struct FusionSensor {
    sources: Vec<Box<dyn Source>>,
}

impl FusionSensor {
    pub fn new(sources: Vec<Box<dyn Source>>) -> Self {
        Self { sources }
    }
}

#[async_trait]
impl Source for FusionSensor {
    fn name(&self) -> &'static str {
        "fusion"
    }
    async fn poll(&self) -> Result<Vec<u8>> {
        let mut combined = Vec::new();
        for src in &self.sources {
            let data = src.poll().await?;
            combined.extend(data);
        }
        Ok(combined)
    }
}
