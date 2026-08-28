//! Website source CRUD + unset multi-select picker.

use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use lariv_rs::{
    components::{button_link_route, table_create_button, ObjectList, SharedChromeFolder, SlotCtx},
    html_form::HtmlFormBody,
    http::Cap,
    picker::respond_picker_select,
    plugins::users::middleware::RequireAuth,
    web::{
        html_built_page_or_app_layout, html_built_page_with_slots, respond_create_modal_done_fk,
        Htmx, ModalFormQuery,
    },
};
use maud::{html, Markup};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use seer_common::{cell, col, row_nav, seer_table_with_actions};
use serde::Deserialize;

use crate::{
    entities::{
        website_runner::Entity as RunnerEntity,
        website_source::{self, ActiveModel as SourceAM, Entity as SourceEntity},
    },
    forms::SourceForm,
    keys::{
        WebsiteSourceCreateModalKey, WebsiteSourceUnsetSelectModalKey,
        WebsiteSourceUnsetSelectTableKey, WebsiteSourcesTableKey,
    },
    routes::{
        WebsiteSourceDeletePostRouteTag, WebsiteSourceDetailRouteTag, WebsiteSourceEditGetRouteTag,
        WebsiteSourcesListRouteTag,
    },
    ssrf::normalize_website_url,
    state::WebsitesState,
    templates::{
        WebsiteSourceCreatePage, WebsiteSourceDetailPage, WebsiteSourceEditPage,
        WebsiteSourceUnsetSelectPage, WebsiteSourcesListPage,
    },
};

use super::source_label;

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

fn normalize_depth(depth: i64) -> i64 {
    depth.max(0)
}

/// Rebuild the create page from raw form input so the user keeps their edits on error.
async fn create_page_from_form(
    db: &sea_orm::DatabaseConnection,
    form: SourceForm,
    q: &ModalFormQuery,
    error: String,
) -> WebsiteSourceCreatePage {
    WebsiteSourceCreatePage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        website_runner_id: form.website_runner_id,
        runner_display: runner_display(db, form.website_runner_id).await,
        url: form.url,
        depth: form.depth,
        filter: form.filter,
        is_filter_whitelist: form.is_filter_whitelist,
        error,
    }
}

