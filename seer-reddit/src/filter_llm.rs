//! LLM-backed content filter for Reddit posts.

use seer_intel::{config::SeerIntelConfig, filter::passes_content_filter};

pub async fn filter_post(
    config: &SeerIntelConfig,
    filter_text: &str,
    is_whitelist: bool,
    title: &str,
    body: &str,
) -> bool {
    let content = format!("Title: {title}\n\nBody:\n{body}");
    passes_content_filter(config, filter_text, is_whitelist, &content).await
}
