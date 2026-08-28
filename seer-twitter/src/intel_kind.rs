use async_trait::async_trait; use sea_orm::{DatabaseConnection, EntityTrait}; use std::sync::Arc;
use seer_intel::{register_intel_kind, DynIntelKind, IntelKind, IntelKindLoader};
use crate::entities::twitter_post::{Entity as PostEntity, Model as Post};
pub struct TwitterIntelKind { pub id: i64, pub title: String, pub selftext: String, pub author: String, pub permalink: String }
impl From<Post> for TwitterIntelKind {
    fn from(p: Post) -> Self { Self { id: p.id, title: p.title, selftext: p.selftext, author: p.author, permalink: p.permalink } }
}
#[async_trait]
impl IntelKind for TwitterIntelKind {
    fn content(&self) -> String { format!("@{}: {}\n\n{}\n\n{}", self.author, self.title, self.selftext, self.permalink) }
    fn kind(&self) -> &str { "twitter" }
    fn intel_id(&self) -> i64 { self.id }
    async fn intel_detail(&self) -> anyhow::Result<String> { Ok(format!("/seer-twitter/posts/{}/", self.id)) }
}
struct Loader;
#[async_trait]
impl IntelKindLoader for Loader {
    async fn load(&self, db: &DatabaseConnection, id: i64) -> anyhow::Result<DynIntelKind> {
        Ok(Arc::new(TwitterIntelKind::from(PostEntity::find_by_id(id).one(db).await?.ok_or_else(|| anyhow::anyhow!("missing"))?)))
    }
}
pub fn register() { register_intel_kind("twitter", Loader); }
