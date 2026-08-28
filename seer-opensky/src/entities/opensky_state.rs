use chrono::{DateTime, Utc}; use sea_orm::entity::prelude::*; use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "seer_opensky_states")]
pub struct Model {
    #[sea_orm(primary_key)] pub id: i64,
    pub created_at: Option<DateTime<Utc>>, pub updated_at: Option<DateTime<Utc>>,
    pub snapshot_time: i64,
    pub icao24: String, pub last_contact: i64,
    pub callsign: Option<String>, pub origin_country: Option<String>,
    pub time_position: Option<i64>,
    pub latitude: Option<f64>, pub longitude: Option<f64>,
    pub baro_altitude: Option<f64>, pub on_ground: Option<bool>,
    pub velocity: Option<f64>, pub true_track: Option<f64>, pub vertical_rate: Option<f64>,
    pub geo_altitude: Option<f64>, pub squawk: Option<String>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)] pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
