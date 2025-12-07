use log::info;
use pingora::server::configuration::Opt;
use pingora::server::Server;
use proxy::{manager::SnapshotManager, GatewayProxy};
use snapshot::Snapshot;
use std::sync::Arc;

fn main() {
    env_logger::init();
    info!("Initializing Drizzle Gatewayd...");

    // 1. Create the Pingora Server
    let mut server = Server::new(Some(Opt::default())).unwrap();
    server.bootstrap();

    // 2. Initialize Snapshot Manager with default snapshot
    let initial_snapshot = Snapshot::default();
    let manager = Arc::new(SnapshotManager::new(initial_snapshot));

    // 3. Start Poller in a separate thread
    let poller_manager = manager.clone();
    let admin_url =
        std::env::var("ADMIN_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let poller = proxy::poller::SnapshotPoller::new(admin_url, poller_manager);

        runtime.block_on(async {
            // Poll every 10 seconds
            poller.run(std::time::Duration::from_secs(10)).await;
        });
    });

    // 4. create the proxy service
    let mut my_proxy =
        pingora::proxy::http_proxy_service(&server.configuration, GatewayProxy { manager });

    // 5. Configure listener
    my_proxy.add_tcp("0.0.0.0:6188");
    info!("Gatewayd listening on 0.0.0.0:6188");

    // 6. Register service and run
    server.add_service(my_proxy);
    server.run_forever();
}
