use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::time::Duration;
use tracing::warn;

use seer_common::{spawn_interval_worker, stop_active_worker, RUNNER_KIND_WEBSITE};
use seer_workerregistry::{finish_worker_run_log, start_worker_run_log};

use crate::{
    entities::{
        website_runner::Entity as RunnerEntity,
        website_source::{self, Entity as SourceEntity},
    },
    scrape::{crawl_url, drain_queue_once},
};

pub fn start_all_runner_pools(db: DatabaseConnection) {
    tokio::spawn(async move {
        // Delay so migrations can finish.
        tokio::time::sleep(Duration::from_secs(2)).await;
        let runners = RunnerEntity::find().all(&db).await.unwrap_or_default();
        for r in runners {
            start_runner_pool(db.clone(), r.id, r.name.clone(), r.duration_secs);
        }
        // Also drain ad-hoc scrape queue.
        spawn_interval_worker("website_queue", 0, "scrape-queue", Duration::from_secs(5), {
            let db = db.clone();
            move || {
                let db = db.clone();
                async move {
                    drain_queue_once(&db).await;
                }
            }
        });
    });
}

pub fn start_runner_pool(db: DatabaseConnection, runner_id: i64, name: String, duration_secs: i64) {
    let secs = duration_secs.max(30) as u64;
    spawn_interval_worker(
        RUNNER_KIND_WEBSITE,
        runner_id,
        &name,
        Duration::from_secs(secs),
        {
            let db = db.clone();
            let name = name.clone();
            move || {
                let db = db.clone();
                let name = name.clone();
                async move {
                    let log = match start_worker_run_log(&db, RUNNER_KIND_WEBSITE, runner_id, &name)
                        .await
                    {
                        Ok(l) => l,
                        Err(e) => {
                            warn!("start run log: {e}");
                            return;
                        }
                    };
                    let result = run_pass(&db, runner_id).await;
                    let err = result.as_ref().err().map(|e| e.to_string());
                    if let Err(e) = finish_worker_run_log(&db, &log, err.as_deref()).await {
                        warn!(error = %e, "failed to finish website worker run log");
                    }
                }
            }
        },
    );
}

pub fn stop_runner_pool(runner_id: i64) -> bool {
    stop_active_worker(RUNNER_KIND_WEBSITE, runner_id)
}

async fn run_pass(db: &DatabaseConnection, runner_id: i64) -> anyhow::Result<()> {
    let sources = SourceEntity::find()
        .filter(website_source::Column::WebsiteRunnerId.eq(runner_id))
        .all(db)
        .await?;
    for src in sources {
        if let Err(e) = crawl_url(db, &src.url, src.depth, &src.filter, src.is_filter_whitelist).await {
            warn!("website source {}: {e:#}", src.id);
        }
    }
    Ok(())
}
