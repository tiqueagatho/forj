use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub struct MetricsCollector {
    counters: Arc<RwLock<HashMap<String, u64>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    histograms: Arc<RwLock<HashMap<String, Vec<f64>>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn increment(&self, name: &str) {
        *self.counters.write().entry(name.into()).or_insert(0) += 1;
    }

    pub fn gauge(&self, name: &str, value: f64) {
        self.gauges.write().insert(name.into(), value);
    }

    pub fn observe(&self, name: &str, value: f64) {
        self.histograms
            .write()
            .entry(name.into())
            .or_default()
            .push(value);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            counters: self.counters.read().clone(),
            gauges: self.gauges.read().clone(),
            histogram_count: self
                .histograms
                .read()
                .iter()
                .map(|(k, v)| (k.clone(), v.len()))
                .collect(),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MetricsSnapshot {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histogram_count: HashMap<String, usize>,
}
