use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "seer_reddit_posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub reddit_runner_id: Option<i64>,
    #[sea_orm(unique)]
    pub post_id: String,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub selftext: String,
    pub author: String,
    pub subreddit: String,
    pub permalink: String,
    pub url: String,
    pub created_utc_unix: f64,
    pub score: i64,
    pub num_comments: i64,
    pub is_self: bool,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
