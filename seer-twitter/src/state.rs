use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct TwitterState {
    pub db: DatabaseConnection,
}

impl TwitterState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
