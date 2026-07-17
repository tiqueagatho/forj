use crate::error::{Error, Result};
use crate::inference::traits::*;
use async_trait::async_trait;

pub struct CandleBackend;

pub struct CandleModel {
    #[cfg(feature = "candle")]
    _inner: candle_core::Device,
}

#[async_trait]
impl InferenceBackend for CandleBackend {
    type Model = CandleModel;

    async fn load(path: &str) -> Result<Self::Model> {
        #[cfg(feature = "candle")]
        {
            let device = candle_core::Device::Cpu;
            Ok(CandleModel { _inner: device })
        }
        #[cfg(not(feature = "candle"))]
        Err(Error::BackendUnavailable("candle".into()))
    }

    async fn infer(model: &Self::Model, _input: &[Tensor]) -> Result<InferenceOutput> {
        let start = std::time::Instant::now();
        let _ = model;
        // Placeholder — Candle model load/infer requires safetensors + architecture
        Err(Error::Inference(
            "candle: model graph not yet loaded — use candle-core directly for custom arch".into(),
        ))
    }
}
