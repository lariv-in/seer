use sea_orm::DatabaseConnection;
#[derive(Clone)]
pub struct RedditState { pub db: DatabaseConnection }
impl RedditState { pub fn new(db: DatabaseConnection) -> Self { Self { db } } }
