use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct WorkerRegistryState {
    pub db: DatabaseConnection,
}

impl WorkerRegistryState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
