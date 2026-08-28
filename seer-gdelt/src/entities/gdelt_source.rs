use chrono::{DateTime, Utc}; use sea_orm::entity::prelude::*; use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "seer_gdelt_sources")]
pub struct Model {
    #[sea_orm(primary_key)] pub id: i64,
    pub created_at: Option<DateTime<Utc>>, pub updated_at: Option<DateTime<Utc>>,
    pub gdelt_worker_id: Option<i64>,
    pub query: String, pub domain: String, pub action_country: String,
    pub start_date: Option<DateTime<Utc>>, pub end_date: Option<DateTime<Utc>>,
    pub min_mentions: i64, pub max_records: i64, pub sort: String,
    #[sea_orm(column_type = "Text")] pub natural_language_filter: String,
    pub is_blacklist: bool,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)] pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
