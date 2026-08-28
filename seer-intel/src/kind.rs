//! IntelKind trait + registry (parity with Go p_seer_intel).

use async_trait::async_trait;
use parking_lot::RwLock;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

#[async_trait]
pub trait IntelKind: Send + Sync {
    fn content(&self) -> String;
    fn kind(&self) -> &str;
    fn intel_id(&self) -> i64;
    async fn intel_detail(&self) -> anyhow::Result<String>;
}

pub type DynIntelKind = Arc<dyn IntelKind>;

#[async_trait]
pub trait IntelKindLoader: Send + Sync {
    async fn load(
        &self,
        db: &DatabaseConnection,
        id: i64,
    ) -> anyhow::Result<DynIntelKind>;
}

type LoaderMap = HashMap<String, Arc<dyn IntelKindLoader>>;

fn registry() -> &'static RwLock<LoaderMap> {
    static REG: OnceLock<RwLock<LoaderMap>> = OnceLock::new();
    REG.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn register_intel_kind(kind: impl Into<String>, loader: impl IntelKindLoader + 'static) {
    registry()
        .write()
        .insert(kind.into(), Arc::new(loader));
}

pub async fn load_intel_kind(
    db: &DatabaseConnection,
    kind: &str,
    id: i64,
) -> anyhow::Result<DynIntelKind> {
    let loader = registry()
        .read()
        .get(kind)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("unknown intel kind {kind}"))?;
    let k = loader.load(db, id).await?;
    if k.kind() != kind {
        anyhow::bail!("kind mismatch: registry {kind}, instance {}", k.kind());
    }
    Ok(k)
}

pub fn list_intel_kinds() -> Vec<String> {
    let mut keys: Vec<_> = registry().read().keys().cloned().collect();
    keys.sort();
    keys
}
