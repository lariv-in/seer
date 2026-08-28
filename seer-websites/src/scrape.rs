//! Scrape via seer_node_fleet::DispatchCommand + optional URL queue.

use chrono::Utc;
use parking_lot::Mutex;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, OnceLock};
use tracing::{info, warn};

use seer_node_fleet::{
    command, response, response_ok, trigger_scraper, Command, DispatchCommand, TriggerScraper,
    WebsiteScraperArgs,
};

use crate::{
    crawl::enqueue_discovered,
    entities::website::{self, ActiveModel as WebsiteAM, Entity as WebsiteEntity},
    filter_llm::filter_content,
    intel_kind::WebsiteIntelKind,
    ssrf::{normalize_website_url, validate_public_url},
};

static URL_QUEUE: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

fn queue() -> &'static Mutex<VecDeque<String>> {
    URL_QUEUE.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn enqueue_scrape_url(url: impl Into<String>) {
    queue().lock().push_back(url.into());
}

pub fn pop_scrape_url() -> Option<String> {
    queue().lock().pop_front()
}

pub struct ScrapeResult {
    pub source_url: String,
    pub content: String,
    pub rendered_html: String,
}

pub async fn scrape_via_fleet(url: &str) -> anyhow::Result<ScrapeResult> {
    let parsed = normalize_website_url(url)?;
    validate_public_url(&parsed)?;
    let source_url = parsed.to_string();

    static CMD_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(100);
    let id = CMD_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let cmd = Command {
        id,
        command_type: Some(command::CommandType::TriggerScraper(TriggerScraper {
            scraper_args: Some(trigger_scraper::ScraperArgs::WebsiteScraper(
                WebsiteScraperArgs {
                    source_url: source_url.clone(),
                },
            )),
        })),
    };
    let resp = DispatchCommand(cmd).await?;
    match resp.response_type {
        Some(response::ResponseType::Ok(ok)) => match ok.response {
            Some(response_ok::Response::WebsiteScraper(w)) => Ok(ScrapeResult {
                source_url: w.source_url,
                content: w.content,
                rendered_html: w.rendered_html,
            }),
            _ => anyhow::bail!("unexpected ok response"),
        },
        Some(response::ResponseType::Error(_)) => anyhow::bail!("scrape error from node"),
        None => anyhow::bail!("empty response"),
    }
}

async fn persist_scrape(db: &DatabaseConnection, result: &ScrapeResult) -> anyhow::Result<i64> {
    let now = Utc::now();
    let existing = WebsiteEntity::find()
        .filter(website::Column::Url.eq(&result.source_url))
        .one(db)
        .await?;
    let model = if let Some(row) = existing {
        let mut am: WebsiteAM = row.into();
        am.markdown = Set(result.content.clone());
        am.updated_at = Set(Some(now));
        am.update(db).await?
    } else {
        WebsiteAM {
            url: Set(result.source_url.clone()),
            markdown: Set(result.content.clone()),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(db)
        .await?
    };
    seer_intel::enqueue_intel(Arc::new(WebsiteIntelKind {
        id: model.id,
        url: model.url.clone(),
        markdown: model.markdown.clone(),
    }));
    info!("seer-websites: scraped {}", model.url);
    Ok(model.id)
}

pub async fn scrape_url(db: &DatabaseConnection, url: &str) -> anyhow::Result<i64> {
    let result = scrape_via_fleet(url).await?;
    persist_scrape(db, &result).await
}

/// BFS same-origin crawl from `seed`. `depth` is hops from the seed (0 = seed only).
pub async fn crawl_url(
    db: &DatabaseConnection,
    seed: &str,
    depth: i64,
    filter: &str,
    is_filter_whitelist: bool,
) -> anyhow::Result<usize> {
    let depth = depth.max(0) as u32;
    let seed_url = normalize_website_url(seed)?;
    validate_public_url(&seed_url)?;
    let config = seer_intel::preferences::resolved_config(db).await?;

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(seed_url.to_string());
    queue.push_back((seed_url.to_string(), 0u32));

    let mut scraped = 0usize;
    while let Some((url, d)) = queue.pop_front() {
        let result = match scrape_via_fleet(&url).await {
            Ok(r) => r,
            Err(e) => {
                warn!("seer-websites: crawl scrape failed for {url}: {e:#}");
                continue;
            }
        };
        visited.insert(result.source_url.clone());
        if filter_content(
            &config,
            filter,
            is_filter_whitelist,
            &result.source_url,
            &result.content,
        )
        .await
        {
            if let Err(e) = persist_scrape(db, &result).await {
                warn!(
                    "seer-websites: crawl persist failed for {}: {e:#}",
                    result.source_url
                );
                continue;
            }
            scraped += 1;
        }

        let remaining = depth.saturating_sub(d);
        enqueue_discovered(
            &result.rendered_html,
            &result.source_url,
            remaining,
            &mut |link| {
                if visited.insert(link.clone()) {
                    queue.push_back((link, d + 1));
                }
            },
        );
    }
    Ok(scraped)
}

pub async fn drain_queue_once(db: &DatabaseConnection) {
    if let Some(url) = pop_scrape_url() {
        if let Err(e) = scrape_url(db, &url).await {
            warn!("seer-websites: queue scrape failed: {e:#}");
        }
    }
}
