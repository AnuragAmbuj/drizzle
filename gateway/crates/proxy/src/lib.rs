use crate::manager::SnapshotManager;
use crate::router::Router;
use async_trait::async_trait;
use pingora::prelude::*;
use pingora::proxy::{ProxyHttp, Session};
use pingora::upstreams::peer::HttpPeer;
use std::sync::Arc;
use url; // Added for URL parsing

pub mod manager;
pub mod poller;
pub mod router; // Added

pub struct GatewayProxy {
    pub manager: Arc<manager::SnapshotManager>, // Updated type
}

#[async_trait]
impl ProxyHttp for GatewayProxy {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {} // Updated to empty block

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
                // No match -> 404
                // We should ideally return a custom error or handle response here,
                // but Pingora expects a Result<Box<HttpPeer>>.
                // We can throw an error that Pingora catches, or pointer to a 404 backend?
                // Pingora allows sending response directly and returning Ok(None) or similar?
                // Actually, checking docs/examples: if we return Err, Pingora closes connection or 502s.
                // We can't easily send 404 here without `session.respond_error(404)`.
                // But `upstream_peer` signature doesn't technically allow stopping easily without error.
                // Let's return a specific error we can recognize later or just Log and error.
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
        // Use the first host for now (Random/Round-Robin would happen here in a real LB)
        let upstream_str = service.hosts.first().ok_or_else(|| {
            pingora::Error::explain(
                pingora::ErrorType::HTTPStatus(502),
                "Service has no healthy upstreams",
            )
        })?;

        // Simple heuristic: if it doesn't start with http/https, assume http://
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
        _upstream_request: &mut pingora::http::RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<()> {
        Ok(())
    }

    async fn request_filter(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        Ok(false)
    }
}
