use chrono::{DateTime, Utc}; use sea_orm::entity::prelude::*; use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "seer_aisstream_position_reports")]
pub struct Model {
    #[sea_orm(primary_key)] pub id: i64,
    pub created_at: Option<DateTime<Utc>>, pub updated_at: Option<DateTime<Utc>>,
    pub mmsi: i64, pub latitude: Option<f64>, pub longitude: Option<f64>,
    pub sog: Option<f64>, pub cog: Option<f64>, pub heading: Option<f64>,
    pub ship_name: String,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)] pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
