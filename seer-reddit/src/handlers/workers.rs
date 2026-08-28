//! Reddit worker (runner) CRUD + start/stop pool.

use axum::{
    extract::{Path, Query},
    http::Uri,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;
use lariv_rs::{
    components::{
        ManyToManyItem, ObjectList, SharedChromeFolder, SlotCtx, button_link_route,
        button_post_route, table_create_button,
    },
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
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use seer_common::{
    cell, col, format_duration_secs, is_active_worker, parse_duration_secs, row_nav,
    seer_table_with_actions_refresh, RUNNER_KIND_REDDIT,
};
use serde::Deserialize;

use crate::{
    entities::{
        reddit_runner::{self, ActiveModel as RunnerAM, Entity as RunnerEntity},
        reddit_source::{self, Entity as SourceEntity},
    },
    forms::WorkerForm,
    keys::{
        RedditWorkerCreateModalKey, RedditWorkerSelectModalKey, RedditWorkerSelectTableKey,
        RedditWorkersTableKey,
    },
    routes::{
        RedditWorkerDetailRouteTag, RedditWorkerEditGetRouteTag, RedditWorkerPoolStartRouteTag,
        RedditWorkerPoolStopRouteTag, RedditWorkersListRouteTag,
    },
    state::RedditState,
    templates::{
        RedditWorkerCreatePage, RedditWorkerDetailPage, RedditWorkerEditPage,
        RedditWorkerSelectPage, RedditWorkersListPage,
    },
    workers::{start_runner_pool, stop_runner_pool},
};

use super::source_label;

pub struct WorkerSelectRow {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct WorkerSelectQuery {
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

async fn source_items_for_runner(
    db: &DatabaseConnection,
    runner_id: i64,
) -> Vec<ManyToManyItem> {
    SourceEntity::find()
        .filter(reddit_source::Column::RedditRunnerId.eq(runner_id))
        .order_by_desc(reddit_source::Column::Id)
        .all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|s| ManyToManyItem::new(s.id.to_string(), source_label(&s)))
        .collect()
}

async fn source_items_from_ids(db: &DatabaseConnection, ids: &[i64]) -> Vec<ManyToManyItem> {
    if ids.is_empty() {
        return Vec::new();
    }
    let rows = SourceEntity::find()
        .filter(reddit_source::Column::Id.is_in(ids.to_vec()))
        .all(db)
        .await
        .unwrap_or_default();
    ids.iter()
        .filter_map(|id| {
            rows.iter()
                .find(|s| s.id == *id)
                .map(|s| ManyToManyItem::new(s.id.to_string(), source_label(s)))
        })
        .collect()
}

/// Sync source assignments for a runner (Go AfterSave semantics).
pub async fn sync_runner_sources(
    db: &DatabaseConnection,
    runner_id: i64,
    source_ids: &[i64],
) -> Result<(), sea_orm::DbErr> {
    // Detach all currently assigned to this runner.
    let assigned = SourceEntity::find()
        .filter(reddit_source::Column::RedditRunnerId.eq(runner_id))
        .all(db)
        .await?;
    for s in assigned {
        let mut am: reddit_source::ActiveModel = s.into();
        am.reddit_runner_id = Set(None);
        am.updated_at = Set(Some(Utc::now()));
        am.update(db).await?;
    }
    // Attach selected (must be unassigned or already this runner — after detach, only unassigned).
    for &sid in source_ids {
        let Some(s) = SourceEntity::find_by_id(sid).one(db).await? else {
            continue;
        };
        if s.reddit_runner_id.is_some() {
            continue; // already owned by another runner
        }
        let mut am: reddit_source::ActiveModel = s.into();
        am.reddit_runner_id = Set(Some(runner_id));
        am.updated_at = Set(Some(Utc::now()));
        am.update(db).await?;
    }
    Ok(())
}

pub async fn list(
    Cap(state): Cap<RedditState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
) -> Markup {
    let runners = RunnerEntity::find()
        .order_by_desc(reddit_runner::Column::Id)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let headers = [
        col("Name", "Name"),
        col("Duration", "Duration"),
        col("Status", "Status"),
    ];
    let rows: Vec<_> = runners
        .iter()
        .map(|r| {
            let href = RedditWorkerDetailRouteTag::new(r.id).url();
            let status = if is_active_worker(RUNNER_KIND_REDDIT, r.id) {
                "Running"
            } else {
                "Stopped"
            };
            row_nav(
                &href,
                vec![
                    cell(&r.name),
                    cell(&format_duration_secs(r.duration_secs)),
                    cell(status),
                ],
            )
        })
        .collect();

    let actions = table_create_button::<RedditWorkersTableKey, RedditWorkerCreateModalKey>(
        Some("plus"),
        "btn-square btn-outline btn-sm",
    );
    let pq = path_and_query(&uri);
    let table = seer_table_with_actions_refresh::<RedditWorkersTableKey>(
        "Workers",
        actions,
        &headers,
        &rows,
        &pq,
    );
    if htmx.targets::<RedditWorkersTableKey>() {
        return table;
    }
    let body = table;
    html_built_page_or_app_layout(
        &RedditWorkersListPage { body },
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
    let page = RedditWorkerCreatePage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        name: String::new(),
        duration: "1 hour".into(),
        sources: Vec::new(),
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn create_post(
    Cap(state): Cap<RedditState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<WorkerForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let render_error = |form: WorkerForm, error: String, sources: Vec<ManyToManyItem>| {
        let page = RedditWorkerCreatePage {
            form_name: q.form_name(),
            refresh_table: q.refresh_table(),
            target_input: q.target_input(),
            name: form.name,
            duration: form.duration,
            sources,
            error,
        };
        html_built_page_with_slots(&page, &chrome, &slot_ctx).into_response()
    };
    let duration_secs = match parse_duration_secs(&form.duration) {
        Ok(s) => s,
        Err(e) => {
            let sources = source_items_from_ids(&state.db, &form.source_ids).await;
            return render_error(form, e, sources);
        }
    };
    let name = form.name.trim().to_string();
    if name.is_empty() {
        let sources = source_items_from_ids(&state.db, &form.source_ids).await;
        return render_error(form, "Name is required".into(), sources);
    }
    let now = Utc::now();
    let row = match (RunnerAM {
        name: Set(name),
        duration_secs: Set(duration_secs),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    }
    .insert(&state.db)
    .await)
    {
        Ok(r) => r,
        Err(e) => {
            let sources = source_items_from_ids(&state.db, &form.source_ids).await;
            return render_error(form, e.to_string(), sources);
        }
    };
    let _ = sync_runner_sources(&state.db, row.id, &form.source_ids).await;
    respond_create_modal_done_fk::<RedditWorkerCreateModalKey>(
        &htmx,
        &q.refresh_table(),
        &RedditWorkerDetailRouteTag::new(row.id).url(),
        row.id,
        &row.name,
        &q.target_input(),
    )
}

pub async fn detail(
    Cap(state): Cap<RedditState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(r) = lariv_rs::web::opt_or_log(
        RunnerEntity::find_by_id(id).one(&state.db).await,
        "find reddit runner",
    ) else {
        return Redirect::to(&RedditWorkersListRouteTag.url()).into_response();
    };
    let sources = SourceEntity::find()
        .filter(reddit_source::Column::RedditRunnerId.eq(id))
        .order_by_desc(reddit_source::Column::Id)
        .all(&state.db)
        .await
        .unwrap_or_default();
    let running = is_active_worker(RUNNER_KIND_REDDIT, id);
    let edit_url = RedditWorkerEditGetRouteTag::new(id).url();
    let pool_btn = if running {
        button_post_route(RedditWorkerPoolStopRouteTag::new(id), "Stop worker pool", "btn-warning")
    } else {
        button_post_route(RedditWorkerPoolStartRouteTag::new(id), "Start worker pool", "btn-success")
    };
    let body = html! {
        div class="container mx-auto p-4 space-y-4" {
            div class="flex flex-wrap gap-2" {
                (pool_btn)
                (button_link_route(RedditWorkerEditGetRouteTag::new(id), "Edit", "btn-outline"))
                form method="post" action=(format!("/seer-reddit/workers/{id}/delete"))
                    hx-boost="true" hx-confirm="Delete this worker?"
                {
                    button class="btn btn-error btn-outline" type="submit" { "Delete" }
                }
            }
            dl class="space-y-2" {
                div { dt class="font-semibold" { "Name" } dd { (r.name.clone()) } }
                div { dt class="font-semibold" { "Duration" } dd { (format_duration_secs(r.duration_secs)) } }
                div { dt class="font-semibold" { "Status" } dd { (if running { "Running" } else { "Stopped" }) } }
            }
            h2 class="text-lg font-semibold mt-4" { "Assigned sources" }
            ul class="list-disc pl-6" {
                @for s in &sources {
                    li { (source_label(s)) }
                }
                @if sources.is_empty() {
                    li class="opacity-60" { "None" }
                }
            }
            p class="text-sm opacity-60" { a class="link" href=(edit_url) { "Edit" } " to change assignments." }
        }
    };
    html_built_page_or_app_layout(
        &RedditWorkerDetailPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
    .into_response()
}

pub async fn edit_get(
    Cap(state): Cap<RedditState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(r) = lariv_rs::web::opt_or_log(
        RunnerEntity::find_by_id(id).one(&state.db).await,
        "find reddit runner",
    ) else {
        return Redirect::to(&RedditWorkersListRouteTag.url()).into_response();
    };
    let sources = source_items_for_runner(&state.db, id).await;
    let page = RedditWorkerEditPage {
        id,
        name: r.name,
        duration: format_duration_secs(r.duration_secs),
        sources,
        error: String::new(),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<RedditState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    HtmlFormBody(form): HtmlFormBody<WorkerForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let Some(existing) = lariv_rs::web::opt_or_log(
        RunnerEntity::find_by_id(id).one(&state.db).await,
        "find reddit runner",
    ) else {
        return Redirect::to(&RedditWorkersListRouteTag.url()).into_response();
    };
    let duration_secs = match parse_duration_secs(&form.duration) {
        Ok(s) => s,
        Err(e) => {
            let sources = source_items_from_ids(&state.db, &form.source_ids).await;
            let page = RedditWorkerEditPage {
                id,
                name: form.name,
                duration: form.duration,
                sources,
                error: e,
            };
            return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
        }
    };
    let name = form.name.trim().to_string();
    if name.is_empty() {
        let sources = source_items_from_ids(&state.db, &form.source_ids).await;
        let page = RedditWorkerEditPage {
            id,
            name: form.name,
            duration: form.duration,
            sources,
            error: "Name is required".into(),
        };
        return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
    }
    let mut am: RunnerAM = existing.into();
    am.name = Set(name.clone());
    am.duration_secs = Set(duration_secs);
    am.updated_at = Set(Some(Utc::now()));
    if let Err(e) = am.update(&state.db).await {
        let sources = source_items_from_ids(&state.db, &form.source_ids).await;
        let page = RedditWorkerEditPage {
            id,
            name: form.name,
            duration: form.duration,
            sources,
            error: e.to_string(),
        };
        return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
    }
    let _ = sync_runner_sources(&state.db, id, &form.source_ids).await;
    // Restart if running so new duration takes effect.
    if is_active_worker(RUNNER_KIND_REDDIT, id) {
        start_runner_pool(state.db.clone(), id, name, duration_secs);
    }
    htmx.redirect(&RedditWorkerDetailRouteTag::new(id).url())
}

pub async fn delete_post(
    Cap(state): Cap<RedditState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let linked = SourceEntity::find()
        .filter(reddit_source::Column::RedditRunnerId.eq(id))
        .count(&state.db)
        .await
        .unwrap_or(0);
    if linked > 0 {
        // Refuse delete while sources reference this runner.
        return htmx.redirect(&RedditWorkerDetailRouteTag::new(id).url());
    }
    stop_runner_pool(id);
    let _ = RunnerEntity::delete_by_id(id).exec(&state.db).await;
    htmx.redirect(&RedditWorkersListRouteTag.url())
}

pub async fn pool_start(
    Cap(state): Cap<RedditState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    if let Some(r) = lariv_rs::web::opt_or_log(
        RunnerEntity::find_by_id(id).one(&state.db).await,
        "find reddit runner",
    ) {
        start_runner_pool(state.db.clone(), r.id, r.name, r.duration_secs);
    }
    htmx.redirect(&RedditWorkerDetailRouteTag::new(id).url())
}

pub async fn pool_stop(
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    stop_runner_pool(id);
    htmx.redirect(&RedditWorkerDetailRouteTag::new(id).url())
}

pub async fn select(
    Cap(state): Cap<RedditState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<WorkerSelectQuery>,
) -> Response {
    let mut query = RunnerEntity::find().order_by_desc(reddit_runner::Column::Id);
    if let Some(name) = q.name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        query = query.filter(reddit_runner::Column::Name.contains(name));
    }
    let runners = query
        .limit(50)
        .all(&state.db)
        .await
        .unwrap_or_default();
    let items: Vec<_> = runners
        .into_iter()
        .map(|r| (r.id, r.name))
        .collect();
    let list = ObjectList::from_page(
        items
            .iter()
            .map(|(id, name)| WorkerSelectRow {
                id: *id,
                name: name.clone(),
            })
            .collect(),
        1,
        50,
        items.len() as u64,
    );
    let page = RedditWorkerSelectPage {
        runners: list,
        filter_name: q.name.clone().unwrap_or_default(),
        target_input: q
            .target_input
            .unwrap_or_else(|| "reddit_runner_id".into()),
        path_and_query: path_and_query(&uri),
    };
    respond_picker_select::<RedditWorkerSelectTableKey, RedditWorkerSelectModalKey, _>(&htmx, &page)
        .into_response()
}
