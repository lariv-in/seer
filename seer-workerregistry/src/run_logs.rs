//! Start / Finish / List helpers for worker run logs.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect,
};

use crate::entities::worker_run_log::{
    self, ActiveModel, Entity as WorkerRunLogEntity, Model as WorkerRunLog, WorkerRunStatus,
};

pub async fn start_worker_run_log(
    db: &DatabaseConnection,
    kind: &str,
    runner_id: i64,
    runner_name: &str,
) -> Result<WorkerRunLog, sea_orm::DbErr> {
    let now = Utc::now();
    ActiveModel {
        runner_kind: Set(kind.to_string()),
        runner_id: Set(runner_id),
        runner_name: Set(runner_name.to_string()),
        status: Set(WorkerRunStatus::Pending),
        started_at: Set(now),
        finished_at: Set(None),
        duration_ms: Set(0),
        error_message: Set(String::new()),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
        ..Default::default()
    }
    .insert(db)
    .await
}

pub async fn finish_worker_run_log(
    db: &DatabaseConnection,
    log: &WorkerRunLog,
    run_err: Option<&str>,
) -> Result<(), sea_orm::DbErr> {
    if log.id == 0 {
        return Ok(());
    }
    let now = Utc::now();
    let (status, err_msg) = match run_err {
        Some(e) => (WorkerRunStatus::Error, e.to_string()),
        None => (WorkerRunStatus::Success, String::new()),
    };
    let dur = (now - log.started_at).num_milliseconds().max(0);
    let mut am: ActiveModel = log.clone().into();
    am.status = Set(status);
    am.finished_at = Set(Some(now));
    am.duration_ms = Set(dur);
    am.error_message = Set(err_msg);
    am.updated_at = Set(Some(now));
    am.update(db).await?;
    Ok(())
}

pub async fn latest_worker_run_log(
    db: &DatabaseConnection,
    kind: &str,
    runner_id: i64,
) -> Result<Option<WorkerRunLog>, sea_orm::DbErr> {
    WorkerRunLogEntity::find()
        .filter(worker_run_log::Column::RunnerKind.eq(kind))
        .filter(worker_run_log::Column::RunnerId.eq(runner_id))
        .filter(worker_run_log::Column::FinishedAt.is_not_null())
        .order_by_desc(worker_run_log::Column::FinishedAt)
        .order_by_desc(worker_run_log::Column::Id)
        .one(db)
        .await
}

pub async fn list_worker_run_logs(
    db: &DatabaseConnection,
    kind: &str,
    runner_id: i64,
    limit: u64,
) -> Result<Vec<WorkerRunLog>, sea_orm::DbErr> {
    let limit = if limit == 0 {
        100
    } else {
        limit.min(500)
    };
    WorkerRunLogEntity::find()
        .filter(worker_run_log::Column::RunnerKind.eq(kind))
        .filter(worker_run_log::Column::RunnerId.eq(runner_id))
        .order_by_desc(worker_run_log::Column::StartedAt)
        .order_by_desc(worker_run_log::Column::Id)
        .limit(limit)
        .all(db)
        .await
}
