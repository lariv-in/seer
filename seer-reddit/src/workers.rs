use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::time::Duration;
use tracing::warn;

use seer_common::{spawn_interval_worker, stop_active_worker, RUNNER_KIND_REDDIT};
use seer_workerregistry::{finish_worker_run_log, start_worker_run_log};

use crate::{
    entities::{
        reddit_runner::Entity as RunnerEntity,
        reddit_source::{self, Entity as SourceEntity},
    },
    fetch::fetch_new_reddit_posts,
};

pub fn start_all_runner_pools(db: DatabaseConnection) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let runners = RunnerEntity::find().all(&db).await.unwrap_or_default();
        for r in runners {
            start_runner_pool(db.clone(), r.id, r.name.clone(), r.duration_secs);
        }
    });
}

pub fn start_runner_pool(db: DatabaseConnection, runner_id: i64, name: String, duration_secs: i64) {
    let secs = duration_secs.max(60) as u64;
    spawn_interval_worker(
        RUNNER_KIND_REDDIT,
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
                    let log = match start_worker_run_log(&db, RUNNER_KIND_REDDIT, runner_id, &name)
                        .await
                    {
                        Ok(l) => l,
                        Err(e) => {
                            warn!("{e}");
                            return;
                        }
                    };
                    let result = run_pass(&db, runner_id).await;
                    let err = result.err().map(|e| e.to_string());
                    if let Err(e) = finish_worker_run_log(&db, &log, err.as_deref()).await {
                        warn!(error = %e, "failed to finish reddit worker run log");
                    }
                }
            }
        },
    );
}

pub fn stop_runner_pool(runner_id: i64) -> bool {
    stop_active_worker(RUNNER_KIND_REDDIT, runner_id)
}

async fn run_pass(db: &DatabaseConnection, runner_id: i64) -> anyhow::Result<()> {
    let sources = SourceEntity::find()
        .filter(reddit_source::Column::RedditRunnerId.eq(runner_id))
        .all(db)
        .await?;
    for s in sources {
        let _ = fetch_new_reddit_posts(db, &s).await?;
    }
    Ok(())
}
