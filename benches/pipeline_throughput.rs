use async_trait::async_trait;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use forj::error::Result;
use forj::pipeline::{BackpressureConfig, Pipeline, PipelineBuilder};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct BenchSource {
    data: Vec<u8>,
    count: Arc<AtomicU64>,
}

#[async_trait]
impl forj::pipeline::Source for BenchSource {
    fn name(&self) -> &'static str {
        "bench-source"
    }
    async fn poll(&self) -> Result<Vec<u8>> {
        self.count.fetch_add(1, Ordering::Relaxed);
        Ok(self.data.clone())
    }
}

struct BenchSink;

#[async_trait]
impl forj::pipeline::Sink for BenchSink {
    fn name(&self) -> &'static str {
        "bench-sink"
    }
    async fn send(&self, data: Vec<u8>) -> Result<()> {
        black_box(&data);
        Ok(())
    }
}

fn pipeline_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("pipeline_throughput", |b| {
        b.to_async(&rt).iter(|| async {
            let data = vec![0u8; 1024];
            let pipeline = Pipeline::builder()
                .source_raw(BenchSource {
                    data,
                    count: Arc::new(AtomicU64::new(0)),
                })
                .sink_raw(BenchSink)
                .backpressure(BackpressureConfig::bounded(1024))
                .build()
                .unwrap();

            // run for 100ms then cancel
            let handle = tokio::spawn(async move { pipeline.run().await });
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            handle.abort();
        })
    });
}

criterion_group!(benches, pipeline_throughput);
criterion_main!(benches);
