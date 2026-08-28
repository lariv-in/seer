//! HTTP handlers for AIS stream list and preferences.

pub mod preferences;

use axum::extract::Path;
use maud::{html, Markup, PreEscaped};
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};
use seer_common::{cell, col, row_nav, seer_table_with_subtitle};

use crate::{
    entities::position_report::{self, Entity},
    keys::AisTableKey,
    map::{map_style_url, DETAIL_MAP_JS},
    preferences::resolved_config,
    routes::AisDetailRouteTag,
    state::AisstreamState,
    templates::{AisDetailPage, AisListPage},
};

struct AisRow {
    id: String,
    href: String,
    mmsi: String,
    name: String,
    lat: String,
    lng: String,
}

pub async fn list(
    Cap(state): Cap<AisstreamState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let models = Entity::find()
        .order_by_desc(position_report::Column::Id)
        .limit(100)
        .all(&state.db)
        .await
        .unwrap_or_default();
    let enabled = resolved_config(&state.db)
        .await
        .map(|c| c.enabled)
        .unwrap_or(false);
    let subtitle = if enabled {
        "Client enabled"
    } else {
        "Client disabled"
    };

    let data: Vec<AisRow> = models
        .into_iter()
        .map(|r| AisRow {
            id: r.id.to_string(),
            href: AisDetailRouteTag::new(r.id).url(),
            mmsi: r.mmsi.to_string(),
            name: r.ship_name,
            lat: r.latitude.map(|v| v.to_string()).unwrap_or_default(),
            lng: r.longitude.map(|v| v.to_string()).unwrap_or_default(),
        })
        .collect();

    let headers = [
        col("Id", "ID"),
        col("Mmsi", "MMSI"),
        col("Name", "Name"),
        col("Lat", "Lat"),
        col("Lng", "Lng"),
    ];
    let rows: Vec<_> = data
        .iter()
        .map(|r| {
            row_nav(
                &r.href,
                vec![
                    cell(&r.id),
                    cell(&r.mmsi),
                    cell(&r.name),
                    cell(&r.lat),
                    cell(&r.lng),
                ],
            )
        })
        .collect();

    let body = seer_table_with_subtitle::<AisTableKey>(
        "AIS Stream",
        subtitle,
        &headers,
        &rows,
    );
    html_built_page_or_app_layout(
        &AisListPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}

pub async fn detail(
    Cap(state): Cap<AisstreamState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Markup {
    let row = lariv_rs::web::opt_or_log(
        Entity::find_by_id(id).one(&state.db).await,
        "find ais position report by id",
    );
    let body = if let Some(r) = row {
        let style = map_style_url();
        html! {
            div class="container mx-auto p-4 space-y-4" {
                h1 class="text-2xl font-semibold" { (r.ship_name.clone()) }
                dl class="grid grid-cols-2 gap-x-4 gap-y-2 max-w-lg text-sm" {
                    dt { "MMSI" }
                    dd { (r.mmsi.to_string()) }
                    @if let Some(lat) = r.latitude {
                        dt { "Latitude" }
                        dd { (format!("{lat}")) }
                    }
                    @if let Some(lng) = r.longitude {
                        dt { "Longitude" }
                        dd { (format!("{lng}")) }
                    }
                    @if let Some(sog) = r.sog {
                        dt { "Speed (SOG)" }
                        dd { (format!("{sog} kn")) }
                    }
                    @if let Some(cog) = r.cog {
                        dt { "Course (COG)" }
                        dd { (format!("{cog}°")) }
                    }
                    @if let Some(heading) = r.heading {
                        dt { "Heading" }
                        dd { (format!("{heading}°")) }
                    }
                }
                @if let (Some(lat), Some(lng)) = (r.latitude, r.longitude) {
                    div
                        data-seer-ais-detail-map
                        data-lat=(lat.to_string())
                        data-lng=(lng.to_string())
                        data-label=(r.ship_name.as_str())
                        data-map-style=(style.as_str())
                        class="space-y-2"
                    {
                        div
                            data-seer-map-canvas
                            class="bg-base-200 rounded w-full h-[50vh] min-h-64 overflow-hidden"
                        {}
                    }
                    (PreEscaped(format!("<script>\n{DETAIL_MAP_JS}\n</script>")))
                } @else {
                    p class="opacity-70" { "No position data available." }
                }
            }
        }
    } else {
        html! { div class="p-4" { "Not found" } }
    };
    html_built_page_or_app_layout(
        &AisDetailPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}
