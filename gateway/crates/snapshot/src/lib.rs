use chrono::{DateTime, Utc};
use domain::{
    policy::{LimitPolicy, Policy},
    route::Route,
    service::Service,
    tenant::Tenant,
};
use serde::{Deserialize, Serialize};

pub mod builder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Schema version (e.g., "snapshot/v1")
    pub schema: String,

    /// Unique version identifier for this snapshot (monotonic or time-based)
    pub version: String,

    /// SHA256 hash of the content (calculated before signing)
    pub sha256: String,

    // --- Data Blobs ---
    pub tenants: Vec<Tenant>,
    pub services: Vec<Service>,
    pub routes: Vec<Route>,
    pub policies: Vec<Policy>,
    pub limit_policies: Vec<LimitPolicy>,

    // --- Metadata ---
    pub metadata: SnapshotMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub issuer: String,
    pub issued_at: DateTime<Utc>,
    pub signature: String, // Base64 encoded signature
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            schema: "snapshot/v1".to_string(),
            version: "0.0.0".to_string(),
            sha256: String::new(),
            tenants: vec![],
            services: vec![],
            routes: vec![],
            policies: vec![],
            limit_policies: vec![],
            metadata: SnapshotMetadata {
                issuer: "unknown".to_string(),
                issued_at: Utc::now(),
                signature: String::new(),
            },
        }
    }
}
