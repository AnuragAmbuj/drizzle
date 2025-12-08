use log::info;
use pingora::server::configuration::Opt;
use pingora::server::Server;
use proxy::{manager::SnapshotManager, GatewayProxy};
use snapshot::Snapshot;
use std::sync::Arc;

mod analytics;
use analytics::AnalyticsDb;

fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();
    info!("Initializing Drizzle Gatewayd...");

    // 1. Create the Pingora Server
    let mut server = Server::new(Some(Opt::default())).unwrap();
    server.bootstrap();

    // 2. Initialize Snapshot Manager with default snapshot
    let initial_snapshot = Snapshot::default();
    let manager = Arc::new(SnapshotManager::new(initial_snapshot));

    use authz::CedarPolicyEnforcer;
    let authz_enforcer = Arc::new(CedarPolicyEnforcer::new());

    // 3. Start Poller in a separate thread
    let poller_manager = manager.clone();
    let poller_manager_authz = authz_enforcer.clone();
    let admin_url =
        std::env::var("ADMIN_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let poller_authz = poller_manager_authz.clone();
        let poller = proxy::poller::SnapshotPoller::new(admin_url, poller_manager, poller_authz);

        runtime.block_on(async {
            // Poll every 10 seconds
            poller.run(std::time::Duration::from_secs(10)).await;
        });
    });

    use limits::TokenBucketRateLimiter;
    let rate_limiter = Arc::new(TokenBucketRateLimiter::new());
    
    // Initialize analytics channel
    let (analytics_tx, mut analytics_rx) = tokio::sync::mpsc::unbounded_channel();

    let mut my_proxy = pingora::proxy::http_proxy_service(
        &server.configuration,
        GatewayProxy {
            manager,
            authz: authz_enforcer.clone(),
            limits: rate_limiter.clone(),
            analytics_tx: Some(analytics_tx),
        },
    );

    // 5. Configure listener
    my_proxy.add_tcp("0.0.0.0:6188");
    info!("Gatewayd listening on 0.0.0.0:6188");

    // 6. Spawn Admin/Metrics Server (Port 9111)
    std::thread::spawn(move || {
         let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        
        runtime.block_on(async {
            // Init DB
            let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            let db = match AnalyticsDb::new(&db_url).await {
                Ok(d) => Arc::new(d),
                Err(e) => {
                    log::error!("Failed to open analytics db: {}", e);
                    // Fallback or panic? Let's panic to be safe for now, or just handle gracefully?
                    // Panic makes it obvious.
                    panic!("Failed to open DB: {}", e);
                }
            };
            
            // Spawn Writer Task
            let db_writer = db.clone();
            tokio::spawn(async move {
                while let Some(req) = analytics_rx.recv().await {
                     if let Err(e) = db_writer.insert(&req).await {
                         log::error!("Failed to write log: {}", e);
                     }
                }
            });

            let addr = "0.0.0.0:9111";
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            info!("Admin/Metrics server listening on {}", addr);

            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let db_reader = db.clone();
                
                tokio::spawn(async move {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};
                    
                    let mut buf = [0; 1024];
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    let req_str = String::from_utf8_lossy(&buf[..n]);
                    
                    let response = if req_str.contains("GET /metrics") {
                        use prometheus::Encoder;
                        let mut buffer = Vec::new();
                        let encoder = prometheus::TextEncoder::new();
                        let metric_families = prometheus::gather();
                        encoder.encode(&metric_families, &mut buffer).unwrap();
                        
                        let response_body = String::from_utf8(buffer).unwrap();
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                            response_body.len(),
                            response_body
                        )
                    } else if req_str.contains("GET /observability/logs") {
                        // TODO: Parse query params for limit/time?
                        // For now default to limit 100
                        let logs = db_reader.get_logs(100).await.unwrap_or_default();
                        let json = serde_json::to_string(&logs).unwrap_or_else(|_| "[]".to_string());
                         format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                            json.len(),
                            json
                        )
                    } else if req_str.contains("GET /observability/stats") {
                         // Default to last 24h or similar? 
                         // Let's just get all stats > 1 hour ago for now?
                         // "2024-01-01"
                         let start = chrono::Utc::now() - chrono::Duration::hours(1);
                         let stats = match db_reader.get_stats(start.to_rfc3339()).await {
                             Ok(s) => s,
                             Err(e) => {
                                 log::error!("Failed to get stats: {}", e);
                                 vec![]
                             }
                         };
                         
                         // Map to JSON structure
                         // Vec<(time, status_group, count)>
                         let json_structure: Vec<serde_json::Value> = stats.into_iter().map(|(time, status, count)| {
                            serde_json::json!({ "time": time, "status": status, "count": count })
                         }).collect();
                         
                         let json = serde_json::to_string(&json_structure).unwrap_or_else(|_| "[]".to_string());
                         format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                            json.len(),
                            json
                        )
                    } else if req_str.contains("GET /health") || req_str.contains("GET /ready") {
                         let body = "OK";
                         format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                            body.len(),
                            body
                        )
                    } else {
                         let body = "Not Found";
                         format!(
                             "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                            body.len(),
                            body
                        )
                    };

                    let _ = socket.write_all(response.as_bytes()).await;
                });
            }
        });
    });

    // 7. Register service and run
    server.add_service(my_proxy);
    server.run_forever();
}
