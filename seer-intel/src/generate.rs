//! LLM title/summary/event generation for intel ingest (parity with Go `NewFromIntelKind`).

use chrono::{DateTime, Utc};
use lariv_rs::genai::GenaiClient;
use serde::Deserialize;
use serde_json::json;

use crate::config::SeerIntelConfig;
use crate::preferences::api_key_from_prefs_or_env;

const INTEL_SUMMARY_SYSTEM_PROMPT: &str = "\
You write concise factual summaries for an intelligence ingest pipeline.
Given raw source content, respond with a short plain-text summary only (no markdown headings, no preamble).
Aim for 2–6 sentences. If the content is empty or unusable, reply with a single sentence stating that.";

const INTEL_TITLE_SYSTEM_PROMPT: &str = "\
You label rows in an intelligence ingest pipeline.
Given raw source content, respond with one short plain-text title only: no markdown, no preamble, no quotation marks.
At most 12 words. Describe the subject (what it is about), not the medium (avoid \"post\", \"article\", \"document\" unless necessary).";

const INTEL_EVENT_EXTRACT_SYSTEM_PROMPT: &str = "\
You extract a geographic location and an event time from an intelligence summary.
Respond ONLY with a JSON object (no markdown, no commentary) with exactly these keys:
\"address\": string — a concise postal-style or place name string suitable for a geocoder (empty if none).
\"datetime\": string — RFC3339 timestamp in UTC (e.g. \"2006-01-02T15:04:05Z\").
If the summary implies a date but not a time, use 00:00:00Z for that date.
If no date can be inferred, use the current UTC time in RFC3339.";

const INTEL_TITLE_MAX_RUNES: usize = 200;
/// Leave headroom for thinking models that spend tokens on reasoning parts.
const INTEL_EVENT_EXTRACT_MAX_TOKENS: i32 = 1024;

#[derive(Debug, Deserialize)]
struct IntelEventLlmOut {
    #[serde(default)]
    address: String,
    datetime: String,
}

/// Address + inferred event time extracted from a summary (parity with Go `extractIntelEventFromSummary`).
#[derive(Debug, Clone)]
pub struct ExtractedEvent {
    pub address: String,
    pub datetime: DateTime<Utc>,
}

/// Generate a short title from raw source content using `config.title_model`.
pub async fn generate_title(config: &SeerIntelConfig, content: &str) -> anyhow::Result<String> {
    let raw = generate_with_model(config, &config.title_model, INTEL_TITLE_SYSTEM_PROMPT, content)
        .await?;
    let title = normalize_intel_title(&raw);
    if title.is_empty() {
        Ok(intel_title_fallback(content))
    } else {
        Ok(title)
    }
}

/// Generate a plain-text summary from raw source content using `config.summary_model`.
pub async fn generate_summary(config: &SeerIntelConfig, content: &str) -> anyhow::Result<String> {
    let raw = generate_with_model(
        config,
        &config.summary_model,
        INTEL_SUMMARY_SYSTEM_PROMPT,
        content,
    )
    .await?;
    Ok(raw.trim().to_string())
}

/// Extract a geocodeable address and event datetime from a summary using `config.summary_model`.
pub async fn extract_event_from_summary(
    config: &SeerIntelConfig,
    summary: &str,
) -> anyhow::Result<ExtractedEvent> {
    let summary = summary.trim();
    if summary.is_empty() {
        anyhow::bail!("extract intel event: empty summary");
    }
    let user_prompt = format!(
        "Current UTC time: {}\n\nSummary:\n{}",
        Utc::now().to_rfc3339(),
        summary
    );
    // OpenAPI-subset schema for Gemini `responseSchema` (lowercase types are accepted).
    let schema = json!({
        "type": "object",
        "properties": {
            "address": {
                "type": "string",
                "description": "Concise postal-style or place name for geocoding; empty if none."
            },
            "datetime": {
                "type": "string",
                "description": "Event time in RFC3339, UTC (e.g. 2006-01-02T15:04:05Z)."
            }
        },
        "required": ["address", "datetime"]
    });
    let raw = generate_json_with_model(
        config,
        &config.summary_model,
        INTEL_EVENT_EXTRACT_SYSTEM_PROMPT,
        &user_prompt,
        schema,
        INTEL_EVENT_EXTRACT_MAX_TOKENS,
    )
    .await?;
    let out: IntelEventLlmOut = parse_event_json(&raw)?;
    let datetime = DateTime::parse_from_rfc3339(out.datetime.trim())
        .map_err(|e| anyhow::anyhow!("event extract datetime: {e}"))?
        .with_timezone(&Utc);
    Ok(ExtractedEvent {
        address: out.address.trim().to_string(),
        datetime,
    })
}

/// Parse model JSON, tolerating markdown fences or leading/trailing prose.
fn parse_event_json(raw: &str) -> anyhow::Result<IntelEventLlmOut> {
    let trimmed = raw.trim();
    if let Ok(out) = serde_json::from_str::<IntelEventLlmOut>(trimmed) {
        return Ok(out);
    }
    let candidate = strip_json_fence(trimmed);
    if let Ok(out) = serde_json::from_str::<IntelEventLlmOut>(candidate) {
        return Ok(out);
    }
    if let Some(obj) = extract_first_json_object(candidate) {
        if let Ok(out) = serde_json::from_str::<IntelEventLlmOut>(obj) {
            return Ok(out);
        }
    }
    let preview: String = trimmed.chars().take(120).collect();
    anyhow::bail!("event extract json: expected object, got {preview:?}")
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

async fn generate_with_model(
    config: &SeerIntelConfig,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> anyhow::Result<String> {
    let client = genai_client(config, model)?;
    Ok(client.generate_text(system_prompt, user_prompt).await?)
}

async fn generate_json_with_model(
    config: &SeerIntelConfig,
    model: &str,
    system_prompt: &str,
    user_prompt: &str,
    schema: serde_json::Value,
    max_output_tokens: i32,
) -> anyhow::Result<String> {
    let client = genai_client(config, model)?;
    Ok(client
        .generate_json(system_prompt, user_prompt, schema, max_output_tokens)
        .await?)
}

fn genai_client(config: &SeerIntelConfig, model: &str) -> anyhow::Result<GenaiClient> {
    let api_key = api_key_from_prefs_or_env(&config.api_key);
    if api_key.is_empty() {
        anyhow::bail!("missing Gemini API key (Intel preferences or GOOGLE_API_KEY)");
    }
    let model = model.trim();
    if model.is_empty() {
        anyhow::bail!("generate model is empty");
    }
    let model = model.strip_prefix("models/").unwrap_or(model);
    Ok(GenaiClient::new(api_key, model.to_string()))
}

/// Cleans model output to a single-line DB title.
pub fn normalize_intel_title(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    if s.is_empty() {
        return String::new();
    }
    if let Some(i) = s.find(['\r', '\n']) {
        s = s[..i].trim().to_string();
    }
    s = s.trim_matches(|c| c == '"' || c == '\'').to_string();
    let count = s.chars().count();
    if count <= INTEL_TITLE_MAX_RUNES {
        return s;
    }
    s.chars().take(INTEL_TITLE_MAX_RUNES).collect::<String>().trim().to_string()
}

fn intel_title_fallback(content: &str) -> String {
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("---") {
            continue;
        }
        let t = normalize_intel_title(t);
        if !t.is_empty() {
            return t;
        }
    }
    "Intel".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_quotes_and_newlines() {
        assert_eq!(
            normalize_intel_title("  \"Hello world\"\nmore "),
            "Hello world"
        );
    }

    #[test]
    fn title_fallback_uses_first_nonempty_line() {
        assert_eq!(
            intel_title_fallback("\n---\nr/news: Something happened\n\nbody"),
            "r/news: Something happened"
        );
    }

    #[test]
    fn parse_event_json_accepts_fenced_and_prose() {
        let out = parse_event_json(
            "Here you go:\n```json\n{\"address\":\"Kyiv, Ukraine\",\"datetime\":\"2026-08-26T00:00:00Z\"}\n```\n",
        )
        .unwrap();
        assert_eq!(out.address, "Kyiv, Ukraine");
        assert_eq!(out.datetime, "2026-08-26T00:00:00Z");
    }

    #[test]
    fn parse_event_json_rejects_empty() {
        assert!(parse_event_json("").is_err());
        assert!(parse_event_json("   ").is_err());
    }
}
