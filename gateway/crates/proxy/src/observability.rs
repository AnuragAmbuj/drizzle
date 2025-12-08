use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentRequest {
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String, // Full URL or path
    pub status: u16,
    pub duration_ms: f64,
    pub tenant_id: String,
}

pub type RequestRingBuffer = Arc<RwLock<VecDeque<RecentRequest>>>;

pub fn new_ring_buffer() -> RequestRingBuffer {
    Arc::new(RwLock::new(VecDeque::with_capacity(100)))
}
