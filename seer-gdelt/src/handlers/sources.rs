//! GDELT source CRUD + unset multi-select picker.

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
use maud::{Markup, html};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use seer_common::{cell, col, row_nav, seer_table_with_actions};
use serde::Deserialize;

use crate::{
    entities::{
        gdelt_source::{self, ActiveModel as SourceAM, Entity as SourceEntity},
        gdelt_worker::Entity as WorkerEntity,
    },
    forms::SourceForm,
    keys::{
        GdeltSourceCreateModalKey, GdeltSourceUnsetSelectModalKey, GdeltSourceUnsetSelectTableKey,
        GdeltSourcesTableKey,
    },
    routes::{
        GdeltSourceDetailRouteTag, GdeltSourceEditGetRouteTag, GdeltSourcesListRouteTag,
    },
    state::GdeltState,
    templates::{
        GdeltSourceCreatePage, GdeltSourceDetailPage, GdeltSourceEditPage,
        GdeltSourceUnsetSelectPage, GdeltSourcesListPage,
    },
};

use super::{format_opt_date, parse_opt_date, source_label};

pub struct SourceSelectRow {
    pub id: i64,
    pub label: String,
}

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

async fn worker_display(db: &sea_orm::DatabaseConnection, worker_id: Option<i64>) -> String {
    let Some(id) = worker_id.filter(|&i| i > 0) else {
        return String::new();
    };
    lariv_rs::web::opt_or_log(
        WorkerEntity::find_by_id(id).one(db).await,
        "find worker for display",
    )
    .map(|w| w.name)
    .unwrap_or_default()
}

/// Validated pieces of a submitted source form.
struct ParsedSource {
    start_date: Option<chrono::DateTime<Utc>>,
    end_date: Option<chrono::DateTime<Utc>>,
}

fn validate(form: &SourceForm, tz: &str) -> Result<ParsedSource, String> {
    if form.query.trim().is_empty() {
        return Err("Query is required".into());
    }
    Ok(ParsedSource {
        start_date: parse_opt_date(&form.start_date, tz)?,
        end_date: parse_opt_date(&form.end_date, tz)?,
    })
}

/// Rebuild the create page from raw form input so the user keeps their edits on error.
async fn create_page_from_form(
    db: &sea_orm::DatabaseConnection,
    form: SourceForm,
    q: &ModalFormQuery,
    error: String,
) -> GdeltSourceCreatePage {
    GdeltSourceCreatePage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        worker_display: worker_display(db, form.gdelt_worker_id).await,
        gdelt_worker_id: form.gdelt_worker_id,
        query: form.query,
        domain: form.domain,
        action_country: form.action_country,
        start_date: form.start_date,
        end_date: form.end_date,
        min_mentions: form.min_mentions,
        max_records: form.max_records,
        sort: form.sort,
        natural_language_filter: form.natural_language_filter,
        is_blacklist: form.is_blacklist,
        error,
    }
}

/// Rebuild the edit page from raw form input so the user keeps their edits on error.
async fn edit_page_from_form(
    db: &sea_orm::DatabaseConnection,
    id: i64,
    form: SourceForm,
    error: String,
) -> GdeltSourceEditPage {
    GdeltSourceEditPage {
        id,
        worker_display: worker_display(db, form.gdelt_worker_id).await,
        gdelt_worker_id: form.gdelt_worker_id,
        query: form.query,
        domain: form.domain,
        action_country: form.action_country,
        start_date: form.start_date,
        end_date: form.end_date,
        min_mentions: form.min_mentions,
        max_records: form.max_records,
        sort: form.sort,
        natural_language_filter: form.natural_language_filter,
        is_blacklist: form.is_blacklist,
        error,
    }
}

