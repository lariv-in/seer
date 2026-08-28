use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct OpenskyState {
    pub db: DatabaseConnection,
}

impl OpenskyState {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
