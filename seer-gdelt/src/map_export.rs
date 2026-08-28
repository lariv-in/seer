use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;
use sea_orm::DatabaseConnection;
use crate::entities::event::{self, Entity as EventEntity};

#[derive(Serialize)]
pub struct MapEvent { pub id: i64, pub lat: f64, pub lng: f64, pub source_url: String }

pub async fn export_map_events(db: &DatabaseConnection) -> Vec<MapEvent> {
    EventEntity::find()
        .filter(event::Column::ActionGeoLat.is_not_null())
        .filter(event::Column::ActionGeoLong.is_not_null())
        .all(db).await.unwrap_or_default()
        .into_iter().filter_map(|e| Some(MapEvent {
            id: e.id, lat: e.action_geo_lat?, lng: e.action_geo_long?, source_url: e.source_url,
        })).collect()
}
