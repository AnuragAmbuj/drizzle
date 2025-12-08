use crate::Snapshot;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use domain::{
    policy::{LimitPolicy, Policy},
    route::Route,
    service::Service,
    tenant::Tenant,
};
use sha2::{Digest, Sha256};

pub struct SnapshotBuilder {
    snapshot: Snapshot,
}

impl SnapshotBuilder {
    pub fn new(version: String, issuer: String) -> Self {
        let mut snapshot = Snapshot::default();
        snapshot.version = version;
        snapshot.metadata.issuer = issuer;
        snapshot.metadata.issued_at = Utc::now();
        Self { snapshot }
    }

    pub fn with_tenants(mut self, tenants: Vec<Tenant>) -> Self {
        self.snapshot.tenants = tenants;
        self
    }

    pub fn with_services(mut self, services: Vec<Service>) -> Self {
        self.snapshot.services = services;
        self
    }

    pub fn with_routes(mut self, routes: Vec<Route>) -> Self {
        self.snapshot.routes = routes;
        self
    }

    pub fn with_policies(mut self, policies: Vec<Policy>) -> Self {
        self.snapshot.policies = policies;
        self
    }

    pub fn with_limits(mut self, limits: Vec<LimitPolicy>) -> Self {
        self.snapshot.limit_policies = limits;
        self
    }

    pub fn with_api_keys(mut self, api_keys: std::collections::HashMap<String, String>) -> Self {
        self.snapshot.api_keys = api_keys;
        self
    }

    /// Finalizes the snapshot by calling hash and "signing" it.
    /// In a real system, you'd pass a KeyPair here.
    pub fn build(mut self) -> anyhow::Result<Snapshot> {
        // 1. Calculate SHA256 of the data content
        // Note: In an ideal world we'd hash *only* the data fields, not the whole default struct with empty metadata.
        // For MVP, we'll hash the current state (minus hash/sig) or just hash the "data" tuple.
        // Let's stick to simple: canonicalize by serializing data vectors.

        let data_tuple = (
            &self.snapshot.tenants,
            &self.snapshot.services,
            &self.snapshot.routes,
            &self.snapshot.policies,
            &self.snapshot.limit_policies,
            &self.snapshot.api_keys,
        );
        let canonical_json = serde_json::to_string(&data_tuple)?;

        let mut hasher = Sha256::new();
        hasher.update(canonical_json.as_bytes());
        let hash = hasher.finalize();
        self.snapshot.sha256 = hex::encode(hash);

        // 2. Mock Signature (sign the hash)
        // In real life: sign(private_key, self.snapshot.sha256)
        let signature = format!("mock-sig-of-{}", self.snapshot.sha256);
        self.snapshot.metadata.signature = STANDARD.encode(signature);

        Ok(self.snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::tenant::Tenant;

    #[test]
    fn test_builder_hashes_deterministically() {
        let tenant = Tenant::new("acme".to_string(), "ACME".to_string());

        let snap1 = SnapshotBuilder::new("v1".to_string(), "admin".to_string())
            .with_tenants(vec![tenant.clone()])
            .build()
            .unwrap();

        let snap2 = SnapshotBuilder::new("v2".to_string(), "admin".to_string())
            .with_tenants(vec![tenant.clone()])
            .build()
            .unwrap();

        // Hashes should be identical because content (tenants) is identical,
        // even if versions differ (builder hash logic only includes content).
        assert_eq!(snap1.sha256, snap2.sha256);
        assert!(!snap1.sha256.is_empty());
    }
}
