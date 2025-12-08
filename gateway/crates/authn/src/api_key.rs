use super::{Authenticator, Identity};
use async_trait::async_trait;
use http::Request;
use std::collections::HashMap;
use std::sync::Arc;

/// Authenticates requests using an API Key header.
pub struct ApiKeyAuthenticator {
    /// Map of API Key -> Identity
    keys: HashMap<String, Identity>,
    /// Header name to look for (default: "x-api-key")
    header_name: String,
}

impl ApiKeyAuthenticator {
    pub fn new(keys: HashMap<String, Identity>) -> Self {
        Self {
            keys,
            header_name: "x-api-key".to_string(),
        }
    }

    pub fn with_header(mut self, name: &str) -> Self {
        self.header_name = name.to_string();
        self
    }
}

#[async_trait]
impl Authenticator for ApiKeyAuthenticator {
    async fn authenticate<B>(&self, req: &Request<B>) -> anyhow::Result<Option<Identity>>
    where
        B: Send + Sync,
    {
        if let Some(value) = req.headers().get(&self.header_name) {
            if let Ok(key_str) = value.to_str() {
                if let Some(identity) = self.keys.get(key_str) {
                    return Ok(Some(identity.clone()));
                }
            }
        }

        // If header is missing, we return Ok(None) to allow other authenticators or anonymous access if configured.
        // If header is present but invalid, maybe we should error?
        // For now, let's say: if header matches but key is wrong -> Ok(None) (not found).
        // A strict mode could return Err.
        Ok(None)
    }
}
