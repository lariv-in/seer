//! HTTP handlers for twitter list/detail, preferences, workers and sources.

pub mod preferences;
pub mod sources;
pub mod workers;

use axum::extract::Path;
use maud::{html, Markup};
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{html_built_page_or_app_layout, Htmx},
};
use seer_common::{cell, col, row_nav, seer_table};

use crate::{
    entities::twitter_post::{self, Entity as PostEntity},
    keys::TwitterTableKey,
    routes::TwitterPostDetailRouteTag,
    state::TwitterState,
    templates::{TwitterListPage, TwitterPostDetailPage},
};

pub(crate) fn source_label(s: &crate::entities::twitter_source::Model) -> String {
    let names: Vec<String> = serde_json::from_value(s.usernames.clone()).unwrap_or_default();
    let label = if names.is_empty() {
        format!("#{}", s.id)
    } else {
        names.join(",")
    };
    if s.search_query.trim().is_empty() {
        label
    } else {
        format!("{label} ({})", s.search_query.trim())
    }
}

pub(crate) fn normalize_username(raw: &str) -> String {
    raw.trim().trim_start_matches('@').trim().to_string()
}

pub(crate) fn normalize_string_list(items: &[String]) -> Vec<String> {
    items
        .iter()
        .map(|s| normalize_username(s))
        .filter(|s| !s.is_empty())
        .collect()
}

pub(crate) fn json_string_list(value: &serde_json::Value) -> Vec<String> {
    serde_json::from_value(value.clone()).unwrap_or_default()
}

struct PostRow {
    id: String,
    href: String,
    author: String,
    title: String,
}

pub async fn list(
    Cap(state): Cap<TwitterState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let posts = PostEntity::find()
        .order_by_desc(twitter_post::Column::Id)
        .limit(100)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let data: Vec<PostRow> = posts
        .into_iter()
        .map(|p| PostRow {
            id: p.id.to_string(),
            href: TwitterPostDetailRouteTag::new(p.id).url(),
            author: p.author,
            title: p.title,
        })
        .collect();

    let headers = [
        col("Id", "ID"),
        col("Author", "Author"),
        col("Title", "Title"),
    ];
    let rows: Vec<_> = data
        .iter()
        .map(|r| {
            row_nav(
                &r.href,
                vec![cell(&r.id), cell(&r.author), cell(&r.title)],
            )
        })
        .collect();

    let body = seer_table::<TwitterTableKey>("Twitter / Nitter", &headers, &rows);
    html_built_page_or_app_layout(
        &TwitterListPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}

pub async fn post_detail(
    Cap(state): Cap<TwitterState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Markup {
    let row = lariv_rs::web::opt_or_log(
        PostEntity::find_by_id(id).one(&state.db).await,
        "find post by id",
    );
    let body = if let Some(p) = row {
        html! {
            div class="container mx-auto p-4 space-y-2" {
                h1 class="text-xl font-semibold" { (p.title.clone()) }
                p { (format!("@{}", p.author)) }
                pre class="whitespace-pre-wrap text-sm bg-base-200 p-4 rounded" { (p.selftext.clone()) }
            }
        }
    } else {
        html! { div class="p-4" { "Not found" } }
    };
    html_built_page_or_app_layout(
        &TwitterPostDetailPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}
