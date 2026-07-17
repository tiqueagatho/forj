pub mod builder;
pub mod fs;
pub mod traits;

use crate::error::Result;
use crate::inference::postprocessor::Postprocessor;
use crate::inference::preprocessor::Preprocessor;
use crate::inference::traits::{DynInferenceBackend, Tensor};
use async_trait::async_trait;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct Pipeline {
    pub(crate) inner: Arc<PipelineInner>,
    pub(crate) cancel: CancellationToken,
}

pub(crate) struct PipelineInner {
    pub source: Box<dyn Source>,
    pub pre: Option<Box<dyn Preprocessor>>,
    pub model: Box<dyn DynInferenceBackend>,
    pub post: Option<Box<dyn Postprocessor>>,
    pub sink: Box<dyn Sink>,
    pub backpressure: BackpressureConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct BackpressureConfig {
    pub capacity: usize,
}

impl BackpressureConfig {
    pub fn bounded(cap: usize) -> Self {
        Self { capacity: cap }
    }
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self { capacity: 1024 }
    }
}

#[async_trait]
pub trait Source: Send + Sync {
    fn name(&self) -> &'static str;
    async fn poll(&self) -> Result<Vec<u8>>;
}

#[async_trait]
pub trait Sink: Send + Sync {
    fn name(&self) -> &'static str;
    async fn send(&self, data: Vec<u8>) -> Result<()>;
}

impl Pipeline {
    pub fn builder() -> builder::PipelineBuilder {
        builder::PipelineBuilder::new()
    }

    pub async fn run(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(self.inner.backpressure.capacity);
        let cancel = self.cancel.clone();
        let cancel2 = self.cancel.clone();

        let inner2 = self.inner.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    result = inner2.source.poll() => {
                        match result {
                            Ok(data) => { let _ = tx.send(data).await; }
                            Err(_) => { tokio::time::sleep(std::time::Duration::from_millis(10)).await; }
                        }
                    }
                }
            }
        });

        let inner3 = self.inner.clone();
        let cancel3 = self.cancel.clone();
        let stream = async_stream::stream! {
            while let Some(raw) = rx.recv().await {
                if cancel3.is_cancelled() { break; }
                let data = if let Some(ref pre) = inner3.pre {
                    match pre.process(&raw).await {
                        Ok(t) => t,
                        Err(_) => continue,
                    }
                } else {
                    Tensor::new(raw, vec![1], crate::inference::traits::DType::U8)
                };
                let result = match inner3.model.infer(&[data]).await {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let decision = if let Some(ref post) = inner3.post {
                    match post.process(&result.outputs).await {
                        Ok(d) => d,
                        Err(_) => continue,
                    }
                } else {
                    result.outputs.first().map(|t| t.data.clone()).unwrap_or_default()
                };
                if inner3.sink.send(decision).await.is_err() { break; }
                yield ();
            }
        };

        let _ = stream.collect::<Vec<_>>().await;
        cancel2.cancel();
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancel.cancel();
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::inference::postprocessor::Classify;
    use crate::inference::preprocessor::Normalize;
    use crate::inference::traits::{DType, InferenceOutput};
    use async_trait::async_trait;

    struct TestSource;
    #[async_trait]
    impl Source for TestSource {
        fn name(&self) -> &'static str {
            "test-source"
        }
        async fn poll(&self) -> Result<Vec<u8>> {
            Ok(vec![0u8; 16])
        }
    }

    struct TestSink;
    #[async_trait]
    impl Sink for TestSink {
        fn name(&self) -> &'static str {
            "test-sink"
        }
        async fn send(&self, _data: Vec<u8>) -> Result<()> {
            Ok(())
        }
    }

    struct TestBackend;
    #[async_trait]
    impl DynInferenceBackend for TestBackend {
        fn name(&self) -> &'static str {
            "test"
        }
        async fn infer(&self, _input: &[Tensor]) -> Result<InferenceOutput> {
            Ok(InferenceOutput {
                outputs: vec![Tensor::new(
                    vec![0u8, 0x80, 0x3f], // 1.0f32 as le bytes
                    vec![1, 1],
                    DType::F32,
                )],
                latency_us: 0,
            })
        }
    }

    #[tokio::test]
    async fn test_builder_valid() {
        let p = Pipeline::builder()
            .source_raw(TestSource)
            .infer(TestBackend)
            .sink_raw(TestSink)
            .build();
        assert!(p.is_ok());
    }

    #[tokio::test]
    async fn test_builder_missing_source() {
        let p = Pipeline::builder()
            .infer(TestBackend)
            .sink_raw(TestSink)
            .build();
        assert!(matches!(p, Err(Error::Config(_))));
    }

    #[tokio::test]
    async fn test_builder_missing_model() {
        let p = Pipeline::builder()
            .source_raw(TestSource)
            .sink_raw(TestSink)
            .build();
        assert!(matches!(p, Err(Error::Config(_))));
    }

    #[tokio::test]
    async fn test_builder_missing_sink() {
        let p = Pipeline::builder()
            .source_raw(TestSource)
            .infer(TestBackend)
            .build();
        assert!(matches!(p, Err(Error::Config(_))));
    }

    #[tokio::test]
    async fn test_pipeline_run_cancel() {
        let p = Pipeline::builder()
            .source_raw(TestSource)
            .infer(TestBackend)
            .sink_raw(TestSink)
            .build()
            .unwrap();
        p.cancel();
        let result = p.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pipeline_with_pre_post() {
        let p = Pipeline::builder()
            .source_raw(TestSource)
            .preprocess(Normalize::new(0.0, 255.0))
            .infer(TestBackend)
            .postprocess(Classify::new(0.5))
            .sink_raw(TestSink)
            .backpressure(BackpressureConfig::bounded(16))
            .build()
            .unwrap();
        p.cancel();
        let result = p.run().await;
        assert!(result.is_ok());
    }

    #[test]
    fn backpressure_default() {
        let cfg = BackpressureConfig::default();
        assert_eq!(cfg.capacity, 1024);
    }

    #[test]
    fn backpressure_bounded() {
        let cfg = BackpressureConfig::bounded(64);
        assert_eq!(cfg.capacity, 64);
    }

    #[test]
    fn pipeline_drop_cancels() {
        let p = Pipeline {
            inner: Arc::new(PipelineInner {
                source: Box::new(TestSource),
                pre: None,
                model: Box::new(TestBackend),
                post: None,
                sink: Box::new(TestSink),
                backpressure: BackpressureConfig::default(),
            }),
            cancel: CancellationToken::new(),
        };
        assert!(!p.cancel.is_cancelled());
        drop(p);
        // Token is cancelled on drop
    }
}
