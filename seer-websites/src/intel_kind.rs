use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::sync::Arc;

use seer_intel::{register_intel_kind, DynIntelKind, IntelKind, IntelKindLoader};

use crate::entities::website::{Entity as WebsiteEntity, Model as Website};
use crate::routes::WebsiteDetailRouteTag;

pub struct WebsiteIntelKind {
    pub id: i64,
    pub url: String,
    pub markdown: String,
}

#[async_trait]
impl IntelKind for WebsiteIntelKind {
    fn content(&self) -> String {
        format!("# {}\n\n{}", self.url, self.markdown)
    }
    fn kind(&self) -> &str {
        "website"
    }
    fn intel_id(&self) -> i64 {
        self.id
    }
    async fn intel_detail(&self) -> anyhow::Result<String> {
        Ok(WebsiteDetailRouteTag::new(self.id).url())
    }
}

struct Loader;

#[async_trait]
impl IntelKindLoader for Loader {
    async fn load(&self, db: &DatabaseConnection, id: i64) -> anyhow::Result<DynIntelKind> {
        let row = WebsiteEntity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("website {id} not found"))?;
        Ok(Arc::new(WebsiteIntelKind {
            id: row.id,
            url: row.url,
            markdown: row.markdown,
        }))
    }
}

pub fn register() {
    register_intel_kind("website", Loader);
}

impl From<Website> for WebsiteIntelKind {
    fn from(w: Website) -> Self {
        Self {
            id: w.id,
            url: w.url,
            markdown: w.markdown,
        }
    }
}
