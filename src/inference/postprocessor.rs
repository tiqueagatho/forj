use crate::error::{Error, Result};
use crate::inference::traits::{DType, Tensor};
use async_trait::async_trait;

/// Postprocessor transforms inference tensors into decisions / commands.
#[async_trait]
pub trait Postprocessor: Send + Sync {
    fn name(&self) -> &'static str;
    async fn process(&self, output: &[Tensor]) -> Result<Vec<u8>>;
}

pub struct Classify {
    pub threshold: f32,
}

impl Classify {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

#[async_trait]
impl Postprocessor for Classify {
    fn name(&self) -> &'static str {
        "classify"
    }

    async fn process(&self, output: &[Tensor]) -> Result<Vec<u8>> {
        if output.is_empty() {
            return Err(Error::Postprocess("no output tensors".into()));
        }
        let t = &output[0];
        match t.dtype {
            DType::F32 => {
                let floats: Vec<f32> = t
                    .data
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                let (class_idx, prob) = floats
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(i, p)| (i as u8, *p))
                    .unwrap_or((0, 0.0));
                let decision = if prob >= self.threshold { class_idx } else { 0 };
                Ok(vec![decision, (prob * 100.0) as u8])
            }
            _ => Err(Error::Postprocess("unsupported dtype".into())),
        }
    }
}

pub struct Threshold {
    pub min: f32,
    pub max: f32,
}

impl Threshold {
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }
}

#[async_trait]
impl Postprocessor for Threshold {
    fn name(&self) -> &'static str {
        "threshold"
    }

    async fn process(&self, output: &[Tensor]) -> Result<Vec<u8>> {
        if output.is_empty() {
            return Err(Error::Postprocess("no output".into()));
        }
        let t = &output[0];
        match t.dtype {
            DType::F32 => {
                let vals: Vec<f32> = t
                    .data
                    .chunks_exact(4)
                    .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                let triggered = vals.iter().any(|v| *v >= self.min && *v <= self.max);
                Ok(vec![if triggered { 1u8 } else { 0u8 }])
            }
            _ => Err(Error::Postprocess("unsupported dtype".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::traits::Tensor;

    fn make_f32_tensor(vals: &[f32]) -> Tensor {
        let data: Vec<u8> = vals.iter().flat_map(|f| f.to_le_bytes()).collect();
        Tensor::new(data, vec![1, vals.len()], DType::F32)
    }

    #[tokio::test]
    async fn classify_above_threshold() {
        let c = Classify::new(0.5);
        let t = make_f32_tensor(&[0.1, 0.8, 0.1]);
        let result = c.process(&[t]).await.unwrap();
        assert_eq!(result, vec![1, 80]);
    }

    #[tokio::test]
    async fn classify_below_threshold() {
        let c = Classify::new(0.5);
        let t = make_f32_tensor(&[0.4, 0.3, 0.3]);
        let result = c.process(&[t]).await.unwrap();
        assert_eq!(result[0], 0);
    }

    #[tokio::test]
    async fn classify_empty_output() {
        let c = Classify::new(0.5);
        let result = c.process(&[]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn threshold_triggered() {
        let t = Threshold::new(50.0, 100.0);
        let tensor = make_f32_tensor(&[10.0, 75.0, 200.0]);
        let result = t.process(&[tensor]).await.unwrap();
        assert_eq!(result, vec![1]);
    }

    #[tokio::test]
    async fn threshold_not_triggered() {
        let t = Threshold::new(50.0, 100.0);
        let tensor = make_f32_tensor(&[10.0, 20.0, 30.0]);
        let result = t.process(&[tensor]).await.unwrap();
        assert_eq!(result, vec![0]);
    }

    #[tokio::test]
    async fn threshold_empty() {
        let t = Threshold::new(0.0, 1.0);
        let result = t.process(&[]).await;
        assert!(result.is_err());
    }

    #[test]
    fn classify_name() {
        assert_eq!(Classify::new(0.5).name(), "classify");
    }

    #[test]
    fn threshold_name() {
        assert_eq!(Threshold::new(0.0, 1.0).name(), "threshold");
    }
}
