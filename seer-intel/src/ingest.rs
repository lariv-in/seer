//! Async intel ingest channel + worker.

use chrono::Utc;
use parking_lot::Mutex;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use std::sync::OnceLock;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::{
    embed, generate,
    entities::{
        intel::{self, ActiveModel as IntelAM, Entity as IntelEntity},
        intel_event::{ActiveModel as EventAM, Entity as EventEntity},
    },
    kind::DynIntelKind,
    preferences,
};

pub struct IngestRequest {
    pub kind: DynIntelKind,
}

static GLOBAL_TX: OnceLock<Mutex<Option<mpsc::Sender<IngestRequest>>>> = OnceLock::new();
static WORKER_STARTED: OnceLock<()> = OnceLock::new();

fn global_slot() -> &'static Mutex<Option<mpsc::Sender<IngestRequest>>> {
    GLOBAL_TX.get_or_init(|| Mutex::new(None))
}

pub fn enqueue_intel(kind: DynIntelKind) {
    if let Some(tx) = global_slot().lock().as_ref() {
        if let Err(e) = tx.try_send(IngestRequest { kind }) {
            warn!(error = %e, "seer-intel: ingest try_send failed");
        }
    } else {
        warn!("seer-intel: ingest channel not ready");
    }
}

pub fn start_intel_ingest_worker(db: DatabaseConnection) {
    if WORKER_STARTED.set(()).is_err() {
        return;
    }
    let (tx, mut rx) = mpsc::channel::<IngestRequest>(256);
    *global_slot().lock() = Some(tx);
    tokio::spawn(async move {
        info!("seer-intel: ingest worker started");
        while let Some(req) = rx.recv().await {
            if let Err(e) = process_ingest(&db, req.kind).await {
                error!("seer-intel: ingest failed: {e:#}");
            }
        }
    });
}

async fn process_ingest(db: &DatabaseConnection, kind: DynIntelKind) -> anyhow::Result<()> {
    let config = preferences::resolved_config(db).await?;
    let content = kind.content();
    if content.trim().is_empty() {
        return Ok(());
    }
    let k = kind.kind().to_string();
    let kid = kind.intel_id();

    let existing = IntelEntity::find()
        .filter(intel::Column::Kind.eq(&k))
        .filter(intel::Column::KindId.eq(kid))
        .one(db)
        .await?;

    // Preference models: title_model / summary_model / embedding_model.
    let title = generate::generate_title(&config, &content).await?;
    let summary = generate::generate_summary(&config, &content).await?;
    let now = Utc::now();

    let embedding = match embed::embed_query_text(&config, &content).await {
        Ok(v) => Some(embed::vector_to_pg_literal(&v)),
        Err(e) => {
            warn!("seer-intel: embed skipped: {e:#}");
            None
        }
    };

    let intel_id = if let Some(row) = existing {
        let mut am: IntelAM = row.clone().into();
        am.title = Set(title);
        am.summary = Set(summary.clone());
        am.datetime = Set(now);
        am.embedding = Set(embedding);
        am.updated_at = Set(Some(now));
        am.update(db).await?.id
    } else {
        IntelAM {
            title: Set(title),
            summary: Set(summary.clone()),
            datetime: Set(now),
            embedding: Set(embedding),
            kind: Set(k),
            kind_id: Set(kid),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(db)
        .await?
        .id
    };

    let ev = EventEntity::find()
        .filter(crate::entities::intel_event::Column::IntelId.eq(intel_id))
        .one(db)
        .await?;
    if ev.is_none() {
        match generate::extract_event_from_summary(&config, &summary).await {
            Ok(extracted) => {
                let (lat, lng) = if extracted.address.is_empty() {
                    (None, None)
                } else {
                    match crate::geocode::geocode_address(&config, &extracted.address).await {
                        Some((la, lo)) => (Some(la), Some(lo)),
                        None => (None, None),
                    }
                };
                EventAM {
                    intel_id: Set(intel_id),
                    address: Set(extracted.address),
                    datetime: Set(extracted.datetime),
                    latitude: Set(lat),
                    longitude: Set(lng),
                    created_at: Set(Some(now)),
                    updated_at: Set(Some(now)),
                    ..Default::default()
                }
                .insert(db)
                .await?;
            }
            Err(e) => {
                warn!("seer-intel: event extract failed: {e:#}");
            }
        }
    }
    let _ = &kind;
    Ok(())
}
