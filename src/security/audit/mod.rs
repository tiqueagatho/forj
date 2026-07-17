use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub user: String,
    pub action: String,
    pub resource: String,
    pub result: String,
    pub detail: Option<String>,
}

pub struct AuditTrail {
    entries: Arc<RwLock<Vec<AuditEntry>>>,
}

impl AuditTrail {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn record(
        &self,
        user: &str,
        action: &str,
        resource: &str,
        result: &str,
        detail: Option<String>,
    ) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            user: user.into(),
            action: action.into(),
            resource: resource.into(),
            result: result.into(),
            detail,
        };
        self.entries.write().push(entry);
    }

    pub fn recent(&self, n: usize) -> Vec<AuditEntry> {
        let guard = self.entries.read();
        guard.iter().rev().take(n).cloned().collect()
    }

    pub fn all(&self) -> Vec<AuditEntry> {
        self.entries.read().clone()
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}
