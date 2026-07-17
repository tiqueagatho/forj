use crate::error::{Error, Result};
use crate::inference::traits::DynInferenceBackend;
use crate::inference::traits::*;
use async_trait::async_trait;

pub struct OnnxBackend;

pub struct OnnxModel {
    #[cfg(feature = "onnx")]
    inner: ort::session::Session,
}

#[async_trait]
impl InferenceBackend for OnnxBackend {
    type Model = OnnxModel;

    async fn load(path: &str) -> Result<Self::Model> {
        #[cfg(feature = "onnx")]
        {
            let session = ort::session::Session::builder()
                .map_err(|e| Error::ModelLoad(format!("onnx builder: {e}")))?
                .commit_from_file(path)
                .map_err(|e| Error::ModelLoad(format!("onnx file: {e}")))?;
            return Ok(OnnxModel { inner: session });
        }
        #[cfg(not(feature = "onnx"))]
        Err(Error::BackendUnavailable("onnx".into()))
    }

    async fn infer(_model: &Self::Model, _input: &[Tensor]) -> Result<InferenceOutput> {
        #[cfg(feature = "onnx")]
        {
            let start = std::time::Instant::now();
            // ort rc.12: model.inner.run(inputs)
            let _elapsed = start.elapsed().as_micros() as u64;
            Err(Error::Inference("onnx: actual run requires ort::value::Value — use ort crate directly for real inference".into()))
        }
        #[cfg(not(feature = "onnx"))]
        Err(Error::BackendUnavailable("onnx".into()))
    }
}

pub fn dyn_model(model: OnnxModel) -> Box<dyn DynInferenceBackend> {
    Box::new(OnnxDyn {
        #[cfg(feature = "onnx")]
        inner: Some(model.inner),
    })
}

pub(crate) struct OnnxDyn {
    #[cfg(feature = "onnx")]
    #[allow(dead_code)]
    inner: Option<ort::session::Session>,
}

#[async_trait]
impl DynInferenceBackend for OnnxDyn {
    fn name(&self) -> &'static str {
        "onnx"
    }

    async fn infer(&self, _input: &[Tensor]) -> Result<InferenceOutput> {
        #[cfg(feature = "onnx")]
        {
            Err(Error::Inference(
                "onnx dyn: run requires ort value types".into(),
            ))
        }
        #[cfg(not(feature = "onnx"))]
        Err(Error::BackendUnavailable("onnx".into()))
    }
}
