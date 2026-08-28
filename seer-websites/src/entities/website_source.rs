use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "seer_website_sources")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub website_runner_id: Option<i64>,
    #[sea_orm(column_type = "Text")]
    pub url: String,
    pub depth: i64,
    #[sea_orm(column_type = "Text")]
    pub filter: String,
    pub is_filter_whitelist: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::website_runner::Entity",
        from = "Column::WebsiteRunnerId",
        to = "super::website_runner::Column::Id"
    )]
    Runner,
}

impl Related<super::website_runner::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Runner.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
