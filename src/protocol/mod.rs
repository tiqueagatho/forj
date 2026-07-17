pub mod common;
pub mod modbus;
pub mod traits;

use crate::error::{Error, Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

type DriverFactory = Box<dyn Fn() -> Box<dyn traits::ErasedDriver> + Send>;
static REGISTRY: Lazy<Mutex<HashMap<&'static str, DriverFactory>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn register(name: &'static str, factory: DriverFactory) {
    REGISTRY.lock().unwrap().insert(name, factory);
}

pub fn create(name: &str) -> Result<Box<dyn traits::ErasedDriver>> {
    let guard = REGISTRY.lock().unwrap();
    guard
        .get(name)
        .map(|f| f())
        .ok_or_else(|| Error::UnsupportedProtocol(name.to_owned()))
}
