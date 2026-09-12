//! HTTP handlers for OpenSky list and preferences.

pub mod preferences;

use maud::Markup;
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};
use seer_common::{cell, col, row, seer_table};

use crate::{
    entities::opensky_state::{self, Entity},
    keys::OpenskyTableKey,
    state::OpenskyState,
    templates::OpenskyListPage,
};

struct StateRow {
    icao: String,
    callsign: String,
    lat: String,
    lng: String,
}

pub async fn list(
    Cap(state): Cap<OpenskyState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let models = Entity::find()
        .order_by_desc(opensky_state::Column::Id)
        .limit(100)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let data: Vec<StateRow> = models
        .into_iter()
        .map(|r| StateRow {
            icao: r.icao24,
            callsign: r.callsign.unwrap_or_default(),
            lat: r.latitude.map(|v| v.to_string()).unwrap_or_default(),
            lng: r.longitude.map(|v| v.to_string()).unwrap_or_default(),
        })
        .collect();

    let headers = [
        col("Icao", "ICAO"),
        col("Callsign", "Callsign"),
        col("Lat", "Lat"),
        col("Lng", "Lng"),
    ];
    let rows: Vec<_> = data
        .iter()
        .map(|r| {
            row(vec![
                cell(&r.icao),
                cell(&r.callsign),
                cell(&r.lat),
                cell(&r.lng),
            ])
        })
        .collect();

    let body = seer_table::<OpenskyTableKey>("OpenSky", &headers, &rows);
    html_built_page_or_app_layout(
        &OpenskyListPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}
