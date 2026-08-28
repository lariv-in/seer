//! HTTP handlers for GDELT events, workers, sources and preferences.

pub mod preferences;
pub mod sources;
pub mod workers;

use chrono::{DateTime, Utc};
use maud::Markup;
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    datetime::{format_date_in_tz, parse_date_start_in_tz},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};
use seer_common::{cell, cell_link, col, row, seer_table_with_subtitle};

use crate::{
    entities::event::{self, Entity as EventEntity},
    keys::GdeltTableKey,
    map_export,
    state::GdeltState,
    templates::GdeltListPage,
};

/// Human label for a source, used in pickers and detail pages.
pub(crate) fn source_label(s: &crate::entities::gdelt_source::Model) -> String {
    let query = s.query.trim();
    let domain = s.domain.trim();
    let label = if !query.is_empty() {
        query.to_string()
    } else if !domain.is_empty() {
        domain.to_string()
    } else {
        format!("#{}", s.id)
    };
    if domain.is_empty() || query.is_empty() {
        label
    } else {
        format!("{label} ({domain})")
    }
}

/// Parse an optional day-first date (`DD/MM/YYYY`). Empty input yields `None`.
pub(crate) fn parse_opt_date(raw: &str, tz: &str) -> Result<Option<DateTime<Utc>>, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    parse_date_start_in_tz(raw, tz)
        .map(Some)
        .ok_or_else(|| format!("Invalid date '{raw}': use DD/MM/YYYY"))
}

/// Format an optional stored instant as a calendar date in `tz`.
pub(crate) fn format_opt_date(value: Option<DateTime<Utc>>, tz: &str) -> String {
    value
        .map(|d| format_date_in_tz(d, tz))
        .unwrap_or_default()
}

struct EventRow {
    id: String,
    actors: String,
    url: String,
}

pub async fn list(
    Cap(state): Cap<GdeltState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let events = EventEntity::find()
        .order_by_desc(event::Column::Id)
        .limit(100)
        .all(&state.db)
        .await
        .unwrap_or_default();
    let map_pts = map_export::export_map_events(&state.db).await.len();
    let subtitle = format!("Map-exportable points: {map_pts}");

    let data: Vec<EventRow> = events
        .into_iter()
        .map(|e| EventRow {
            id: e.id.to_string(),
            actors: format!("{} / {}", e.actor1_name, e.actor2_name),
            url: e.source_url,
        })
        .collect();

    let headers = [col("Id", "ID"), col("Actors", "Actors"), col("Url", "URL")];
    let rows: Vec<_> = data
        .iter()
        .map(|r| {
            row(vec![
                cell(&r.id),
                cell(&r.actors),
                cell_link(&r.url, "source"),
            ])
        })
        .collect();

    let body = seer_table_with_subtitle::<GdeltTableKey>(
        "GDELT",
        &subtitle,
        &headers,
        &rows,
    );
    html_built_page_or_app_layout(
        &GdeltListPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}
