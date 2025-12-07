use crate::manager::SnapshotManager;
use log::{error, info};
use snapshot::Snapshot;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct SnapshotPoller {
    client: reqwest::Client,
    admin_url: String,
    manager: Arc<SnapshotManager>,
}

impl SnapshotPoller {
    pub fn new(admin_url: String, manager: Arc<SnapshotManager>) -> Self {
        Self {
            client: reqwest::Client::new(),
            admin_url,
            manager,
        }
    }

    pub async fn run(&self, interval: Duration) {
        info!("Starting SnapshotPoller with interval {:?}", interval);
        loop {
            match self.fetch_snapshot().await {
                Ok(snapshot) => {
                    // Update the manager
                    let current_version = self.manager.get().version.clone();
                    if snapshot.version != current_version {
                        info!(
                            "Updated snapshot from {} to {}",
                            current_version, snapshot.version
                        );
                        self.manager.update(snapshot);
                    } else {
                        // For MVP admin-api generates new version every time,
                        // but if we had stability checks we'd log debug here.
                        // info!("Snapshot version {} is up to date", current_version);

                        // Because our MVP Admin API is stateless/random generation for now,
                        // we just blindly update to prove the pipes work.
                        self.manager.update(snapshot);
                    }
                }
                Err(e) => {
                    error!("Failed to fetch snapshot: {:?}", e);
                }
            }
            sleep(interval).await;
        }
    }

    async fn fetch_snapshot(&self) -> anyhow::Result<Snapshot> {
        let url = format!("{}/snapshot", self.admin_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Admin API returned {}", response.status()));
        }

        let snapshot = response.json::<Snapshot>().await?;
        Ok(snapshot)
    }
}
