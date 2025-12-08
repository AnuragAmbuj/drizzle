use async_trait::async_trait;
use http::Request;

#[derive(Debug, Clone)]
pub struct Identity {
    pub subject: String,
    pub claims: serde_json::Value,
}

#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Authenticate the incoming request.
    /// Returns Ok(Some(Identity)) on success.
    /// Returns Ok(None) if authentication is optional/skipped (or should fallback).
    /// Returns Err if authentication fails (invalid credential).
    async fn authenticate<B>(&self, req: &Request<B>) -> anyhow::Result<Option<Identity>>
    where
        B: Send + Sync;
}

pub mod api_key;
