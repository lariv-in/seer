use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AisstreamState {
    pub db: DatabaseConnection,
}

impl AisstreamState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
