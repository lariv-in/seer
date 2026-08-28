//! Twitter source CRUD + unset multi-select picker.

use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use lariv_rs::{
    components::{ObjectList, SharedChromeFolder, SlotCtx, button_link_route, table_create_button},
    html_form::HtmlFormBody,
    http::Cap,
    picker::respond_picker_select,
    plugins::users::middleware::RequireAuth,
    web::{
        Htmx, ModalFormQuery, html_built_page_or_app_layout, html_built_page_with_slots,
        respond_create_modal_done_fk,
    },
};
use maud::{html, Markup};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use seer_common::{cell, col, row_nav, seer_table_with_actions};
use serde::Deserialize;
use serde_json::json;

use crate::{
    entities::{
        twitter_runner::Entity as RunnerEntity,
        twitter_source::{self, ActiveModel as SourceAM, Entity as SourceEntity},
    },
    forms::SourceForm,
    keys::{
        TwitterSourceCreateModalKey, TwitterSourceUnsetSelectModalKey,
        TwitterSourceUnsetSelectTableKey, TwitterSourcesTableKey,
    },
    routes::{
        TwitterSourceDetailRouteTag, TwitterSourceEditGetRouteTag, TwitterSourcesListRouteTag,
    },
    state::TwitterState,
    templates::{
        TwitterSourceCreatePage, TwitterSourceDetailPage, TwitterSourceEditPage,
        TwitterSourceUnsetSelectPage, TwitterSourcesListPage,
    },
};

use super::{json_string_list, normalize_string_list, source_label};

#[derive(Debug, Deserialize, Default)]
pub struct SourceUnsetSelectQuery {
    #[serde(default, rename = "Name", alias = "name")]
    pub name: Option<String>,
    #[serde(default)]
    pub target_input: Option<String>,
}

fn path_and_query(uri: &Uri) -> String {
    uri.path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string())
}

async fn runner_display(db: &sea_orm::DatabaseConnection, runner_id: Option<i64>) -> String {
    let Some(id) = runner_id.filter(|&i| i > 0) else {
        return String::new();
    };
    lariv_rs::web::opt_or_log(
        RunnerEntity::find_by_id(id).one(db).await,
        "find runner for display",
    )
    .map(|r| r.name)
    .unwrap_or_default()
}

/// Rebuild the create page from raw form input so the user keeps their edits on error.
async fn create_page_from_form(
    db: &sea_orm::DatabaseConnection,
    form: SourceForm,
    q: &ModalFormQuery,
    error: String,
) -> TwitterSourceCreatePage {
    TwitterSourceCreatePage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        twitter_runner_id: form.twitter_runner_id,
        runner_display: runner_display(db, form.twitter_runner_id).await,
        usernames: normalize_string_list(&form.usernames),
        search_query: form.search_query,
        is_filter_whitelist: form.is_filter_whitelist,
        filter: form.filter,
        max_fresh_posts: form.max_fresh_posts,
        load_websites: form.load_websites,
        error,
    }
}

