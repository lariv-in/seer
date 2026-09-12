use maud::{Markup, html};
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};

use lariv_rs::{
    components::{SharedChromeFolder, SlotCtx},
    http::Cap,
    plugins::users::middleware::RequireAuth,
    web::{Htmx, html_built_page_or_app_layout},
};
use seer_common::{cell, col, list_active_workers, row, seer_table};

use crate::{
    entities::worker_run_log::{Entity as WorkerRunLogEntity, Model as WorkerRunLog},
    keys::{ActiveWorkersTableKey, WorkerRunLogsTableKey},
    state::WorkerRegistryState,
    templates::WorkersHomePage,
};

struct ActiveRow {
    kind: String,
    runner: String,
    name: String,
    started: String,
}

struct LogRow {
    kind: String,
    runner: String,
    status: String,
    started: String,
    duration: String,
    error: String,
}

pub async fn home(
    Cap(state): Cap<WorkerRegistryState>,
    Cap(chrome): Cap<SharedChromeFolder>,
    RequireAuth(ctx): RequireAuth,
    htmx: Htmx,
) -> Markup {
    let active = list_active_workers();
    let recent: Vec<WorkerRunLog> = WorkerRunLogEntity::find()
        .order_by_desc(crate::entities::worker_run_log::Column::Id)
        .limit(50)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let active_data: Vec<ActiveRow> = active
        .into_iter()
        .map(|w| ActiveRow {
            kind: w.kind,
            runner: w.runner_id.to_string(),
            name: w.name,
            started: ctx.format_datetime(w.started_at).into_string(),
        })
        .collect();
    let active_headers = [
        col("Kind", "Kind"),
        col("Runner", "Runner"),
        col("Name", "Name"),
        col("Started", "Started"),
    ];
    let active_rows: Vec<_> = active_data
        .iter()
        .map(|r| {
            row(vec![
                cell(&r.kind),
                cell(&r.runner),
                cell(&r.name),
                cell(&r.started),
            ])
        })
        .collect();

    let log_data: Vec<LogRow> = recent
        .into_iter()
        .map(|row| LogRow {
            kind: row.runner_kind,
            runner: format!("{} ({})", row.runner_name, row.runner_id),
            status: row.status.as_str().to_string(),
            started: ctx.format_datetime(row.started_at).into_string(),
            duration: row.duration_ms.to_string(),
            error: row.error_message,
        })
        .collect();
    let log_headers = [
        col("Kind", "Kind"),
        col("Runner", "Runner"),
        col("Status", "Status"),
        col("Started", "Started"),
        col("Duration", "Duration ms"),
        col("Error", "Error"),
    ];
    let log_rows: Vec<_> = log_data
        .iter()
        .map(|r| {
            row(vec![
                cell(&r.kind),
                cell(&r.runner),
                cell(&r.status),
                cell(&r.started),
                cell(&r.duration),
                cell(&r.error),
            ])
        })
        .collect();

    let body = html! {
        div class="space-y-6" {
            (seer_table::<ActiveWorkersTableKey>("Active workers", &active_headers, &active_rows))
            (seer_table::<WorkerRunLogsTableKey>("Recent run logs", &log_headers, &log_rows))
        }
    };

    let page = WorkersHomePage { body };
    html_built_page_or_app_layout(&page, &htmx, &chrome, &SlotCtx::from_auth(&ctx))
}
