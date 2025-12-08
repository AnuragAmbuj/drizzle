use crate::router::Router;
use async_trait::async_trait;
use authn::Identity;
use http::header::HeaderValue;
use pingora::prelude::*;
use pingora::proxy::{ProxyHttp, Session};
use pingora::upstreams::peer::HttpPeer;
use std::sync::Arc;
use url;

pub mod manager;
pub mod poller;
pub mod router;

use authz::{CedarPolicyEnforcer, PolicyEnforcer};
use limits::{RateLimiter, TokenBucketRateLimiter};

pub struct GatewayProxy {
    pub manager: Arc<manager::SnapshotManager>,
    pub authz: Arc<CedarPolicyEnforcer>,
    pub limits: Arc<TokenBucketRateLimiter>,
}

pub struct ProxyContext {
    pub identity: Option<Identity>,
}

#[async_trait]
impl ProxyHttp for GatewayProxy {
    type CTX = ProxyContext;

    fn new_ctx(&self) -> Self::CTX {
        ProxyContext { identity: None }
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        // 1. Get Snapshot
        let snapshot = self.manager.get();

        // 2. Perform Authentication (API Key)
        // For MVP, we reconstruct it from snapshot keys or just check map directly.

        // Check headers manually here since I know it's API Key.
        let headers = &session.req_header().headers;
        if let Some(val) = headers.get("x-api-key") {
            if let Ok(key_str) = val.to_str() {
                if let Some(identity_subject) = snapshot.api_keys.get(key_str) {
                    // Authenticated!
                    ctx.identity = Some(Identity {
                        subject: identity_subject.clone(),
                        claims: serde_json::Value::Null,
                    });
                    // println!("Authenticated: {}", identity_subject);
                } else {
                    // Key present but invalid -> 401
                    session.respond_error(401).await?;
                    return Ok(true); // Stop processing
                }
            }
        } else {
            // No key provided -> Anonymous
        }

        // 3. Perform Authorization (Cedar)
        let method = session.req_header().method.as_str();
        // Resource is fixed "api" for MVP, or we could use the path?
        // Let's use "api" to match a simple policy like:
        // permit(principal, action, resource == Resource::"api");
        let resource = "api";

        let allowed = self
            .authz
            .authorize(ctx.identity.as_ref(), method, resource)
            .await
            .map_err(|e| {
                pingora::Error::explain(pingora::ErrorType::InternalError, e.to_string())
            })?;

        if !allowed {
            session.respond_error(403).await?;
            return Ok(true);
        }

        // 4. Perform Rate Limiting
        // Strategy: Find first limit policy for this tenant (MVP)
        // Ideally we match specific policy to route/tier.
        // For now, if we have an identity, check if there's a limit policy for its tenant.

        if let Some(identity) = &ctx.identity {
            // Find tenant ID?
            // Snapshot api_keys maps Key -> Subject (User ID or Tenant ID).
            // We need to resolve Subject to Tenant ID if it's a User ID, or assume it is Tenant ID.
            // For created keys, we stored tenant_id as subject (in `api_keys` table -> snapshot map).

            // Find a LimitPolicy for this tenant
            if let Some(limit_policy) = snapshot
                .limit_policies
                .iter()
                .find(|p| p.tenant_id.to_string() == identity.subject)
            {
                // Key for limiter: "tenant:{id}"
                let limit_key = format!("tenant:{}", identity.subject);

                let allowed = self
                    .limits
                    .check(&limit_key, limit_policy)
                    .await
                    .map_err(|e| {
                        pingora::Error::explain(pingora::ErrorType::InternalError, e.to_string())
                    })?;

                if !allowed {
                    session.respond_error(429).await?;
                    return Ok(true);
                }
            }
        }

        Ok(false) // Continue
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let snapshot = self.manager.get();
        // 1. Match Route
        let route = match Router::match_request(&snapshot, session) {
            Some(r) => r,
            None => {
                return Err(pingora::Error::explain(
                    pingora::ErrorType::HTTPStatus(404),
                    "No matching route",
                ));
            }
        };

        // 2. Find Service
        let service = snapshot.services.iter().find(|s| s.id == route.service_id);

        let service = match service {
            Some(s) => s,
            None => {
                return Err(pingora::Error::explain(
                    pingora::ErrorType::HTTPStatus(500),
                    "Service not found for route",
                ));
            }
        };

        // 3. Construct Peer
        let upstream_str = service.hosts.first().ok_or_else(|| {
            pingora::Error::explain(
                pingora::ErrorType::HTTPStatus(502),
                "Service has no healthy upstreams",
            )
        })?;

        let upstream_url = if upstream_str.contains("://") {
            upstream_str.to_string()
        } else {
            format!("http://{}", upstream_str)
        };

        let url = url::Url::parse(&upstream_url).map_err(|e| {
            pingora::Error::explain(
                pingora::ErrorType::InternalError,
                format!("Invalid upstream URL: {}", e),
            )
        })?;

        let host = url.host_str().unwrap_or("localhost");
        let port = url.port_or_known_default().unwrap_or(80);
        let addr = (host, port);

        let tls = url.scheme() == "https";
        let sni = host.to_string();

        let peer = Box::new(HttpPeer::new(addr, tls, sni));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut pingora::http::RequestHeader,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<()> {
        if let Some(identity) = &ctx.identity {
            let val = HeaderValue::from_str(&identity.subject).map_err(|e| {
                pingora::Error::explain(pingora::ErrorType::InternalError, e.to_string())
            })?;
            upstream_request.insert_header("X-Drizzle-User", val)?;
        }
        Ok(())
    }
}
