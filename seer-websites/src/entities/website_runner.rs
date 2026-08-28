use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "seer_website_runners")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[sea_orm(unique)]
    pub name: String,
    /// Cadence in seconds.
    pub duration_secs: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::website_source::Entity")]
    Sources,
}

impl Related<super::website_source::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sources.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
