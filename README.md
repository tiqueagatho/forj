# forj — Fusion Ops Runtime for Joint Industrial Intelligence

**Safe, concurrent, modular AI inference for Industry 4.0** — PLCs, sensors, cameras, robots.

[![CI](https://github.com/javier/forj/actions/workflows/ci.yml/badge.svg)](https://github.com/javier/forj/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue)](https://www.rust-lang.org)

---

## Problem

Industry 4.0 demands low-latency AI inference at the edge, but existing solutions
(Node-RED, Apache PLC4X, ROS2 Industrial) impose interpreters, GC pauses, and
runtime overhead that make sub-10ms deterministic decision loops difficult or
impossible.

## Solution

`forj` provides a modular, type-safe framework written in Rust
that combines **deterministic protocol I/O** with **pluggable AI inference** —
all without garbage collection, with zero-cost abstractions, and with memory
safety guaranteed at compile time.

### Comparison

|              | forj | Node-RED | PLC4X | ROS2 Industrial | EdgeX Foundry |
|--------------|-----------------------|----------|-------|-----------------|---------------|
| Language     | Rust                  | JS       | Java  | C++/Python      | Go            |
| Latency      | <10 ms                | ~50 ms   | ~30 ms| ~20 ms          | ~40 ms        |
| Memory safe  | ✅ (compile time)     | ❌       | ❌    | ❌              | ❌            |
| No GC        | ✅                    | ❌       | ❌    | ❌              | ✅            |
| AI inference | ✅ native             | ❌ addon | ❌    | ✅ (ROS2)      | ❌            |
| Async native | ✅ (Tokio)            | ✅       | ❌    | ❌              | ✅            |

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    Pipeline Orchestrator                  │
├─────────┬──────────┬──────────┬──────────┬───────────────┤
│  Source │ Preproc  │ Inference│ Postproc │     Sink      │
│ (Device)│ (Stage)  │ (Backend)│ (Stage)  │   (Device)    │
├─────────┴──────────┴──────────┴──────────┴───────────────┤
│           Protocol Layer (Modbus/OPC-UA/MQTT/...)         │
├──────────────────────────────────────────────────────────┤
│    Security (mTLS, RBAC, Audit)   │  Observability       │
├──────────────────────────────────────────────────────────┤
│              Realtime Scheduler (Tokio + priority)        │
└──────────────────────────────────────────────────────────┘
```

### Crate Modules

| Module         | Description                                   |
|----------------|-----------------------------------------------|
| `error`        | Typed error hierarchy (protocol, inference, pipeline, security, config) |
| `device`       | Industrial device abstraction (sensors, actuators, cameras) |
| `protocol`     | Protocol drivers (Modbus TCP/RTU, OPC-UA, MQTT) |
| `inference`    | Inference backends (ONNX Runtime, Tract, Candle, OpenVINO) with preprocessing/postprocessing |
| `pipeline`     | Pipeline orchestrator with backpressure, cancellation, and error propagation |
| `security`     | TLS, mTLS, RBAC, audit logging, anti-replay |
| `monitoring`   | Metrics, health checks, OpenTelemetry         |
| `realtime`     | Linux realtime scheduler integration (SCHED_FIFO/SCHED_RR) |
| `config`       | Declarative YAML/TOML/JSON configuration       |

### Supported Protocols

| Protocol    | Status      | Crate             |
|-------------|-------------|-------------------|
| Modbus TCP  | ✅ MVP      | `tokio-modbus`    |
| Modbus RTU  | ✅ MVP      | `tokio-modbus`    |
| OPC-UA      | ✅ MVP      | `opcua-client`    |
| MQTT        | ✅ MVP      | `rumqttc`         |
| CANopen     | 🧪 Preview  | `socketcan`       |
| Profinet    | 🧩 Contrib  | —                 |
| EtherNet/IP | 🧩 Contrib  | —                 |
| IO-Link     | 🧩 Contrib  | —                 |
| EtherCAT    | 🔮 Future   | —                 |

### Supported Inference Backends

| Backend       | Status      | Crate         |
|---------------|-------------|---------------|
| ONNX Runtime  | ✅ MVP      | `ort`         |
| Tract         | ✅ MVP      | `tract-onnx`  |
| Candle        | 🧪 Preview  | `candle-core` |
| OpenVINO      | 🧪 Preview  | `openvino`    |
| TensorFlow Lt | 🧩 Contrib  | —             |
| Burn          | 🔮 Future   | —             |

## Quick Start

```rust
use forj::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let plc = ModbusTcpClient::connect("192.168.1.5:502".parse()?).await?;
    let camera = Camera::connect("rtsp://192.168.1.10/stream").await?;
    let model = OnnxBackend::load("defect.onnx").await?;

    let pipeline = Pipeline::builder()
        .source(camera)
        .preprocess(Normalize::new(0.0, 255.0))
        .infer(model.into_dyn())
        .postprocess(Classify::new(0.5))
        .sink(plc)
        .backpressure(BackpressureConfig::bounded(256))
        .build()?;

    pipeline.run().await?;
    Ok(())
}
```

## Feature Flags

| Feature       | Description                           |
|---------------|---------------------------------------|
| `modbus-tcp`  | Modbus TCP client                     |
| `modbus-rtu`  | Modbus RTU (serial) client            |
| `opcua`       | OPC-UA client                         |
| `mqtt`        | MQTT 3.1.1 / 5.0 client               |
| `onnx`        | ONNX Runtime backend                  |
| `candle`      | Candle (pure-Rust) backend            |
| `tract`       | Tract inference backend               |
| `openvino`    | OpenVINO backend                      |
| `tls`         | TLS via rustls                        |
| `mtls`        | Mutual TLS with certificate loading   |
| `feat-tracing`| Structured logging via `tracing`      |
| `otel`        | OpenTelemetry integration             |
| `prometheus`  | Prometheus metrics                    |
| `realtime`    | Linux realtime scheduler (SCHED_FIFO) |
| `full`        | All features                          |

## Performance Design

- **Zero-copy** data paths: Sensors → preprocessor avoids allocations via `bytes::Bytes` and pooling.
- **SIMD** acceleration: Candle and Tract leverage SIMD through their backends.
- **Pre-allocated buffer pools**: Pipeline stages reuse tensor buffers.
- **Batch processing**: Inflight frames batched before inference.
- **Backpressure**: Bounded channels with configurable drop/block strategies.
- **Latency target**: <10 ms end-to-end.

## Testing Strategy

| Type             | Tool             | Scope              |
|------------------|------------------|--------------------|
| Unit             | `#[cfg(test)]`   | Per-module         |
| Integration      | `tests/`         | End-to-end         |
| Property         | `proptest`       | Serialization      |
| Fuzz             | `cargo fuzz`     | Protocol parsing   |
| Benchmark        | `criterion`      | Pipeline throughput |

## Roadmap

- **0.1 (MVP)**: Modbus TCP, ONNX + Tract backends, pipeline, cfg → *this release*
- **0.2**: OPC-UA, MQTT, TLS, tracing
- **0.5**: CANopen, Candle backends, RBAC, audit
- **0.8**: Profinet, OpenVINO, batch pipeline, zero-copy pools
- **1.0**: EtherCAT, safety certification documentation, formal verification

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual-licensed as above, without any
additional terms or conditions.
