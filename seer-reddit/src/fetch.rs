use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use std::sync::Arc;
use tracing::warn;

use crate::{
    entities::{
        reddit_post::{self, ActiveModel as PostAM, Entity as PostEntity},
        reddit_source::Model as Source,
    },
    filter_llm::filter_post,
    intel_kind::RedditIntelKind,
};

pub async fn fetch_new_reddit_posts(db: &DatabaseConnection, source: &Source) -> anyhow::Result<usize> {
    let subs: Vec<String> = match &source.subreddits {
        serde_json::Value::Array(a) => a
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.trim_start_matches("r/").to_string()))
            .filter(|s| !s.is_empty())
            .collect(),
        serde_json::Value::String(s) => s
            .split(',')
            .map(|x| x.trim().trim_start_matches("r/").to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => vec![],
    };
    let client = reqwest::Client::builder()
        .user_agent("seer-rs/0.1 (OSINT research)")
        .build()?;
    let config = seer_intel::preferences::resolved_config(db).await?;
    let mut inserted = 0usize;
    let cap = source.max_fresh_posts.max(1) as usize;
    for sub in subs {
        let feed_url = if source.search_query.trim().is_empty() {
            format!("https://www.reddit.com/r/{sub}/.rss")
        } else {
            let q = urlencoding_lite(&source.search_query);
            format!("https://www.reddit.com/r/{sub}/search.rss?q={q}&restrict_sr=1&sort=new")
        };
        let bytes = match client.get(&feed_url).send().await {
            Ok(r) => r.bytes().await.unwrap_or_default(),
            Err(e) => {
                warn!("reddit feed {feed_url}: {e}");
                continue;
            }
        };
        let feed = match feed_rs::parser::parse(&bytes[..]) {
            Ok(f) => f,
            Err(e) => {
                warn!("parse feed: {e}");
                continue;
            }
        };
        let mut fresh = 0usize;
        for entry in feed.entries {
            if fresh >= cap {
                break;
            }
            let post_id = entry
                .id
                .trim_start_matches("t3_")
                .rsplit('/')
                .next()
                .unwrap_or(&entry.id)
                .to_string();
            if PostEntity::find()
                .filter(reddit_post::Column::PostId.eq(&post_id))
                .one(db)
                .await?
                .is_some()
            {
                continue;
            }
            let title = entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default();
            let selftext = entry
                .summary
                .as_ref()
                .map(|s| s.content.clone())
                .or_else(|| entry.content.as_ref().map(|c| c.body.clone().unwrap_or_default()))
                .unwrap_or_default();
            if !filter_post(
                &config,
                &source.filter,
                source.is_filter_whitelist,
                &title,
                &selftext,
            )
            .await
            {
                continue;
            }
            let permalink = entry
                .links
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_default();
            let url = permalink.clone();
            let now = Utc::now();
            let model = PostAM {
                reddit_runner_id: Set(source.reddit_runner_id),
                post_id: Set(post_id),
                title: Set(title),
                selftext: Set(selftext.clone()),
                author: Set(entry
                    .authors
                    .first()
                    .map(|a| a.name.clone())
                    .unwrap_or_default()),
                subreddit: Set(sub.clone()),
                permalink: Set(permalink),
                url: Set(url.clone()),
                created_utc_unix: Set(entry
                    .published
                    .map(|t| t.timestamp() as f64)
                    .unwrap_or(0.0)),
                score: Set(0),
                num_comments: Set(0),
                is_self: Set(true),
                created_at: Set(Some(now)),
                updated_at: Set(Some(now)),
                ..Default::default()
            }
            .insert(db)
            .await?;
            seer_intel::enqueue_intel(Arc::new(RedditIntelKind::from(model.clone())));
            if source.load_websites {
                for u in extract_http_urls(&selftext) {
                    seer_websites::enqueue_scrape_url(u);
                }
                if url.starts_with("http") && !url.contains("reddit.com") {
                    seer_websites::enqueue_scrape_url(url);
                }
            }
            inserted += 1;
            fresh += 1;
        }
    }
    Ok(inserted)
}

fn urlencoding_lite(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

fn extract_http_urls(text: &str) -> Vec<String> {
    let re = regex::Regex::new(r#"https?://[^\s<>\"']+"#).unwrap();
    re.find_iter(text).map(|m| m.as_str().trim_end_matches(['.', ',', ')']).to_string()).collect()
}
