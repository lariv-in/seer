use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use std::time::Duration;
use tracing::warn;

use seer_common::{RUNNER_KIND_GDELT, spawn_interval_worker, stop_active_worker};
use seer_workerregistry::{finish_worker_run_log, start_worker_run_log};

use crate::{
    bigquery,
    entities::{
        event::ActiveModel as EvAM,
        gdelt_source::{self, Entity as SourceEntity},
        gdelt_worker::Entity as WorkerEntity,
    },
    preferences::resolved_config,
};

pub fn start_all_runner_pools(db: DatabaseConnection) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        for w in WorkerEntity::find().all(&db).await.unwrap_or_default() {
            start_runner_pool(db.clone(), w.id, w.name.clone(), w.duration_secs);
        }
    });
}

pub fn start_runner_pool(db: DatabaseConnection, worker_id: i64, name: String, duration_secs: i64) {
    let secs = duration_secs.max(300) as u64;
    spawn_interval_worker(
        RUNNER_KIND_GDELT,
        worker_id,
        &name,
        Duration::from_secs(secs),
        {
            let db = db.clone();
            let name = name.clone();
            move || {
                let db = db.clone();
                let name = name.clone();
                async move {
                    let log =
                        match start_worker_run_log(&db, RUNNER_KIND_GDELT, worker_id, &name).await {
                            Ok(l) => l,
                            Err(e) => {
                                warn!("{e}");
                                return;
                            }
                        };
                    let err = run_pass(&db, worker_id).await.err().map(|e| e.to_string());
                    if let Err(e) = finish_worker_run_log(&db, &log, err.as_deref()).await {
                        warn!(error = %e, "failed to finish gdelt worker run log");
                    }
                }
            }
        },
    );
}

pub fn stop_runner_pool(worker_id: i64) -> bool {
    stop_active_worker(RUNNER_KIND_GDELT, worker_id)
}

async fn run_pass(db: &DatabaseConnection, worker_id: i64) -> anyhow::Result<()> {
    let cfg = resolved_config(db).await?;
    let sources = SourceEntity::find()
        .filter(gdelt_source::Column::GdeltWorkerId.eq(worker_id))
        .all(db)
        .await?;
    for s in sources {
        let sql = bigquery::default_gdelt_sql(&s.query, s.max_records);
        let rows = match bigquery::fetch_gdelt_rows(&cfg, &sql).await {
            Ok(r) => r,
            Err(e) => {
                warn!("gdelt fetch: {e:#}");
                continue;
            }
        };
        for row in rows {
            // Best-effort parse of BQ REST row shape: f: [{v: ...}, ...]
            let cols = row
                .get("f")
                .and_then(|f| f.as_array())
                .cloned()
                .unwrap_or_default();
            let get = |i: usize| {
                cols.get(i)
                    .and_then(|c| c.get("v"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let now = Utc::now();
            if let Err(e) = (EvAM {
                gdelt_source_id: Set(Some(s.id)),
                global_event_id: Set(get(0)),
                source_url: Set(get(1)),
                actor1_name: Set(get(2)),
                actor2_name: Set(get(3)),
                action_geo_lat: Set(get(4).parse().ok()),
                action_geo_long: Set(get(5).parse().ok()),
                event_date: Set(None),
                created_at: Set(Some(now)),
                updated_at: Set(Some(now)),
                ..Default::default()
            })
            .insert(db)
            .await
            {
                warn!(error = %e, "failed to insert gdelt event");
            }
        }
    }
    Ok(())
}