pub async fn list(
    Cap(state): Cap<GdeltState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let sources = SourceEntity::find()
        .order_by_desc(gdelt_source::Column::Id)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let headers = [
        col("Query", "Query"),
        col("Domain", "Domain"),
        col("Country", "Country"),
        col("Max", "Max records"),
        col("Filter", "Filter"),
    ];
    let rows: Vec<_> = sources
        .iter()
        .map(|s| {
            let href = GdeltSourceDetailRouteTag::new(s.id).url();
            row_nav(
                &href,
                vec![
                    cell(&s.query),
                    cell(&s.domain),
                    cell(&s.action_country),
                    cell(&s.max_records.to_string()),
                    cell(if s.is_blacklist { "Blacklist" } else { "Whitelist" }),
                ],
            )
        })
        .collect();

    let actions = table_create_button::<GdeltSourcesTableKey, GdeltSourceCreateModalKey>(
        Some("plus"),
        "btn-square btn-outline btn-sm",
    );
    let body =
        seer_table_with_actions::<GdeltSourcesTableKey>("Sources", actions, &headers, &rows);
    html_built_page_or_app_layout(
        &GdeltSourcesListPage { body },
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
    let page = GdeltSourceCreatePage {
        form_name: q.form_name(),
        refresh_table: q.refresh_table(),
        target_input: q.target_input(),
        gdelt_worker_id: None,
        worker_display: String::new(),
        query: String::new(),
        domain: String::new(),
        action_country: String::new(),
        start_date: String::new(),
        end_date: String::new(),
        min_mentions: 0,
        max_records: 100,
        sort: String::new(),
        natural_language_filter: String::new(),
        is_blacklist: false,
        error: String::new(),
    };
    html_built_page_with_slots(&page, &chrome, &SlotCtx::from_auth(&ctx))
}

pub async fn create_post(
    Cap(state): Cap<GdeltState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Query(q): Query<ModalFormQuery>,
    HtmlFormBody(form): HtmlFormBody<SourceForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let parsed = match validate(&form, &ctx.timezone) {
        Ok(p) => p,
        Err(e) => {
            let page = create_page_from_form(&state.db, form, &q, e).await;
            return html_built_page_with_slots(&page, &chrome, &slot_ctx).into_response();
        }
    };

    let now = Utc::now();
    let row = match (SourceAM {
        gdelt_worker_id: Set(form.gdelt_worker_id.filter(|&i| i > 0)),
        query: Set(form.query.trim().into()),
        domain: Set(form.domain.trim().into()),
        action_country: Set(form.action_country.trim().into()),
        start_date: Set(parsed.start_date),
        end_date: Set(parsed.end_date),
        min_mentions: Set(form.min_mentions.max(0)),
        max_records: Set(if form.max_records > 0 {
            form.max_records
        } else {
            100
        }),
        sort: Set(form.sort.trim().into()),
        natural_language_filter: Set(form.natural_language_filter.clone()),
        is_blacklist: Set(form.is_blacklist),
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
    respond_create_modal_done_fk::<GdeltSourceCreateModalKey>(
        &htmx,
        &q.refresh_table(),
        &GdeltSourceDetailRouteTag::new(row.id).url(),
        row.id,
        &label,
        &q.target_input(),
    )
}

pub async fn detail(
    Cap(state): Cap<GdeltState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(s) = lariv_rs::web::opt_or_log(
        SourceEntity::find_by_id(id).one(&state.db).await,
        "find gdelt source",
    ) else {
        return Redirect::to(&GdeltSourcesListRouteTag.url()).into_response();
    };
    let worker_name = worker_display(&state.db, s.gdelt_worker_id).await;
    let worker_label = if worker_name.is_empty() {
        "—".into()
    } else {
        worker_name
    };
    let filter_mode = if s.is_blacklist {
        "Blacklist"
    } else {
        "Whitelist"
    };
    let dash = |v: String| if v.is_empty() { "—".to_string() } else { v };
    let start = dash(format_opt_date(s.start_date, &ctx.timezone));
    let end = dash(format_opt_date(s.end_date, &ctx.timezone));
    let body = html! {
        div class="container mx-auto p-4 space-y-4" {
            div class="flex flex-wrap gap-2" {
                (button_link_route(GdeltSourceEditGetRouteTag::new(id), "Edit", "btn-outline"))
                form method="post" action=(format!("/seer-gdelt/sources/{id}/delete"))
                    hx-boost="true" hx-confirm="Delete this source?"
                {
                    button class="btn btn-error btn-outline" type="submit" { "Delete" }
                }
            }
            dl class="space-y-2" {
                div { dt class="font-semibold" { "Worker" } dd { (worker_label) } }
                div { dt class="font-semibold" { "Query" } dd { (source_label(&s)) } }
                div { dt class="font-semibold" { "Domain" } dd { (s.domain.clone()) } }
                div { dt class="font-semibold" { "Action country" } dd { (s.action_country.clone()) } }
                div { dt class="font-semibold" { "Start date" } dd { (start) } }
                div { dt class="font-semibold" { "End date" } dd { (end) } }
                div { dt class="font-semibold" { "Min mentions" } dd { (s.min_mentions.to_string()) } }
                div { dt class="font-semibold" { "Max records" } dd { (s.max_records.to_string()) } }
                div { dt class="font-semibold" { "Sort" } dd { (s.sort.clone()) } }
                div { dt class="font-semibold" { "Filter" } dd {
                    (filter_mode) ": " (s.natural_language_filter.clone())
                } }
            }
        }
    };
    html_built_page_or_app_layout(
        &GdeltSourceDetailPage { body },
        &htmx,
        &chrome,
        &SlotCtx::from_auth(&ctx),
    )
    .into_response()
}

pub async fn edit_get(
    Cap(state): Cap<GdeltState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let Some(s) = lariv_rs::web::opt_or_log(
        SourceEntity::find_by_id(id).one(&state.db).await,
        "find gdelt source",
    ) else {
        return Redirect::to(&GdeltSourcesListRouteTag.url()).into_response();
    };
    let page = GdeltSourceEditPage {
        id,
        gdelt_worker_id: s.gdelt_worker_id,
        worker_display: worker_display(&state.db, s.gdelt_worker_id).await,
        query: s.query,
        domain: s.domain,
        action_country: s.action_country,
        start_date: format_opt_date(s.start_date, &ctx.timezone),
        end_date: format_opt_date(s.end_date, &ctx.timezone),
        min_mentions: s.min_mentions,
        max_records: s.max_records,
        sort: s.sort,
        natural_language_filter: s.natural_language_filter,
        is_blacklist: s.is_blacklist,
        error: String::new(),
    };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx)).into_response()
}

pub async fn edit_post(
    Cap(state): Cap<GdeltState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
    HtmlFormBody(form): HtmlFormBody<SourceForm>,
) -> Response {
    let slot_ctx = SlotCtx::from_auth(&ctx);
    let Some(existing) = lariv_rs::web::opt_or_log(
        SourceEntity::find_by_id(id).one(&state.db).await,
        "find gdelt source",
    ) else {
        return Redirect::to(&GdeltSourcesListRouteTag.url()).into_response();
    };

    let parsed = match validate(&form, &ctx.timezone) {
        Ok(p) => p,
        Err(e) => {
            let page = edit_page_from_form(&state.db, id, form, e).await;
            return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
        }
    };

    let mut am: SourceAM = existing.into();
    am.gdelt_worker_id = Set(form.gdelt_worker_id.filter(|&i| i > 0));
    am.query = Set(form.query.trim().into());
    am.domain = Set(form.domain.trim().into());
    am.action_country = Set(form.action_country.trim().into());
    am.start_date = Set(parsed.start_date);
    am.end_date = Set(parsed.end_date);
    am.min_mentions = Set(form.min_mentions.max(0));
    am.max_records = Set(if form.max_records > 0 {
        form.max_records
    } else {
        100
    });
    am.sort = Set(form.sort.trim().into());
    am.natural_language_filter = Set(form.natural_language_filter.clone());
    am.is_blacklist = Set(form.is_blacklist);
    am.updated_at = Set(Some(Utc::now()));
    if let Err(e) = am.update(&state.db).await {
        let page = edit_page_from_form(&state.db, id, form, e.to_string()).await;
        return html_built_page_or_app_layout(&page, &htmx, &chrome, &slot_ctx).into_response();
    }
    htmx.redirect(&GdeltSourceDetailRouteTag::new(id).url())
}

pub async fn delete_post(
    Cap(state): Cap<GdeltState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    Path(id): Path<i64>,
) -> Response {
    let _ = SourceEntity::delete_by_id(id).exec(&state.db).await;
    htmx.redirect(&GdeltSourcesListRouteTag.url())
}

pub async fn unset_select(
    Cap(state): Cap<GdeltState>,
    RequireAuth(_ctx): RequireAuth,
    htmx: Htmx,
    uri: Uri,
    Query(q): Query<SourceUnsetSelectQuery>,
) -> Response {
    let sources = SourceEntity::find()
        .filter(gdelt_source::Column::GdeltWorkerId.is_null())
        .order_by_desc(gdelt_source::Column::Id)
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
    let page = GdeltSourceUnsetSelectPage {
        sources: list,
        filter_name: filter,
        target_input: q.target_input.unwrap_or_else(|| "source_ids".into()),
        path_and_query: path_and_query(&uri),
    };
    respond_picker_select::<GdeltSourceUnsetSelectTableKey, GdeltSourceUnsetSelectModalKey, _>(
        &htmx, &page,
    )
    .into_response()
}
