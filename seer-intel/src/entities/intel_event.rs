use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "seer_intel_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub intel_id: i64,
    #[sea_orm(column_type = "Text")]
    pub address: String,
    pub datetime: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::intel::Entity",
        from = "Column::IntelId",
        to = "super::intel::Column::Id"
    )]
    Intel,
}

impl Related<super::intel::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Intel.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