pub async fn list(
    Cap(state): Cap<TwitterState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let sources = SourceEntity::find()
        .order_by_desc(twitter_source::Column::Id)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let headers = [
        col("Usernames", "Usernames"),
        col("Search", "Search"),
        col("Max", "Max fresh"),
        col("Websites", "Websites"),
    ];
    let rows: Vec<_> = sources
        .iter()
        .map(|s| {
            let href = TwitterSourceDetailRouteTag::new(s.id).url();
            let names: Vec<String> =
                serde_json::from_value(s.usernames.clone()).unwrap_or_default();
            row_nav(
                &href,
                vec![
                    cell(&names.join(", ")),
                    cell(&s.search_query),
                    cell(&s.max_fresh_posts.to_string()),
                    cell(if s.load_websites { "Yes" } else { "No" }),
                ],
            )
        })
        .collect();

    let actions = table_create_button::<TwitterSourcesTableKey, TwitterSourceCreateModalKey>(
        Some("plus"),
        "btn-square btn-outline btn-sm",
    );
    let body =
        seer_table_with_actions::<TwitterSourcesTableKey>("Sources", actions, &headers, &rows);
    html_built_page_or_app_layout(
        &TwitterSourcesListPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
}

pub async fn create_get(
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    Query(q): Query<ModalFormQuery>,
) -> Markup {
    let page = TwitterSourceCreatePage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        twitter_runner_id: None,
        runner_display: String::new(),
        usernames: Vec::new(),
        search_query: String::new(),
        is_filter_whitelist: false,
        filter: String::new(),
        max_fresh_posts: 25,
        load_websites: false,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn create_post(
    Cap(state): Cap<TwitterState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<SourceForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let names = normalize_string_list(&form.usernames);
    if names.is_empty() {
        let page = create_page_from_form(
            &state.db,
            form,
            &q,
            "At least one username is required".into(),
        )
        .await;
        return html_built_page_with_slots(&page, &chrome, &slot_ctx).into_response();
    }
    let now = Utc::now();
    let runner_id = form.twitter_runner_id.filter(|&i| i > 0);
    let row = match (SourceAM {
        twitter_runner_id: Set(runner_id),
        usernames: Set(json!(names)),
        search_query: Set(form.search_query.trim().into()),
        filter: Set(form.filter.clone()),
        is_filter_whitelist: Set(form.is_filter_whitelist),
        max_fresh_posts: Set(if form.max_fresh_posts > 0 {
            form.max_fresh_posts
        } else {
            25
        }),
        load_websites: Set(form.load_websites),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    }
    .insert(&state.db)
    .await)
    {
        Ok(r) => r,
        Err(e) => {
            let page = create_page_from_form(&state.db, form, &q, e.to_string()).await;
            return html_built_page_with_slots(&page, &chrome, &slot_ctx).into_response();
        }
    };
    let label = source_label(&row);
    respond_create_modal_done_fk::<TwitterSourceCreateModalKey>(
        &htmx,
        &q.refresh_table(),
        &TwitterSourceDetailRouteTag::new(row.id).url(),
        row.id,
        &label,
        &q.target_input(),
    )
}

pub async fn detail(
    Cap(state): Cap<TwitterState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(s) = lariv_rs::web::opt_or_log(
        SourceEntity::find_by_id(id).one(&state.db).await,
        "find twitter source",
    ) else {
        return Redirect::to(&TwitterSourcesListRouteTag.url()).into_response();
    };
    let worker_name = runner_display(&state.db, s.twitter_runner_id).await;
    let worker_label = if worker_name.is_empty() {
        "—".into()
    } else {
        worker_name
    };
    let filter_mode = if s.is_filter_whitelist {
        "Whitelist"
    } else {
        "Blacklist"
    };
    let body = html! {
        div class="container mx-auto p-4 space-y-4" {
            div class="flex flex-wrap gap-2" {
                (button_link_route(TwitterSourceEditGetRouteTag::new(id), "Edit", "btn-outline"))
                form method="post" action=(format!("/seer-twitter/sources/{id}/delete"))
                    hx-boost="true" hx-confirm="Delete this source?"
                {
                    button class="btn btn-error btn-outline" type="submit" { "Delete" }
                }
            }
            dl class="space-y-2" {
                div { dt class="font-semibold" { "Worker" } dd { (worker_label) } }
                div { dt class="font-semibold" { "Usernames" } dd { (source_label(&s)) } }
                div { dt class="font-semibold" { "Search query" } dd { (s.search_query.clone()) } }
                div { dt class="font-semibold" { "Filter" } dd {
                    (filter_mode) ": " (s.filter.clone())
                } }
                div { dt class="font-semibold" { "Max fresh posts" } dd { (s.max_fresh_posts.to_string()) } }
                div { dt class="font-semibold" { "Load websites" } dd {
                    (if s.load_websites { "Yes" } else { "No" })
                } }
            }
        }
    };
    html_built_page_or_app_layout(
        &TwitterSourceDetailPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
    .into_response()
}

pub async fn edit_get(
    Cap(state): Cap<TwitterState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(s) = lariv_rs::web::opt_or_log(
        SourceEntity::find_by_id(id).one(&state.db).await,
        "find twitter source",
    ) else {
        return Redirect::to(&TwitterSourcesListRouteTag.url()).into_response();
    };
    let page = TwitterSourceEditPage {
        id,
        twitter_runner_id: s.twitter_runner_id,
        runner_display: runner_display(&state.db, s.twitter_runner_id).await,
        usernames: json_string_list(&s.usernames),
        search_query: s.search_query,
        is_filter_whitelist: s.is_filter_whitelist,
        filter: s.filter,
        max_fresh_posts: s.max_fresh_posts,
        load_websites: s.load_websites,
        error: String::new(),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<TwitterState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    HtmlFormBody(form): HtmlFormBody<SourceForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let Some(existing) = lariv_rs::web::opt_or_log(
        SourceEntity::find_by_id(id).one(&state.db).await,
        "find twitter source",
    ) else {
        return Redirect::to(&TwitterSourcesListRouteTag.url()).into_response();
    };
    let names = normalize_string_list(&form.usernames);
    if names.is_empty() {
        let page = TwitterSourceEditPage {
            id,
            twitter_runner_id: form.twitter_runner_id,
            runner_display: runner_display(&state.db, form.twitter_runner_id).await,
            usernames: normalize_string_list(&form.usernames),
            search_query: form.search_query,
            is_filter_whitelist: form.is_filter_whitelist,
            filter: form.filter,
            max_fresh_posts: form.max_fresh_posts,
            load_websites: form.load_websites,
            error: "At least one username is required".into(),
        };
        return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
    }
    let mut am: SourceAM = existing.into();
    am.twitter_runner_id = Set(form.twitter_runner_id.filter(|&i| i > 0));
    am.usernames = Set(json!(names));
    am.search_query = Set(form.search_query.trim().into());
    am.filter = Set(form.filter.clone());
    am.is_filter_whitelist = Set(form.is_filter_whitelist);
    am.max_fresh_posts = Set(if form.max_fresh_posts > 0 {
        form.max_fresh_posts
    } else {
        25
    });
    am.load_websites = Set(form.load_websites);
    am.updated_at = Set(Some(Utc::now()));
    if let Err(e) = am.update(&state.db).await {
        let page = TwitterSourceEditPage {
            id,
            twitter_runner_id: form.twitter_runner_id,
            runner_display: runner_display(&state.db, form.twitter_runner_id).await,
            usernames: normalize_string_list(&form.usernames),
            search_query: form.search_query,
            is_filter_whitelist: form.is_filter_whitelist,
            filter: form.filter,
            max_fresh_posts: form.max_fresh_posts,
            load_websites: form.load_websites,
            error: e.to_string(),
        };
        return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
    }
    htmx.redirect(&TwitterSourceDetailRouteTag::new(id).url())
}

pub async fn delete_post(
    Cap(state): Cap<TwitterState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let _ = SourceEntity::delete_by_id(id).exec(&state.db).await;
    htmx.redirect(&TwitterSourcesListRouteTag.url())
}

pub async fn unset_select(
    Cap(state): Cap<TwitterState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<SourceUnsetSelectQuery>,
) -> Response {
    let sources = SourceEntity::find()
        .filter(twitter_source::Column::TwitterRunnerId.is_null())
        .order_by_desc(twitter_source::Column::Id)
        .limit(50)
        .all(&state.db)
        .await
        .unwrap_or_default();
    let items: Vec<SourceSelectRow> = sources
        .into_iter()
        .map(|s| SourceSelectRow {
            id: s.id,
            label: source_label(&s),
        })
        .collect();
    // Optional name filter against label
    let filter = q.name.clone().unwrap_or_default();
    let filtered: Vec<_> = if filter.trim().is_empty() {
        items
    } else {
        let f = filter.to_lowercase();
        items
            .into_iter()
            .filter(|s| s.label.to_lowercase().contains(&f))
            .collect()
    };
    let total = filtered.len() as u64;
    let list = ObjectList::from_page(filtered, 1, 50, total);
    let page = TwitterSourceUnsetSelectPage {
        sources: list,
        filter_name: filter,
        target_input: q.target_input.unwrap_or_else(|| "source_ids".into()),
        path_and_query: path_and_query(&uri),
    };
    respond_picker_select::<TwitterSourceUnsetSelectTableKey, TwitterSourceUnsetSelectModalKey, _>(
        &htmx, &page,
    )
    .into_response()
}

pub struct SourceSelectRow {
    pub id: i64,
    pub label: String,
}
