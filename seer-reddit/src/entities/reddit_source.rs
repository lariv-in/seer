use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "seer_reddit_sources")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub reddit_runner_id: Option<i64>,
    pub subreddits: Json,
    pub search_query: String,
    #[sea_orm(column_type = "Text")]
    pub filter: String,
    pub is_filter_whitelist: bool,
    pub max_fresh_posts: i64,
    pub load_websites: bool,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
