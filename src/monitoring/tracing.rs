#[cfg(feature = "feat-tracing")]
pub use tracing::{debug, error, info, span, warn, Level, Span};

/// Initialize tracing subscriber.
#[cfg(feature = "feat-tracing")]
pub fn init_tracing(service_name: &str) {
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_file(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    info!(service = %service_name, "tracing initialized");
}

/// Stub when tracing feature is off.
#[cfg(not(feature = "feat-tracing"))]
pub fn init_tracing(_service_name: &str) {}

/// Spans for industrial operations.
pub fn industrial_span(_device: &str, _operation: &str) {
    // placeholder — tracing spans gated behind feat-tracing
}

#[cfg(not(feature = "feat-tracing"))]
pub use std::{eprintln as warn, eprintln as error, eprintln as debug, println as info};
