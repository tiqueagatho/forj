//! Simple ergonomic wrappers for common industrial patterns.
//!
//! This module provides opinionated but easy-to-use constructors for
//! typical Industry 4.0 use cases.

use crate::error::{Result, Error};
use crate::config::RuntimeConfig;
use crate::pipeline::Pipeline;

impl RuntimeConfig {
    /// Build a complete pipeline from config file.
    pub async fn build_pipeline(&self) -> Result<Pipeline> {
        let _ = self;
        Err(Error::Other("use Pipeline::builder() for now".into()))
    }
}
