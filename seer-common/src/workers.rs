//! Active worker bookkeeping shared across source plugins.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::watch;

#[derive(Clone, Debug)]
pub struct ActiveWorker {
    pub kind: String,
    pub runner_id: i64,
    pub name: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

struct WorkerHandle {
    info: ActiveWorker,
    cancel: watch::Sender<bool>,
    generation: u64,
}

static ACTIVE: OnceLock<RwLock<HashMap<(String, i64), WorkerHandle>>> = OnceLock::new();
static GENERATION: AtomicU64 = AtomicU64::new(1);

fn active() -> &'static RwLock<HashMap<(String, i64), WorkerHandle>> {
    ACTIVE.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn register_active_worker(info: ActiveWorker, cancel: watch::Sender<bool>) {
    let key = (info.kind.clone(), info.runner_id);
    let generation = GENERATION.fetch_add(1, Ordering::Relaxed);
    active().write().insert(
        key,
        WorkerHandle {
            info,
            cancel,
            generation,
        },
    );
}

fn register_active_worker_generation(
    info: ActiveWorker,
    cancel: watch::Sender<bool>,
    generation: u64,
) {
    let key = (info.kind.clone(), info.runner_id);
    active().write().insert(
        key,
        WorkerHandle {
            info,
            cancel,
            generation,
        },
    );
}

pub fn unregister_active_worker(kind: &str, runner_id: i64) {
    active().write().remove(&(kind.to_string(), runner_id));
}

fn unregister_active_worker_generation(kind: &str, runner_id: i64, generation: u64) {
    let mut map = active().write();
    let key = (kind.to_string(), runner_id);
    if map.get(&key).map(|h| h.generation) == Some(generation) {
        map.remove(&key);
    }
}

pub fn stop_active_worker(kind: &str, runner_id: i64) -> bool {
    if let Some(h) = active().write().remove(&(kind.to_string(), runner_id)) {
        if h.cancel.send(true).is_err() {
            tracing::warn!(kind, runner_id, "failed to signal worker cancel");
        }
        true
    } else {
        false
    }
}

pub fn is_active_worker(kind: &str, runner_id: i64) -> bool {
    active()
        .read()
        .contains_key(&(kind.to_string(), runner_id))
}

pub fn list_active_workers() -> Vec<ActiveWorker> {
    let mut out: Vec<_> = active().read().values().map(|h| h.info.clone()).collect();
    out.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.runner_id.cmp(&b.runner_id)));
    out
}

pub fn spawn_interval_worker<F, Fut>(
    kind: &str,
    runner_id: i64,
    name: &str,
    interval: std::time::Duration,
    mut tick: F,
) where
    F: FnMut() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    // Safe restart: cancel any existing handle for this key before registering.
    let _ = stop_active_worker(kind, runner_id);

    let generation = GENERATION.fetch_add(1, Ordering::Relaxed);
    let (tx, mut rx) = watch::channel(false);
    register_active_worker_generation(
        ActiveWorker {
            kind: kind.to_string(),
            runner_id,
            name: name.to_string(),
            started_at: chrono::Utc::now(),
        },
        tx,
        generation,
    );
    let kind = kind.to_string();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    tick().await;
                }
                _ = rx.changed() => {
                    if *rx.borrow() {
                        break;
                    }
                }
            }
        }
        unregister_active_worker_generation(&kind, runner_id, generation);
    });
}
