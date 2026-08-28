use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct WebsitesState {
    pub db: DatabaseConnection,
}

impl WebsitesState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
