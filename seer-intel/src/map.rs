use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use sea_orm::DatabaseConnection;

use crate::entities::intel_event::{self, Entity as EventEntity};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MapBounds {
    #[serde(default)] pub north: f64,
    #[serde(default)] pub south: f64,
    #[serde(default)] pub east: f64,
    #[serde(default)] pub west: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MapPoint {
    pub id: i64,
    pub lat: f64,
    pub lng: f64,
    pub address: String,
    pub intel_id: i64,
}

pub async fn points_in_bounds(db: &DatabaseConnection, bounds: &MapBounds) -> Vec<MapPoint> {
    let rows = EventEntity::find()
        .filter(intel_event::Column::Latitude.is_not_null())
        .filter(intel_event::Column::Longitude.is_not_null())
        .all(db)
        .await
        .unwrap_or_default();
    rows.into_iter().filter_map(|e| {
        let lat = e.latitude?;
        let lng = e.longitude?;
        if lat < bounds.south || lat > bounds.north { return None; }
        // naive lon filter (no antimeridian)
        if bounds.west <= bounds.east {
            if lng < bounds.west || lng > bounds.east { return None; }
        }
        Some(MapPoint { id: e.id, lat, lng, address: e.address, intel_id: e.intel_id })
    }).collect()
}

pub const LIVE_MAP_JS: &str = include_str!("assets/live_map.js");
