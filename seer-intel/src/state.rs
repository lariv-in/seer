use sea_orm::DatabaseConnection;

use crate::ingest;

#[derive(Clone)]
pub struct IntelState {
    pub db: DatabaseConnection,
}

impl IntelState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn spawn_ingest_worker(&self) {
        ingest::start_intel_ingest_worker(self.db.clone());
    }

    pub fn enqueue(&self, kind: crate::kind::DynIntelKind) {
        ingest::enqueue_intel(kind);
    }
}
