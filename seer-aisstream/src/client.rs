use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use serde_json::{json, Value};
use std::time::Duration;
use tracing::{info, warn};

use crate::{
    config::SeerAisstreamConfig, entities::position_report::ActiveModel as AM,
    preferences::resolved_config,
};

pub fn start_client(db: DatabaseConnection) {
    tokio::spawn(async move {
        loop {
            let cfg = match resolved_config(&db).await {
                Ok(c) => c,
                Err(e) => {
                    warn!("aisstream prefs: {e}");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };
            if !cfg.enabled || cfg.api_key.is_empty() {
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
            if let Err(e) = run_session(&db, &cfg).await {
                warn!("aisstream session: {e:#}");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
}

async fn run_session(db: &DatabaseConnection, cfg: &SeerAisstreamConfig) -> anyhow::Result<()> {
    let (ws, _) = tokio_tungstenite::connect_async("wss://stream.aisstream.io/v0/stream").await?;
    let (mut sink, mut stream) = ws.split();
    let sub = json!({
        "APIKey": cfg.api_key,
        "BoundingBoxes": [[[-90.0, -180.0], [90.0, 180.0]]],
        "FilterMessageTypes": ["PositionReport"]
    });
    sink.send(tokio_tungstenite::tungstenite::Message::Text(sub.to_string().into()))
        .await?;
    info!("aisstream: connected");
    let mut saved = 0u64;
    while let Some(msg) = stream.next().await {
        let msg = msg?;
        let text = match msg {
            tokio_tungstenite::tungstenite::Message::Text(t) => t.to_string(),
            tokio_tungstenite::tungstenite::Message::Binary(b) => {
                String::from_utf8_lossy(&b).to_string()
            }
            tokio_tungstenite::tungstenite::Message::Ping(p) => {
                sink.send(tokio_tungstenite::tungstenite::Message::Pong(p))
                    .await?;
                continue;
            }
            tokio_tungstenite::tungstenite::Message::Close(_) => break,
            _ => continue,
        };
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            if let Some(report) = v.get("Message").and_then(|m| m.get("PositionReport")) {
                let mmsi = report.get("UserID").and_then(|x| x.as_i64()).unwrap_or(0);
                let lat = report.get("Latitude").and_then(|x| x.as_f64());
                let lng = report.get("Longitude").and_then(|x| x.as_f64());
                let meta = v.get("MetaData").cloned().unwrap_or(Value::Null);
                let ship = meta
                    .get("ShipName")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let now = Utc::now();
                if let Err(e) = (AM {
                    mmsi: Set(mmsi),
                    latitude: Set(lat),
                    longitude: Set(lng),
                    sog: Set(report.get("Sog").and_then(|x| x.as_f64())),
                    cog: Set(report.get("Cog").and_then(|x| x.as_f64())),
                    heading: Set(report.get("TrueHeading").and_then(|x| x.as_f64())),
                    ship_name: Set(ship),
                    created_at: Set(Some(now)),
                    updated_at: Set(Some(now)),
                    ..Default::default()
                })
                .insert(db)
                .await
                {
                    warn!(error = %e, mmsi, "failed to insert aisstream position report");
                } else {
                    saved += 1;
                    if saved % 100 == 0 {
                        info!("aisstream: saved {saved} reports");
                    }
                }
            }
        }
    }
    Ok(())
}
