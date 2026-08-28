//! LLM-backed content filter for scraped website pages.

use seer_intel::{config::SeerIntelConfig, filter::passes_content_filter};

pub async fn filter_content(
    config: &SeerIntelConfig,
    filter_text: &str,
    is_whitelist: bool,
    url: &str,
    markdown: &str,
) -> bool {
    let content = format!("URL: {url}\n\nContent:\n{markdown}");
    passes_content_filter(config, filter_text, is_whitelist, &content).await
}
