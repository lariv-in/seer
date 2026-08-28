use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "seer_intels")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub summary: String,
    pub datetime: DateTime<Utc>,
    /// Stored as pgvector `vector`; SeaORM binds text so we cast on read/write.
    #[sea_orm(column_type = "Text", nullable, select_as = "text", save_as = "vector")]
    pub embedding: Option<String>,
    pub kind: String,
    pub kind_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::intel_event::Entity")]
    Event,
}

impl Related<super::intel_event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
