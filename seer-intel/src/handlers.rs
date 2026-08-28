//! HTTP handlers for intel list/detail and preferences.

pub mod map;
pub mod preferences;

use axum::extract::Path;
use maud::{html, Markup};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{html_built_page_or_app_layout, Htmx},
};
use seer_common::{cell, col, row_nav, seer_table};

use crate::{
    entities::{
        intel::{self, Entity as IntelEntity},
        intel_event::{self, Entity as EventEntity},
    },
    keys::IntelTableKey,
    kind::load_intel_kind,
    routes::IntelDetailRouteTag,
    state::IntelState,
    templates::{IntelDetailPage, IntelListPage},
};

struct IntelListRow {
    id: String,
    href: String,
    title: String,
    kind: String,
    when: String,
}

pub async fn list(
    Cap(state): Cap<IntelState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let models = IntelEntity::find()
        .order_by_desc(intel::Column::Datetime)
        .limit(100)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let data: Vec<IntelListRow> = models
        .into_iter()
        .map(|r| IntelListRow {
            id: r.id.to_string(),
            href: IntelDetailRouteTag::new(r.id).url(),
            title: r.title,
            kind: format!("{}#{}", r.kind, r.kind_id),
            when: ctx.format_datetime(r.datetime).into_string(),
        })
        .collect();

    let headers = [
        col("Id", "ID"),
        col("Title", "Title"),
        col("Kind", "Kind"),
        col("When", "When"),
    ];
    let rows: Vec<_> = data
        .iter()
        .map(|r| {
            row_nav(
                &r.href,
                vec![cell(&r.id), cell(&r.title), cell(&r.kind), cell(&r.when)],
            )
        })
        .collect();

    let body = seer_table::<IntelTableKey>("Intel", &headers, &rows);
    html_built_page_or_app_layout(
        &IntelListPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}

pub async fn detail(
    Cap(state): Cap<IntelState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Markup {
    let row = lariv_rs::web::opt_or_log(
        IntelEntity::find_by_id(id).one(&state.db).await,
        "find intel by id",
    );
    let event = lariv_rs::web::opt_or_log(
        EventEntity::find()
            .filter(intel_event::Column::IntelId.eq(id))
            .one(&state.db)
            .await,
        "find intel event",
    );

    let body = if let Some(r) = row {
        let source_href = match load_intel_kind(&state.db, &r.kind, r.kind_id).await {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::error!(error = %e, "load intel kind");
                None
            }
        };
        let detail_path = if let Some(k) = source_href {
            k.intel_detail().await.unwrap_or_default()
        } else {
            String::new()
        };
        html! {
            div class="container mx-auto p-4 space-y-4" {
                h1 class="text-2xl font-semibold" { (r.title.clone()) }
                p class="opacity-70" { (format!("{} · {} · {}", r.kind, r.kind_id, ctx.format_datetime(r.datetime).into_string())) }
                @if !detail_path.is_empty() {
                    p { a href=(detail_path) class="link" { "Open source" } }
                }
                pre class="whitespace-pre-wrap text-sm bg-base-200 p-4 rounded" { (r.summary.clone()) }
                @if let Some(ev) = event {
                    div {
                        h2 class="text-lg font-medium" { "Event" }
                        p { (ev.address.clone()) }
                        @if let (Some(lat), Some(lng)) = (ev.latitude, ev.longitude) {
                            p { (format!("{lat}, {lng}")) }
                        }
                    }
                }
            }
        }
    } else {
        html! { div class="p-4" { "Not found" } }
    };

    html_built_page_or_app_layout(
        &IntelDetailPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}
