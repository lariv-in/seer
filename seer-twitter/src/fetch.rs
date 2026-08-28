use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc; use tracing::warn;
use crate::{
    config::SeerTwitterConfig,
    entities::{twitter_post::{self, ActiveModel as PostAM, Entity as PostEntity}, twitter_source::Model as Source},
    filter_llm::filter_posts_llm_stub, intel_kind::TwitterIntelKind,
};

pub async fn fetch_new_twitter_posts(db: &DatabaseConnection, cfg: &SeerTwitterConfig, source: &Source) -> anyhow::Result<usize> {
    let users: Vec<String> = match &source.usernames {
        serde_json::Value::Array(a) => a.iter().filter_map(|v| v.as_str().map(|s| s.trim_start_matches('@').to_string())).filter(|s| !s.is_empty()).collect(),
        serde_json::Value::String(s) => s.split(',').map(|x| x.trim().trim_start_matches('@').to_string()).filter(|s| !s.is_empty()).collect(),
        _ => vec![],
    };
    let base = cfg.nitter_instance_url.trim_end_matches('/');
    let client = reqwest::Client::builder().user_agent("seer-rs/0.1").build()?;
    let mut inserted = 0usize;
    let cap = source.max_fresh_posts.max(1) as usize;
    for user in users {
        let feed_url = format!("{base}/{user}/rss");
        let bytes = match client.get(&feed_url).send().await {
            Ok(r) => r.bytes().await.unwrap_or_default(),
            Err(e) => { warn!("nitter {feed_url}: {e}"); continue; }
        };
        let feed = match feed_rs::parser::parse(&bytes[..]) {
            Ok(f) => f, Err(e) => { warn!("parse: {e}"); continue; }
        };
        let mut fresh = 0usize;
        for entry in feed.entries {
            if fresh >= cap { break; }
            let post_id = entry.id.clone();
            if PostEntity::find().filter(twitter_post::Column::PostId.eq(&post_id)).one(db).await?.is_some() { continue; }
            let title = entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default();
            let selftext = entry.summary.as_ref().map(|s| s.content.clone()).unwrap_or_default();
            if !filter_posts_llm_stub(&source.filter, source.is_filter_whitelist, &title, &selftext).await { continue; }
            let permalink = entry.links.first().map(|l| l.href.clone()).unwrap_or_default();
            let now = Utc::now();
            let model = PostAM {
                twitter_runner_id: Set(source.twitter_runner_id),
                post_id: Set(post_id), title: Set(title), selftext: Set(selftext.clone()),
                author: Set(user.clone()), permalink: Set(permalink.clone()), url: Set(permalink),
                created_utc_unix: Set(entry.published.map(|t| t.timestamp() as f64).unwrap_or(0.0)),
                created_at: Set(Some(now)), updated_at: Set(Some(now)), ..Default::default()
            }.insert(db).await?;
            seer_intel::enqueue_intel(Arc::new(TwitterIntelKind::from(model.clone())));
            if source.load_websites {
                for u in extract_http_urls(&selftext) { seer_websites::enqueue_scrape_url(u); }
            }
            inserted += 1; fresh += 1;
        }
    }
    Ok(inserted)
}

fn extract_http_urls(text: &str) -> Vec<String> {
    let re = regex::Regex::new(r#"https?://[^\s<>\"']+"#).unwrap();
    re.find_iter(text).map(|m| m.as_str().trim_end_matches(['.',',',')']).to_string()).collect()
}
