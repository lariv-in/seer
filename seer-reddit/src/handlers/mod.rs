//! Reddit HTTP handlers.

pub mod sources;
pub mod workers;

use axum::extract::Path;
use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{html_built_page_or_app_layout, Htmx},
};
use maud::{html, Markup};
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};
use seer_common::{cell, col, row_nav, seer_table};

use crate::{
    entities::reddit_post::{self, Entity as PostEntity},
    keys::RedditTableKey,
    routes::RedditPostDetailRouteTag,
    state::RedditState,
    templates::{RedditListPage, RedditPostDetailPage},
};

pub(crate) fn source_label(s: &crate::entities::reddit_source::Model) -> String {
    let subs: Vec<String> = serde_json::from_value(s.subreddits.clone()).unwrap_or_default();
    let label = if subs.is_empty() {
        format!("#{}", s.id)
    } else {
        subs.join(",")
    };
    if s.search_query.trim().is_empty() {
        label
    } else {
        format!("{label} ({})", s.search_query.trim())
    }
}

pub(crate) fn normalize_subreddit(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    loop {
        let trimmed = s.trim();
        let lower = trimmed.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("/r/") {
            s = trimmed[trimmed.len() - rest.len()..].trim_start().to_string();
            continue;
        }
        if let Some(rest) = lower.strip_prefix("r/") {
            s = trimmed[trimmed.len() - rest.len()..].trim_start().to_string();
            continue;
        }
        return trimmed.to_string();
    }
}

pub(crate) fn normalize_string_list(items: &[String]) -> Vec<String> {
    items
        .iter()
        .map(|s| normalize_subreddit(s))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Public helper for tools / other crates that write Reddit sources.
pub fn normalize_subreddit_names(items: &[String]) -> Vec<String> {
    normalize_string_list(items)
}

pub(crate) fn json_string_list(value: &serde_json::Value) -> Vec<String> {
    serde_json::from_value(value.clone()).unwrap_or_default()
}

#[cfg(test)]
mod normalize_tests {
    use super::{normalize_string_list, normalize_subreddit};

    #[test]
    fn strips_prefixes_and_whitespace() {
        assert_eq!(normalize_subreddit("  r/rust  "), "rust");
        assert_eq!(normalize_subreddit("/r/rust"), "rust");
        assert_eq!(normalize_subreddit("R/AskReddit"), "AskReddit");
        assert_eq!(normalize_subreddit("r/ r/foo"), "foo");
        assert_eq!(normalize_subreddit("rust"), "rust");
        assert_eq!(
            normalize_string_list(&[
                "  r/a ".into(),
                "".into(),
                "/r/b".into(),
                "   ".into()
            ]),
            vec!["a".to_string(), "b".to_string()]
        );
    }
}

struct PostRow {
    id: String,
    href: String,
    sub: String,
    title: String,
}

pub async fn list(
    Cap(state): Cap<RedditState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let posts = PostEntity::find()
        .order_by_desc(reddit_post::Column::Id)
        .limit(100)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let data: Vec<PostRow> = posts
        .into_iter()
        .map(|p| PostRow {
            id: p.id.to_string(),
            href: RedditPostDetailRouteTag::new(p.id).url(),
            sub: p.subreddit,
            title: p.title,
        })
        .collect();

    let headers = [col("Id", "ID"), col("Sub", "Sub"), col("Title", "Title")];
    let rows: Vec<_> = data
        .iter()
        .map(|r| row_nav(&r.href, vec![cell(&r.id), cell(&r.sub), cell(&r.title)]))
        .collect();

    let body = seer_table::<RedditTableKey>("Reddit", &headers, &rows);
    html_built_page_or_app_layout(
        &RedditListPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}

pub async fn post_detail(
    Cap(state): Cap<RedditState>,
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
        html! { div class="container mx-auto p-4 space-y-2" {
            h1 class="text-xl font-semibold" { (p.title.clone()) }
            p class="opacity-70" { (format!("r/{} · {}", p.subreddit, p.author)) }
            pre class="whitespace-pre-wrap text-sm bg-base-200 p-4 rounded" { (p.selftext.clone()) }
            a href=(p.permalink.clone()) class="link" { "Permalink" }
        }}
    } else {
        html! { div class="p-4"{"Not found"} }
    };
    html_built_page_or_app_layout(
        &RedditPostDetailPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}
