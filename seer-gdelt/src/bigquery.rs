//! BigQuery REST client (Application Default Credentials / access token).

use serde_json::{json, Value};
use tracing::warn;
use crate::config::SeerGdeltConfig;

pub async fn fetch_gdelt_rows(cfg: &SeerGdeltConfig, sql: &str) -> anyhow::Result<Vec<Value>> {
    if cfg.project_id.trim().is_empty() {
        anyhow::bail!("GDELT projectId is empty (Preferences)");
    }
    let token = access_token().await?;
    let url = format!(
        "https://bigquery.googleapis.com/bigquery/v2/projects/{}/queries",
        cfg.project_id
    );
    let body = json!({ "query": sql, "useLegacySql": false, "maxResults": 100 });
    let client = reqwest::Client::new();
    let resp = client.post(&url).bearer_auth(token).json(&body).send().await?;
    if !resp.status().is_success() {
        let t = resp.text().await.unwrap_or_default();
        anyhow::bail!("bigquery: {t}");
    }
    let v: Value = resp.json().await?;
    let rows = v.get("rows").and_then(|r| r.as_array()).cloned().unwrap_or_default();
    Ok(rows)
}

async fn access_token() -> anyhow::Result<String> {
    // Prefer GOOGLE_OAUTH_ACCESS_TOKEN for local/dev; otherwise try metadata server.
    if let Ok(t) = std::env::var("GOOGLE_OAUTH_ACCESS_TOKEN") {
        if !t.is_empty() { return Ok(t); }
    }
    let url = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";
    let client = reqwest::Client::new();
    match client.get(url).header("Metadata-Flavor", "Google").send().await {
        Ok(r) if r.status().is_success() => {
            let v: Value = r.json().await?;
            Ok(v["access_token"].as_str().unwrap_or("").to_string())
        }
        Ok(r) => {
            warn!("metadata token status {}", r.status());
            anyhow::bail!("no ADC access token available")
        }
        Err(e) => anyhow::bail!("metadata token: {e}"),
    }
}

pub fn default_gdelt_sql(query: &str, max_records: i64) -> String {
    let limit = if max_records > 0 { max_records } else { 50 };
    let q = query.replace('\'', "\\'");
    format!(
        "SELECT GLOBALEVENTID, SOURCEURL, Actor1Name, Actor2Name, ActionGeo_Lat, ActionGeo_Long, SQLDATE \
         FROM `gdelt-bq.gdeltv2.events` WHERE SOURCEURL IS NOT NULL AND LOWER(SOURCEURL) LIKE '%{q}%' \
         LIMIT {limit}"
    )
}
