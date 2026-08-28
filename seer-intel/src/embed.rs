//! Google GenAI embedContent via reqwest.

use crate::config::SeerIntelConfig;
use serde::Deserialize;
use serde_json::json;

pub const SEER_INTEL_EMBEDDING_DIM: usize = 3072;

pub async fn embed_query_text(
    config: &SeerIntelConfig,
    text: &str,
) -> anyhow::Result<Vec<f32>> {
    let text = text.trim();
    if text.is_empty() {
        anyhow::bail!("empty text");
    }
    let api_key = resolve_api_key(config);
    if api_key.is_empty() {
        anyhow::bail!("missing Gemini API key (Intel preferences or GOOGLE_API_KEY)");
    }
    let model = config.embedding_model.trim();
    if model.is_empty() {
        anyhow::bail!("embeddingModel is empty");
    }
    let model_path = if model.starts_with("models/") {
        model.to_string()
    } else {
        format!("models/{model}")
    };
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/{model_path}:embedContent?key={api_key}"
    );
    let body = json!({
        "content": { "parts": [{ "text": text }] }
    });
    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let t = resp.text().await.unwrap_or_default();
        anyhow::bail!("embed HTTP {status}: {t}");
    }
    #[derive(Deserialize)]
    struct Emb {
        values: Vec<f32>,
    }
    #[derive(Deserialize)]
    struct Resp {
        embedding: Emb,
    }
    let parsed: Resp = resp.json().await?;
    Ok(parsed.embedding.values)
}

fn resolve_api_key(config: &SeerIntelConfig) -> String {
    if !config.api_key.trim().is_empty() {
        return config.api_key.trim().to_string();
    }
    std::env::var("GOOGLE_API_KEY")
        .or_else(|_| std::env::var("GEMINI_API_KEY"))
        .unwrap_or_default()
}

pub fn vector_to_pg_literal(values: &[f32]) -> String {
    let inner = values
        .iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{inner}]")
}