pub async fn list(
    Cap(state): Cap<WebsitesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let sources = SourceEntity::find()
        .order_by_desc(website_source::Column::Id)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let headers = [col("Url", "URL"), col("Depth", "Depth")];
    let rows: Vec<_> = sources
        .iter()
        .map(|s| {
            let href = WebsiteSourceDetailRouteTag::new(s.id).url();
            row_nav(&href, vec![cell(&s.url), cell(&s.depth.to_string())])
        })
        .collect();

    let actions = table_create_button::<WebsiteSourcesTableKey, WebsiteSourceCreateModalKey>(
        Some("plus"),
        "btn-square btn-outline btn-sm",
    );
    let body =
        seer_table_with_actions::<WebsiteSourcesTableKey>("Sources", actions, &headers, &rows);
    html_built_page_or_app_layout(
        &WebsiteSourcesListPage { body },
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
    let page = WebsiteSourceCreatePage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        website_runner_id: None,
        runner_display: String::new(),
        url: String::new(),
        depth: 1,
        filter: String::new(),
        is_filter_whitelist: false,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn create_post(
    Cap(state): Cap<WebsitesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<SourceForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let url = match normalize_website_url(&form.url) {
        Ok(u) => u.to_string(),
        Err(e) => {
            let page = create_page_from_form(&state.db, form, &q, e.to_string()).await;
            return html_built_page_with_slots(&page, &chrome, &slot_ctx).into_response();
        }
    };
    let now = Utc::now();
    let row = match (SourceAM {
        website_runner_id: Set(form.website_runner_id.filter(|&i| i > 0)),
        url: Set(url),
        depth: Set(normalize_depth(form.depth)),
        filter: Set(form.filter.clone()),
        is_filter_whitelist: Set(form.is_filter_whitelist),
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
    respond_create_modal_done_fk::<WebsiteSourceCreateModalKey>(
        &htmx,
        &q.refresh_table(),
        &WebsiteSourceDetailRouteTag::new(row.id).url(),
        row.id,
        &label,
        &q.target_input(),
    )
}

pub async fn detail(
    Cap(state): Cap<WebsitesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(s) = lariv_rs::web::opt_or_log(
        SourceEntity::find_by_id(id).one(&state.db).await,
        "find website source",
    ) else {
        return Redirect::to(&WebsiteSourcesListRouteTag.url()).into_response();
    };
    let worker_name = runner_display(&state.db, s.website_runner_id).await;
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
    let delete_path = WebsiteSourceDeletePostRouteTag::new(id).path();
    let body = html! {
        div class="container mx-auto p-4 space-y-4" {
            div class="flex flex-wrap gap-2" {
                (button_link_route(WebsiteSourceEditGetRouteTag::new(id), "Edit", "btn-outline"))
                form method="post" action=(delete_path)
                    hx-boost="true" hx-confirm="Delete this source?"
                {
                    button class="btn btn-error btn-outline" type="submit" { "Delete" }
                }
            }
            dl class="space-y-2" {
                div { dt class="font-semibold" { "Worker" } dd { (worker_label) } }
                div { dt class="font-semibold" { "URL" } dd { (s.url.clone()) } }
                div { dt class="font-semibold" { "Depth" } dd { (s.depth.to_string()) } }
                div { dt class="font-semibold" { "Filter" } dd {
                    (filter_mode) ": " (s.filter.clone())
                } }
            }
        }
    };
    html_built_page_or_app_layout(
        &WebsiteSourceDetailPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
    .into_response()
}

pub async fn edit_get(
    Cap(state): Cap<WebsitesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(s) = lariv_rs::web::opt_or_log(
        SourceEntity::find_by_id(id).one(&state.db).await,
        "find website source",
    ) else {
        return Redirect::to(&WebsiteSourcesListRouteTag.url()).into_response();
    };
    let page = WebsiteSourceEditPage {
        id,
        website_runner_id: s.website_runner_id,
        runner_display: runner_display(&state.db, s.website_runner_id).await,
        url: s.url,
        depth: s.depth,
        filter: s.filter,
        is_filter_whitelist: s.is_filter_whitelist,
        error: String::new(),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<WebsitesState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    HtmlFormBody(form): HtmlFormBody<SourceForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let Some(existing) = lariv_rs::web::opt_or_log(
        SourceEntity::find_by_id(id).one(&state.db).await,
        "find website source",
    ) else {
        return Redirect::to(&WebsiteSourcesListRouteTag.url()).into_response();
    };
    let url = match normalize_website_url(&form.url) {
        Ok(u) => u.to_string(),
        Err(e) => {
            let page = WebsiteSourceEditPage {
                id,
                website_runner_id: form.website_runner_id,
                runner_display: runner_display(&state.db, form.website_runner_id).await,
                url: form.url,
                depth: form.depth,
                filter: form.filter.clone(),
                is_filter_whitelist: form.is_filter_whitelist,
                error: e.to_string(),
            };
            return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
        }
    };
    let mut am: SourceAM = existing.into();
    am.website_runner_id = Set(form.website_runner_id.filter(|&i| i > 0));
    am.url = Set(url);
    am.depth = Set(normalize_depth(form.depth));
    am.filter = Set(form.filter.clone());
    am.is_filter_whitelist = Set(form.is_filter_whitelist);
    am.updated_at = Set(Some(Utc::now()));
    if let Err(e) = am.update(&state.db).await {
        let page = WebsiteSourceEditPage {
            id,
            website_runner_id: form.website_runner_id,
            runner_display: runner_display(&state.db, form.website_runner_id).await,
            url: form.url,
            depth: form.depth,
            filter: form.filter,
            is_filter_whitelist: form.is_filter_whitelist,
            error: e.to_string(),
        };
        return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
    }
    htmx.redirect(&WebsiteSourceDetailRouteTag::new(id).url())
}

pub async fn delete_post(
    Cap(state): Cap<WebsitesState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let _ = SourceEntity::delete_by_id(id).exec(&state.db).await;
    htmx.redirect(&WebsiteSourcesListRouteTag.url())
}

pub async fn unset_select(
    Cap(state): Cap<WebsitesState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<SourceUnsetSelectQuery>,
) -> Response {
    let sources = SourceEntity::find()
        .filter(website_source::Column::WebsiteRunnerId.is_null())
        .order_by_desc(website_source::Column::Id)
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
    let page = WebsiteSourceUnsetSelectPage {
        sources: list,
        filter_name: filter,
        target_input: q.target_input.unwrap_or_else(|| "source_ids".into()),
        path_and_query: path_and_query(&uri),
    };
    respond_picker_select::<WebsiteSourceUnsetSelectTableKey, WebsiteSourceUnsetSelectModalKey, _>(
        &htmx, &page,
    )
    .into_response()
}

pub struct SourceSelectRow {
    pub id: i64,
    pub label: String,
}
