use crate::device::DynIndustrialDevice;
use crate::error::{Error, Result};
use crate::inference::postprocessor::Postprocessor;
use crate::inference::preprocessor::Preprocessor;
use crate::inference::traits::DynInferenceBackend;
use crate::pipeline::{BackpressureConfig, Pipeline, PipelineInner, Sink, Source};
use async_trait::async_trait;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub struct PipelineBuilder {
    source: Option<Box<dyn Source>>,
    pre: Option<Box<dyn Preprocessor>>,
    model: Option<Box<dyn DynInferenceBackend>>,
    post: Option<Box<dyn Postprocessor>>,
    sink: Option<Box<dyn Sink>>,
    backpressure: BackpressureConfig,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            source: None,
            pre: None,
            model: None,
            post: None,
            sink: None,
            backpressure: BackpressureConfig::default(),
        }
    }

    pub fn source(mut self, device: impl DynIndustrialDevice + 'static) -> Self {
        self.source = Some(Box::new(DeviceSource(Arc::new(device))));
        self
    }

    pub fn source_raw(mut self, source: impl Source + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    pub fn preprocess(mut self, p: impl Preprocessor + 'static) -> Self {
        self.pre = Some(Box::new(p));
        self
    }

    pub fn infer(mut self, backend: impl DynInferenceBackend + 'static) -> Self {
        self.model = Some(Box::new(backend));
        self
    }

    pub fn postprocess(mut self, p: impl Postprocessor + 'static) -> Self {
        self.post = Some(Box::new(p));
        self
    }

    pub fn sink(mut self, device: impl DynIndustrialDevice + 'static) -> Self {
        self.sink = Some(Box::new(DeviceSink(Arc::new(device))));
        self
    }

    pub fn sink_raw(mut self, sink: impl Sink + 'static) -> Self {
        self.sink = Some(Box::new(sink));
        self
    }

    pub fn backpressure(mut self, cfg: BackpressureConfig) -> Self {
        self.backpressure = cfg;
        self
    }

    pub fn build(self) -> Result<Pipeline> {
        let source = self
            .source
            .ok_or(Error::Config("pipeline: no source".into()))?;
        let model = self
            .model
            .ok_or(Error::Config("pipeline: no model".into()))?;
        let sink = self.sink.ok_or(Error::Config("pipeline: no sink".into()))?;
        Ok(Pipeline {
            inner: Arc::new(PipelineInner {
                source,
                pre: self.pre,
                model,
                post: self.post,
                sink,
                backpressure: self.backpressure,
            }),
            cancel: CancellationToken::new(),
        })
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct DeviceSource(Arc<dyn DynIndustrialDevice>);

#[async_trait]
impl Source for DeviceSource {
    fn name(&self) -> &'static str {
        "device-source"
    }
    async fn poll(&self) -> crate::error::Result<Vec<u8>> {
        self.0.dyn_read().await
    }
}

struct DeviceSink(Arc<dyn DynIndustrialDevice>);

#[async_trait]
impl Sink for DeviceSink {
    fn name(&self) -> &'static str {
        "device-sink"
    }
    async fn send(&self, data: Vec<u8>) -> crate::error::Result<()> {
        self.0.dyn_write(data).await
    }
}
