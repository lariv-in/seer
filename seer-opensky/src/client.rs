use parking_lot::Mutex;
use serde::Deserialize;
use std::sync::OnceLock;
use crate::config::SeerOpenskyConfig;

#[derive(Clone)]
struct TokenCache {
    access_token: String,
    expires_at: std::time::Instant,
}

impl Default for TokenCache {
    fn default() -> Self {
        Self {
            access_token: String::new(),
            expires_at: std::time::Instant::now(),
        }
    }
}

fn token_cache() -> &'static Mutex<TokenCache> {
    static C: OnceLock<Mutex<TokenCache>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(TokenCache { access_token: String::new(), expires_at: std::time::Instant::now() }))
}

pub async fn oauth_token(cfg: &SeerOpenskyConfig) -> anyhow::Result<String> {
    {
        let c = token_cache().lock();
        if !c.access_token.is_empty() && c.expires_at > std::time::Instant::now() {
            return Ok(c.access_token.clone());
        }
    }
    if cfg.client_id.is_empty() || cfg.client_secret.is_empty() {
        anyhow::bail!("opensky clientId/secret empty");
    }
    #[derive(Deserialize)]
    struct Tok { access_token: String, #[serde(default)] expires_in: u64 }
    let client = reqwest::Client::new();
    let resp = client
        .post("https://auth.opensky-network.org/auth/realms/opensky-network/protocol/openid-connect/token")
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", cfg.client_id.as_str()),
            ("client_secret", cfg.client_secret.as_str()),
        ])
        .send().await?.error_for_status()?;
    let tok: Tok = resp.json().await?;
    let mut c = token_cache().lock();
    c.access_token = tok.access_token.clone();
    c.expires_at = std::time::Instant::now() + std::time::Duration::from_secs(tok.expires_in.saturating_sub(30).max(60));
    Ok(tok.access_token)
}

#[derive(Debug, Clone)]
pub struct StateVector {
    pub icao24: String, pub callsign: Option<String>, pub origin_country: Option<String>,
    pub time_position: Option<i64>, pub last_contact: i64,
    pub longitude: Option<f64>, pub latitude: Option<f64>,
    pub baro_altitude: Option<f64>, pub on_ground: Option<bool>,
    pub velocity: Option<f64>, pub true_track: Option<f64>, pub vertical_rate: Option<f64>,
    pub geo_altitude: Option<f64>, pub squawk: Option<String>,
}

#[derive(Deserialize)]
struct StatesResp {
    time: i64,
    #[serde(default)]
    states: Option<Vec<Vec<serde_json::Value>>>,
}

pub async fn fetch_states(cfg: &SeerOpenskyConfig) -> anyhow::Result<(i64, Vec<StateVector>)> {
    let token = oauth_token(cfg).await?;
    let client = reqwest::Client::new();
    let resp = client
        .get("https://opensky-network.org/api/states/all")
        .bearer_auth(token)
        .send().await?.error_for_status()?;
    let parsed: StatesResp = resp.json().await?;
    let mut out = Vec::new();
    for row in parsed.states.unwrap_or_default() {
        let get_str = |i: usize| row.get(i).and_then(|v| v.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        let get_f = |i: usize| row.get(i).and_then(|v| v.as_f64());
        let get_i = |i: usize| row.get(i).and_then(|v| v.as_i64()).or_else(|| get_f(i).map(|f| f as i64));
        let get_b = |i: usize| row.get(i).and_then(|v| v.as_bool());
        let Some(icao) = get_str(0) else { continue };
        let last = get_i(4).unwrap_or(0);
        out.push(StateVector {
            icao24: icao, callsign: get_str(1), origin_country: get_str(2),
            time_position: get_i(3), last_contact: last,
            longitude: get_f(5), latitude: get_f(6),
            baro_altitude: get_f(7), on_ground: get_b(8),
            velocity: get_f(9), true_track: get_f(10), vertical_rate: get_f(11),
            geo_altitude: get_f(13), squawk: get_str(14),
        });
    }
    Ok((parsed.time, out))
}
