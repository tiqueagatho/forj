use crate::error::{Error, Result};
use crate::inference::traits::Tensor;
use async_trait::async_trait;

/// Preprocessor transforms raw device data into inference-ready tensors.
#[async_trait]
pub trait Preprocessor: Send + Sync {
    fn name(&self) -> &'static str;
    async fn process(&self, raw: &[u8]) -> Result<Tensor>;
}

pub struct Normalize {
    pub mean: f32,
    pub std: f32,
}

impl Normalize {
    pub fn new(mean: f32, std: f32) -> Self {
        Self { mean, std }
    }
}

#[async_trait]
impl Preprocessor for Normalize {
    fn name(&self) -> &'static str {
        "normalize"
    }

    async fn process(&self, raw: &[u8]) -> Result<Tensor> {
        if raw.len() % 4 != 0 {
            return Err(Error::Preprocess("normalize expects f32 bytes".into()));
        }
        let out: Vec<f32> = raw
            .chunks_exact(4)
            .map(|c| {
                let val = f32::from_le_bytes([c[0], c[1], c[2], c[3]]);
                (val - self.mean) / self.std
            })
            .collect();
        let bytes: Vec<u8> = out.iter().flat_map(|f| f.to_le_bytes()).collect();
        let shape = vec![1, out.len()];
        Ok(Tensor::new(
            bytes,
            shape,
            crate::inference::traits::DType::F32,
        ))
    }
}

pub struct Resize {
    pub width: usize,
    pub height: usize,
}

impl Resize {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            width: w,
            height: h,
        }
    }
}

#[async_trait]
impl Preprocessor for Resize {
    fn name(&self) -> &'static str {
        "resize"
    }

    async fn process(&self, raw: &[u8]) -> Result<Tensor> {
        if raw.len() < self.width * self.height * 3 {
            return Err(Error::Preprocess("image data too small".into()));
        }
        // naive nearest-neighbor downscale for demo
        let step_x = (raw.len() / (self.width * 3)).max(1);
        let step_y = (raw.len() / (self.width * 3 * self.height)).max(1);
        let mut out = Vec::with_capacity(self.width * self.height * 3);
        for y in 0..self.height {
            for x in 0..self.width {
                let src_x = (x * step_x).min((raw.len() / 3) - 1);
                let src_y = (y * step_y).min((raw.len() / (step_x * 3)) - 1);
                let src_idx = (src_y * step_x * 3 + src_x * 3) % raw.len();
                out.extend_from_slice(&raw[src_idx..src_idx + 3]);
            }
        }
        Ok(Tensor::new(
            out,
            vec![1, 3, self.height, self.width],
            crate::inference::traits::DType::U8,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn normalize_valid() {
        let n = Normalize::new(0.0, 255.0);
        let raw: Vec<u8> = vec![0u8, 0, 0x80, 0x3f]; // f32 = 1.0
        let t = n.process(&raw).await.unwrap();
        assert_eq!(t.dtype, crate::inference::traits::DType::F32);
        let normalized: Vec<f32> = t
            .data
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        assert!((normalized[0] - (1.0 / 255.0)).abs() < 1e-6);
    }

    #[tokio::test]
    async fn normalize_invalid_len() {
        let n = Normalize::new(0.0, 1.0);
        let result = n.process(&[0u8, 1, 2]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn resize_valid() {
        let r = Resize::new(2, 2);
        // Enough data for 2x2 RGB
        let raw: Vec<u8> = (0..12).map(|i| i as u8).collect();
        let t = r.process(&raw).await.unwrap();
        assert_eq!(t.shape, vec![1, 3, 2, 2]);
    }

    #[tokio::test]
    async fn resize_too_small() {
        let r = Resize::new(100, 100);
        let result = r.process(&[0u8; 10]).await;
        assert!(result.is_err());
    }

    #[test]
    fn normalize_name() {
        let n = Normalize::new(0.0, 1.0);
        assert_eq!(n.name(), "normalize");
    }

    #[test]
    fn resize_name() {
        let r = Resize::new(224, 224);
        assert_eq!(r.name(), "resize");
    }
}
