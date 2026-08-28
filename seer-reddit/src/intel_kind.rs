use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait};
use std::sync::Arc;
use seer_intel::{register_intel_kind, DynIntelKind, IntelKind, IntelKindLoader};
use crate::entities::reddit_post::{Entity as PostEntity, Model as Post};

pub struct RedditIntelKind {
    pub id: i64, pub title: String, pub selftext: String, pub subreddit: String, pub permalink: String,
}
impl From<Post> for RedditIntelKind {
    fn from(p: Post) -> Self {
        Self { id: p.id, title: p.title, selftext: p.selftext, subreddit: p.subreddit, permalink: p.permalink }
    }
}
#[async_trait]
impl IntelKind for RedditIntelKind {
    fn content(&self) -> String {
        format!("r/{}: {}\n\n{}\n\n{}", self.subreddit, self.title, self.selftext, self.permalink)
    }
    fn kind(&self) -> &str { "reddit" }
    fn intel_id(&self) -> i64 { self.id }
    async fn intel_detail(&self) -> anyhow::Result<String> { Ok(format!("/seer-reddit/posts/{}/", self.id)) }
}
struct Loader;
#[async_trait]
impl IntelKindLoader for Loader {
    async fn load(&self, db: &DatabaseConnection, id: i64) -> anyhow::Result<DynIntelKind> {
        let row = PostEntity::find_by_id(id).one(db).await?.ok_or_else(|| anyhow::anyhow!("missing"))?;
        Ok(Arc::new(RedditIntelKind::from(row)))
    }
}
pub fn register() { register_intel_kind("reddit", Loader); }
