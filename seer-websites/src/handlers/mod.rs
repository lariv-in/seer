//! Websites HTTP handlers.

pub mod sources;
pub mod workers;

use axum::extract::Path;
use maud::{html, Markup};
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};

use lariv_rs::{
    components::{field_markdown, FieldMarkdown, SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{html_built_page_or_app_layout, Htmx},
};
use seer_common::{cell, col, row_nav, seer_table};

use crate::{
    entities::website::{self, Entity as WebsiteEntity},
    keys::WebsitesTableKey,
    routes::WebsiteDetailRouteTag,
    state::WebsitesState,
    templates::{WebsiteDetailPage, WebsitesListPage},
};

/// Human label for a source row, used by pickers and detail views.
pub(crate) fn source_label(s: &crate::entities::website_source::Model) -> String {
    let url = s.url.trim();
    if url.is_empty() {
        format!("#{}", s.id)
    } else {
        url.to_string()
    }
}

struct SiteRow {
    id: String,
    href: String,
    url: String,
}

pub async fn list(
    Cap(state): Cap<WebsitesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let sites = WebsiteEntity::find()
        .order_by_desc(website::Column::Id)
        .limit(100)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let data: Vec<SiteRow> = sites
        .into_iter()
        .map(|s| SiteRow {
            id: s.id.to_string(),
            href: WebsiteDetailRouteTag::new(s.id).url(),
            url: s.url,
        })
        .collect();

    let headers = [col("Id", "ID"), col("Url", "URL")];
    let rows: Vec<_> = data
        .iter()
        .map(|r| row_nav(&r.href, vec![cell(&r.id), cell(&r.url)]))
        .collect();

    let body = seer_table::<WebsitesTableKey>("Scraped pages", &headers, &rows);
    html_built_page_or_app_layout(
        &WebsitesListPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}

pub async fn detail(
    Cap(state): Cap<WebsitesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Markup {
    let row = lariv_rs::web::opt_or_log(
        WebsiteEntity::find_by_id(id).one(&state.db).await,
        "find website by id",
    );
    let body = if let Some(s) = row {
        html! {
            div class="container mx-auto p-4 space-y-2" {
                h1 class="text-xl font-semibold" { (s.url.clone()) }
                (field_markdown(FieldMarkdown {
                    value: &s.markdown,
                    classes: "",
                }))
            }
        }
    } else {
        html! { div class="p-4" { "Not found" } }
    };
    html_built_page_or_app_layout(
        &WebsiteDetailPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}
