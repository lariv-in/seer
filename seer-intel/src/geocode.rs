//! Google Geocoding API helper for intel events.

use serde::Deserialize;

use crate::config::SeerIntelConfig;

#[derive(Debug, Deserialize)]
struct GeocodeResponse {
    results: Vec<GeocodeResult>,
    status: String,
}

#[derive(Debug, Deserialize)]
struct GeocodeResult {
    geometry: GeocodeGeometry,
}

#[derive(Debug, Deserialize)]
struct GeocodeGeometry {
    location: LatLng,
}

#[derive(Debug, Deserialize)]
struct LatLng {
    lat: f64,
    lng: f64,
}

/// Geocode an address string. Returns None if key missing, address empty, or API fails.
pub async fn geocode_address(config: &SeerIntelConfig, address: &str) -> Option<(f64, f64)> {
    let key = config.geocoding_api_key.trim();
    let address = address.trim();
    if key.is_empty() || address.is_empty() {
        return None;
    }
    let url = format!(
        "https://maps.googleapis.com/maps/api/geocode/json?address={}&key={}",
        urlencoding_encode(address),
        key
    );
    let client = reqwest::Client::new();
    let resp = match client.get(&url).send().await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "geocode request failed");
            return None;
        }
    };
    let body: GeocodeResponse = match resp.json().await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "geocode response json failed");
            return None;
        }
    };
    if body.status != "OK" {
        return None;
    }
    let loc = &body.results.first()?.geometry.location;
    Some((loc.lat, loc.lng))
}

fn urlencoding_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
