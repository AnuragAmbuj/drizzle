use crate::router::Router;
use async_trait::async_trait;
use authn::Identity;
use http::header::HeaderValue;
use pingora::prelude::*;
use pingora::proxy::{ProxyHttp, Session};
use pingora::upstreams::peer::HttpPeer;
use std::sync::Arc;
use std::time::Instant;
use url;

use crate::metrics::{HTTP_REQUESTS_TOTAL, HTTP_REQUEST_DURATION_SECONDS};

pub mod manager;
pub mod metrics;
pub mod observability;
pub mod poller;
pub mod router;

use authz::{CedarPolicyEnforcer, PolicyEnforcer};
use limits::{RateLimiter, TokenBucketRateLimiter};

pub struct GatewayProxy {
    pub manager: Arc<manager::SnapshotManager>,
    pub authz: Arc<CedarPolicyEnforcer>,
    pub limits: Arc<TokenBucketRateLimiter>,
    pub analytics_tx: Option<tokio::sync::mpsc::UnboundedSender<observability::RecentRequest>>,
}

pub struct ProxyContext {
    pub identity: Option<Identity>,
    pub start_time: Instant,
}

#[async_trait]
impl ProxyHttp for GatewayProxy {
    type CTX = ProxyContext;

    fn new_ctx(&self) -> Self::CTX {
        ProxyContext {
            identity: None,
            start_time: Instant::now(),
        }
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        // 1. Get Snapshot
        let snapshot = self.manager.get();

        // 2. Perform Authentication (API Key)
        let headers = &session.req_header().headers;
        if let Some(val) = headers.get("x-api-key") {
            if let Ok(key_str) = val.to_str() {
                if let Some(identity_subject) = snapshot.api_keys.get(key_str) {
                    ctx.identity = Some(Identity {
                        subject: identity_subject.clone(),
                        claims: serde_json::Value::Null,
                    });
                } else {
                    session.respond_error(401).await?;
                    return Ok(true);
                }
            }
        }

        // 3. Perform Authorization (Cedar)
        let method = session.req_header().method.as_str();
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

        // 4. Rate Limiting
        if let Some(identity) = &ctx.identity {
            if let Some(limit_policy) = snapshot
                .limit_policies
                .iter()
                .find(|p| p.tenant_id.to_string() == identity.subject)
            {
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

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let snapshot = self.manager.get();
        let route = Router::match_request(&snapshot, session).ok_or_else(|| {
            pingora::Error::explain(pingora::ErrorType::HTTPStatus(404), "No matching route")
        })?;
        let service = snapshot
            .services
            .iter()
            .find(|s| s.id == route.service_id)
            .ok_or_else(|| {
                pingora::Error::explain(pingora::ErrorType::HTTPStatus(500), "Service not found")
            })?;
        let upstream_str = service.hosts.first().ok_or_else(|| {
            pingora::Error::explain(pingora::ErrorType::HTTPStatus(502), "No upstreams")
        })?;

        // Simplified peer construction for brevity in this replace block (keeping existing logic mostly)
        let upstream_url = if upstream_str.contains("://") {
            upstream_str.to_string()
        } else {
            format!("http://{}", upstream_str)
        };
        let url = url::Url::parse(&upstream_url).map_err(|e| {
            pingora::Error::explain(pingora::ErrorType::InternalError, e.to_string())
        })?;
        let peer = Box::new(HttpPeer::new(
            (
                url.host_str().unwrap_or("localhost"),
                url.port_or_known_default().unwrap_or(80),
            ),
            url.scheme() == "https",
            url.host_str().unwrap_or("localhost").to_string(),
        ));
        Ok(peer)
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        let duration = ctx.start_time.elapsed().as_secs_f64();
        let status = session
            .response_written()
            .map(|resp| resp.status.as_u16())
            .unwrap_or(0);
        let method = session.req_header().method.as_str().to_string();
        let path = session.req_header().uri.to_string();
        let tenant_id = ctx
            .identity
            .as_ref()
            .map(|id| id.subject.clone())
            .unwrap_or_else(|| "anonymous".to_string());

        HTTP_REQUESTS_TOTAL
            .with_label_values(&[&method, &status.to_string(), &tenant_id])
            .inc();
        HTTP_REQUEST_DURATION_SECONDS
            .with_label_values(&[&method, &status.to_string()])
            .observe(duration);

        if let Some(tx) = &self.analytics_tx {
            let entry = observability::RecentRequest {
                timestamp: chrono::Utc::now(),
                method,
                path,
                status,
                duration_ms: duration * 1000.0,
                tenant_id,
            };
            let _ = tx.send(entry);
        }
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
