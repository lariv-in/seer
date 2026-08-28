use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use std::time::Duration;
use tracing::{info, warn};

use crate::{
    client,
    entities::opensky_state::{self, ActiveModel as AM, Entity},
    preferences::resolved_config,
};

pub fn start_poller(db: DatabaseConnection) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let mut ticker = tokio::time::interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;
            if let Err(e) = poll_once(&db).await {
                warn!("opensky poll: {e:#}");
            }
        }
    });
}

async fn poll_once(db: &DatabaseConnection) -> anyhow::Result<()> {
    let cfg = resolved_config(db).await?;
    if cfg.client_id.is_empty() {
        return Ok(());
    }
    let (snap, states) = client::fetch_states(&cfg).await?;
    info!("opensky: {} states", states.len());
    for s in states.into_iter().take(500) {
        let existing = Entity::find()
            .filter(opensky_state::Column::Icao24.eq(&s.icao24))
            .filter(opensky_state::Column::LastContact.eq(s.last_contact))
            .one(db)
            .await?;
        if existing.is_some() {
            continue;
        }
        let now = Utc::now();
        let icao24 = s.icao24.clone();
        if let Err(e) = (AM {
            snapshot_time: Set(snap),
            icao24: Set(s.icao24),
            last_contact: Set(s.last_contact),
            callsign: Set(s.callsign),
            origin_country: Set(s.origin_country),
            time_position: Set(s.time_position),
            latitude: Set(s.latitude),
            longitude: Set(s.longitude),
            baro_altitude: Set(s.baro_altitude),
            on_ground: Set(s.on_ground),
            velocity: Set(s.velocity),
            true_track: Set(s.true_track),
            vertical_rate: Set(s.vertical_rate),
            geo_altitude: Set(s.geo_altitude),
            squawk: Set(s.squawk),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        })
        .insert(db)
        .await
        {
            warn!(error = %e, icao24 = %icao24, "failed to insert opensky state");
        }
    }
    Ok(())
}
