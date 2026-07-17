use crate::error::{Error, Result};
use crate::inference::traits::{DType, InferenceBackend, InferenceOutput, Tensor};
use async_trait::async_trait;
use std::sync::Arc;
use tract_onnx::prelude::*;

pub struct TractBackend;

type TractRunnable = Arc<RunnableModel<TypedFact, Box<dyn TypedOp>>>;

pub struct TractModel {
    _name: String,
    #[cfg(feature = "tract")]
    plan: TractRunnable,
}

fn tensor_to_tract(t: &Tensor) -> TractResult<TValue> {
    use tract_ndarray::Array;
    let shape: Vec<usize> = t.shape.clone();
    match t.dtype {
        DType::F32 => {
            let data: Vec<f32> = t
                .data
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            let array = Array::from_shape_vec(shape, data)
                .map_err(|e| anyhow::anyhow!("tract shape: {e}"))?;
            Ok(array.into_tvalue())
        }
        DType::U8 => {
            let array = Array::from_shape_vec(shape, t.data.clone())
                .map_err(|e| anyhow::anyhow!("tract shape: {e}"))?;
            Ok(array.into_tvalue())
        }
        DType::I32 => {
            let data: Vec<i32> = t
                .data
                .chunks_exact(4)
                .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            let array = Array::from_shape_vec(shape, data)
                .map_err(|e| anyhow::anyhow!("tract shape: {e}"))?;
            Ok(array.into_tvalue())
        }
        DType::I64 => {
            let data: Vec<i64> = t
                .data
                .chunks_exact(8)
                .map(|c| i64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]))
                .collect();
            let array = Array::from_shape_vec(shape, data)
                .map_err(|e| anyhow::anyhow!("tract shape: {e}"))?;
            Ok(array.into_tvalue())
        }
        _ => Err(anyhow::anyhow!("unsupported dtype for tract conversion")),
    }
}

fn tensor_from_tract(t: TValue) -> Result<Tensor> {
    let shape: Vec<usize> = t.shape().to_vec();
    let tensor = t.into_tensor();
    let data: Vec<u8> = tensor.as_bytes().to_vec();
    let dt = match tensor.datum_type() {
        DatumType::F32 => DType::F32,
        DatumType::U8 => DType::U8,
        DatumType::I32 => DType::I32,
        DatumType::I64 => DType::I64,
        _ => return Err(Error::Inference("unsupported tract output dtype".into())),
    };
    Ok(Tensor::new(data, shape, dt))
}

#[async_trait]
impl InferenceBackend for TractBackend {
    type Model = TractModel;

    async fn load(path: &str) -> Result<Self::Model> {
        #[cfg(feature = "tract")]
        {
            let model = tract_onnx::onnx()
                .model_for_path(path)
                .map_err(|e| Error::ModelLoad(format!("tract: {e}")))?
                .into_optimized()
                .map_err(|e| Error::ModelLoad(format!("tract optimize: {e}")))?
                .into_runnable()
                .map_err(|e| Error::ModelLoad(format!("tract runnable: {e}")))?;
            Ok(TractModel {
                _name: path.into(),
                plan: model,
            })
        }
        #[cfg(not(feature = "tract"))]
        Err(Error::BackendUnavailable("tract".into()))
    }

    async fn infer(model: &Self::Model, input: &[Tensor]) -> Result<InferenceOutput> {
        #[cfg(feature = "tract")]
        {
            let start = std::time::Instant::now();
            let mut tract_inputs: TVec<TValue> = TVec::new();
            for t in input {
                tract_inputs.push(
                    tensor_to_tract(t)
                        .map_err(|e| Error::Inference(format!("tract input: {e}")))?,
                );
            }
            let outputs = model
                .plan
                .run(tract_inputs)
                .map_err(|e| Error::Inference(format!("tract run: {e}")))?;
            let elapsed = start.elapsed().as_micros() as u64;
            let mut result = Vec::with_capacity(outputs.len());
            for t in outputs {
                result.push(tensor_from_tract(t)?);
            }
            Ok(InferenceOutput {
                outputs: result,
                latency_us: elapsed,
            })
        }
        #[cfg(not(feature = "tract"))]
        Err(Error::BackendUnavailable("tract".into()))
    }
}
