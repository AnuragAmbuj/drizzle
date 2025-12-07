use arc_swap::ArcSwap;
use snapshot::Snapshot;
use std::sync::Arc;

pub struct SnapshotManager {
    current: ArcSwap<Snapshot>,
}

impl SnapshotManager {
    pub fn new(initial: Snapshot) -> Self {
        Self {
            current: ArcSwap::from_pointee(initial),
        }
    }

    pub fn update(&self, new_snapshot: Snapshot) {
        self.current.store(Arc::new(new_snapshot));
    }

    pub fn get(&self) -> Arc<Snapshot> {
        self.current.load().clone()
    }
}
