use crate::error::Result;
use async_trait::async_trait;

/// A multi-dimensional tensor used for inference I/O.
#[derive(Debug, Clone)]
pub struct Tensor {
    pub data: Vec<u8>,
    pub shape: Vec<usize>,
    pub dtype: DType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DType {
    F32,
    F64,
    I32,
    I64,
    U8,
    U16,
}

impl Tensor {
    pub fn new(data: Vec<u8>, shape: Vec<usize>, dtype: DType) -> Self {
        Self { data, shape, dtype }
    }

    pub fn element_count(&self) -> usize {
        self.shape.iter().product()
    }
}

/// The result of a single inference pass.
#[derive(Debug, Clone)]
pub struct InferenceOutput {
    pub outputs: Vec<Tensor>,
    pub latency_us: u64,
}

/// Inference stage used inside a Pipeline.
pub struct InferenceStage {
    #[allow(dead_code)]
    pub(crate) inner: Box<dyn DynInferenceBackend>,
}

/// Erased dynamic backend for pipeline storage.
#[async_trait]
pub trait DynInferenceBackend: Send + Sync {
    fn name(&self) -> &'static str;
    async fn infer(&self, input: &[Tensor]) -> Result<InferenceOutput>;
}

/// Primary trait: any inference backend implements this.
#[async_trait]
pub trait InferenceBackend: Send + Sync + Sized + 'static {
    type Model;
    async fn load(path: &str) -> Result<Self::Model>;
    async fn infer(model: &Self::Model, input: &[Tensor]) -> Result<InferenceOutput>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tensor_new_and_count() {
        let t = Tensor::new(vec![0u8; 12], vec![1, 3, 2, 2], DType::F32);
        assert_eq!(t.element_count(), 12);
        assert_eq!(t.dtype, DType::F32);
    }

    #[test]
    fn tensor_debug_clone() {
        let t = Tensor::new(vec![1, 2, 3], vec![3], DType::U8);
        let t2 = t.clone();
        assert_eq!(t2.data, vec![1, 2, 3]);
    }

    #[test]
    fn tensor_various_dtypes() {
        for dtype in [
            DType::F32,
            DType::F64,
            DType::I32,
            DType::I64,
            DType::U8,
            DType::U16,
        ] {
            let t = Tensor::new(vec![], vec![], dtype);
            assert_eq!(t.dtype, dtype);
        }
    }

    #[test]
    fn inference_output_clone() {
        let out = InferenceOutput {
            outputs: vec![Tensor::new(vec![0u8; 4], vec![4], DType::F32)],
            latency_us: 1234,
        };
        let out2 = out.clone();
        assert_eq!(out2.latency_us, 1234);
        assert_eq!(out2.outputs.len(), 1);
    }

    #[test]
    fn tensor_element_count_zero() {
        let t = Tensor::new(vec![], vec![0], DType::F32);
        assert_eq!(t.element_count(), 0);
    }

    #[test]
    fn inference_stage_size() {
        // InferenceStage contains a Box<dyn> which is 2 words (ptr + vtable) on 64-bit
        assert_eq!(
            std::mem::size_of::<InferenceStage>(),
            std::mem::size_of::<usize>() * 2
        );
    }
}
