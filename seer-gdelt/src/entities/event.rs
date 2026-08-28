use chrono::{DateTime, Utc}; use sea_orm::entity::prelude::*; use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "seer_gdelt_events")]
pub struct Model {
    #[sea_orm(primary_key)] pub id: i64,
    pub created_at: Option<DateTime<Utc>>, pub updated_at: Option<DateTime<Utc>>,
    pub gdelt_source_id: Option<i64>,
    pub global_event_id: String,
    #[sea_orm(column_type = "Text")] pub source_url: String,
    #[sea_orm(column_type = "Text")] pub actor1_name: String,
    #[sea_orm(column_type = "Text")] pub actor2_name: String,
    pub action_geo_lat: Option<f64>, pub action_geo_long: Option<f64>,
    pub event_date: Option<DateTime<Utc>>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)] pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
