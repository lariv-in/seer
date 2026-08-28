//! Runnable job interface (parity with p_seer_runners).

use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use parking_lot::RwLock;

#[async_trait]
pub trait Runnable: Send + Sync {
    async fn run(&self, db: &DatabaseConnection) -> anyhow::Result<()>;
}

type DynRunnable = Arc<dyn Runnable>;

static REGISTRY: OnceLock<RwLock<HashMap<String, DynRunnable>>> = OnceLock::new();

fn registry() -> &'static RwLock<HashMap<String, DynRunnable>> {
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn register_runnable(name: impl Into<String>, runnable: impl Runnable + 'static) {
    registry().write().insert(name.into(), Arc::new(runnable));
}

pub fn get_runnable(name: &str) -> Option<DynRunnable> {
    registry().read().get(name).cloned()
}

pub fn list_runnables() -> Vec<String> {
    let mut keys: Vec<_> = registry().read().keys().cloned().collect();
    keys.sort();
    keys
}
