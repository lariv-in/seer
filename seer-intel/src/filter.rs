//! LLM content filter for Reddit, websites, and other ingest modules.

use lariv_rs::genai::GenaiClient;
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, warn};

use crate::config::SeerIntelConfig;
use crate::preferences::api_key_from_prefs_or_env;

const FILTER_SYSTEM_PROMPT: &str = "\
You are a content filter for an intelligence ingest pipeline.
Given filter criteria and source content, decide whether the content should be saved.
Respond ONLY with a JSON object (no markdown, no commentary) with exactly these keys:
\"pass\": boolean — true if the content should be saved, false if it should be discarded.
\"reason\": string — one short sentence explaining the decision.";

const FILTER_MAX_OUTPUT_TOKENS: i32 = 256;
/// Keep prompts bounded during high-volume crawls.
const FILTER_CONTENT_MAX_CHARS: usize = 12_000;

#[derive(Debug, Deserialize)]
struct FilterLlmOut {
    pass: bool,
    #[serde(default)]
    reason: String,
}

/// Returns true when content should be saved (passes the filter).
pub async fn passes_content_filter(
    config: &SeerIntelConfig,
    filter_text: &str,
    is_whitelist: bool,
    content: &str,
) -> bool {
    let filter_text = filter_text.trim();
    if filter_text.is_empty() {
        return true;
    }

    match evaluate_with_llm(config, filter_text, is_whitelist, content).await {
        Ok(pass) => pass,
        Err(e) => {
            warn!("seer-intel: LLM filter failed, falling back to pattern match: {e:#}");
            pattern_match(filter_text, is_whitelist, content)
        }
    }
}

async fn evaluate_with_llm(
    config: &SeerIntelConfig,
    filter_text: &str,
    is_whitelist: bool,
    content: &str,
) -> anyhow::Result<bool> {
    let api_key = api_key_from_prefs_or_env(&config.api_key);
    if api_key.is_empty() {
        anyhow::bail!("missing Gemini API key (Intel preferences or GOOGLE_API_KEY)");
    }

    let model = config
        .summary_model
        .trim()
        .strip_prefix("models/")
        .unwrap_or(config.summary_model.trim());
    if model.is_empty() {
        anyhow::bail!("filter model is empty");
    }

    let mode = if is_whitelist {
        "whitelist — save content that matches the criteria"
    } else {
        "blacklist — discard content that matches the criteria"
    };
    let clipped = truncate_chars(content, FILTER_CONTENT_MAX_CHARS);
    let user_prompt = format!(
        "Filter mode: {mode}\n\nCriteria:\n{filter_text}\n\nContent:\n{clipped}"
    );

    let schema = json!({
        "type": "object",
        "properties": {
            "pass": {
                "type": "boolean",
                "description": "True if the content should be saved."
            },
            "reason": {
                "type": "string",
                "description": "One short sentence explaining the decision."
            }
        },
        "required": ["pass", "reason"]
    });

    let client = GenaiClient::new(api_key, model.to_string());
    let raw = client
        .generate_json(
            FILTER_SYSTEM_PROMPT,
            &user_prompt,
            schema,
            FILTER_MAX_OUTPUT_TOKENS,
        )
        .await?;
    let out = parse_filter_json(&raw)?;
    debug!(
        pass = out.pass,
        reason = %out.reason,
        "seer-intel: LLM filter decision"
    );
    Ok(out.pass)
}

fn parse_filter_json(raw: &str) -> anyhow::Result<FilterLlmOut> {
    let trimmed = raw.trim();
    if let Ok(out) = serde_json::from_str::<FilterLlmOut>(trimmed) {
        return Ok(out);
    }
    let candidate = strip_json_fence(trimmed);
    if let Ok(out) = serde_json::from_str::<FilterLlmOut>(candidate) {
        return Ok(out);
    }
    if let Some(obj) = extract_first_json_object(candidate) {
        if let Ok(out) = serde_json::from_str::<FilterLlmOut>(obj) {
            return Ok(out);
        }
    }
    let preview: String = trimmed.chars().take(120).collect();
    anyhow::bail!("filter json: expected object, got {preview:?}")
}

fn strip_json_fence(s: &str) -> &str {
    let s = s.trim();
    let Some(rest) = s.strip_prefix("```") else {
        return s;
    };
    let rest = rest
        .strip_prefix("json")
        .or_else(|| rest.strip_prefix("JSON"))
        .unwrap_or(rest)
        .trim_start_matches(['\r', '\n']);
    rest.strip_suffix("```").unwrap_or(rest).trim()
}

fn extract_first_json_object(s: &str) -> Option<&str> {
    let start = s.find('{')?;
    let end = s.rfind('}')?;
    if end < start {
        return None;
    }
    Some(&s[start..=end])
}

fn truncate_chars(s: &str, max: usize) -> &str {
    if s.chars().count() <= max {
        return s;
    }
    let end = s.char_indices().nth(max).map(|(i, _)| i).unwrap_or(s.len());
    &s[..end]
}

/// Fast substring fallback when the LLM is unavailable.
fn pattern_match(filter_text: &str, is_whitelist: bool, content: &str) -> bool {
    let hay = content.to_lowercase();
    let patterns: Vec<_> = filter_text
        .lines()
        .map(|l| l.trim().to_lowercase())
        .filter(|l| !l.is_empty())
        .collect();
    let matched = patterns.iter().any(|p| hay.contains(p));
    if is_whitelist {
        matched
    } else {
        !matched
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_filter_passes() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = SeerIntelConfig::default();
        let pass = rt.block_on(passes_content_filter(&config, "", false, "anything"));
        assert!(pass);
    }

    #[test]
    fn pattern_blacklist_blocks_match() {
        assert!(!pattern_match("spam\nnoise", false, "This is spam content"));
    }

    #[test]
    fn pattern_whitelist_requires_match() {
        assert!(pattern_match("ukraine", true, "News from Ukraine"));
        assert!(!pattern_match("ukraine", true, "News from Poland"));
    }

    #[test]
    fn parse_filter_json_accepts_fenced() {
        let out = parse_filter_json(
            "```json\n{\"pass\":true,\"reason\":\"matches criteria\"}\n```",
        )
        .unwrap();
        assert!(out.pass);
        assert_eq!(out.reason, "matches criteria");
    }
}
