use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct GdeltState {
    pub db: DatabaseConnection,
}

impl GdeltState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
