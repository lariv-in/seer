//! Rune sandbox bindings for website crawl sources and workers.

use std::sync::Arc;

use chrono::Utc;
use lariv_rs::rune_env::{
    block_on_async, json_to_rune, rune_to_json, NativeBinding, RuneEnvCapability, RuneEnvCtx,
    RuneEnvRegistrar,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, EntityTrait, QueryOrder, QuerySelect,
};
use seer_common::parse_duration_secs;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};

use crate::entities::{
    website_runner::{self, ActiveModel as WebsiteRunnerAM, Entity as WebsiteRunnerEntity},
    website_source::{self, ActiveModel as WebsiteSourceAM, Entity as WebsiteSourceEntity},
};

/// Registers website monitoring helpers onto the assistant Rune environment.
#[derive(Clone, Copy, Default)]
pub struct Hook;

impl RuneEnvRegistrar for Hook {
    fn register_rune_env(self, rune_env: &mut RuneEnvCapability) {
        register(rune_env);
    }
}

fn register(rune_env: &mut RuneEnvCapability) {
    rune_env.register_contextual(
        "website_list_sources",
        "website_list_sources(#{ limit?: int }?) -> #{ sources: [#{ id, url, depth, website_runner_id?, filter, is_filter_whitelist }] }",
        |_ctx| NativeBinding::Function(Arc::new(website_list_sources)),
    );
    rune_env.register_contextual(
        "website_list_workers",
        "website_list_workers() -> #{ workers: [#{ id, name, duration_secs }] }",
        |_ctx| NativeBinding::Function(Arc::new(website_list_workers)),
    );
    rune_env.register_contextual(
        "website_add_source",
        "website_add_source(#{ seed_url: string, website_depth?: int, website_runner_id?: int, filter?: string, is_filter_whitelist?: bool }) -> #{ website_source_id }",
        |_ctx| NativeBinding::Function(Arc::new(website_add_source)),
    );
    rune_env.register_contextual(
        "website_edit_source",
        "website_edit_source(#{ website_source_id: int, seed_url?: string, website_depth?: int, website_runner_id?: int, filter?: string, is_filter_whitelist?: bool }) -> #{ website_source_id }",
        |_ctx| NativeBinding::Function(Arc::new(website_edit_source)),
    );
    rune_env.register_contextual(
        "website_add_worker",
        "website_add_worker(#{ worker_name: string, worker_duration?: string }) -> #{ website_runner_id }",
        |_ctx| NativeBinding::Function(Arc::new(website_add_worker)),
    );
    rune_env.register_contextual(
        "website_edit_worker",
        "website_edit_worker(#{ website_runner_id: int, worker_name?: string, worker_duration?: string }) -> #{ website_runner_id }",
        |_ctx| NativeBinding::Function(Arc::new(website_edit_worker)),
    );
}

fn parse_object_arg(name: &str, args: &[rune::Value]) -> Result<JsonValue, String> {
    let value = args
        .first()
        .ok_or_else(|| format!("{name} requires an object argument"))?;
    rune_to_json(value).map_err(|e| format!("invalid {name} arguments: {e}"))
}

fn parse_optional_object_arg(name: &str, args: &[rune::Value]) -> Result<JsonValue, String> {
    match args.first() {
        None => Ok(json!({})),
        Some(value) => rune_to_json(value).map_err(|e| format!("invalid {name} arguments: {e}")),
    }
}

fn website_list_sources(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize, Default)]
    struct Args {
        #[serde(default)]
        limit: Option<u64>,
    }

    let parsed: Args =
        serde_json::from_value(parse_optional_object_arg("website_list_sources", args)?)
            .map_err(|e| format!("invalid website_list_sources arguments: {e}"))?;
    let limit = parsed.limit.unwrap_or(50).min(200);
    let db = ctx.db.clone();
    let rows = block_on_async(async move {
        WebsiteSourceEntity::find()
            .order_by_desc(website_source::Column::Id)
            .limit(limit)
            .all(&db)
            .await
    })
    .map_err(|e| e.to_string())?;
    json_to_rune(json!({
        "sources": rows.into_iter().map(|s| json!({
            "id": s.id,
            "url": s.url,
            "depth": s.depth,
            "website_runner_id": s.website_runner_id,
            "filter": s.filter,
            "is_filter_whitelist": s.is_filter_whitelist,
        })).collect::<Vec<_>>(),
    }))
}

fn website_list_workers(
    ctx: &RuneEnvCtx<'_>,
    _args: &[rune::Value],
) -> Result<rune::Value, String> {
    let db = ctx.db.clone();
    let rows = block_on_async(async move {
        WebsiteRunnerEntity::find()
            .order_by_asc(website_runner::Column::Name)
            .all(&db)
            .await
    })
    .map_err(|e| e.to_string())?;
    json_to_rune(json!({
        "workers": rows.into_iter().map(|r| json!({
            "id": r.id,
            "name": r.name,
            "duration_secs": r.duration_secs,
        })).collect::<Vec<_>>(),
    }))
}

fn website_add_source(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize, Default)]
    struct Args {
        seed_url: Option<String>,
        #[serde(default)]
        website_depth: Option<i64>,
        #[serde(default)]
        website_runner_id: Option<i64>,
        #[serde(default)]
        filter: Option<String>,
        #[serde(default)]
        is_filter_whitelist: Option<bool>,
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("website_add_source", args)?)
        .map_err(|e| format!("invalid website_add_source arguments: {e}"))?;
    let url = parsed
        .seed_url
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_string();
    if url.is_empty() {
        return Err("seed_url required".into());
    }
    let now = Utc::now();
    let db = ctx.db.clone();
    let row = block_on_async(async move {
        WebsiteSourceAM {
            url: Set(url),
            depth: Set(parsed.website_depth.unwrap_or(1)),
            website_runner_id: Set(parsed.website_runner_id),
            filter: Set(parsed.filter.unwrap_or_default()),
            is_filter_whitelist: Set(parsed.is_filter_whitelist.unwrap_or(false)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&db)
        .await
    })
    .map_err(|e| e.to_string())?;
    json_to_rune(json!({"website_source_id": row.id}))
}

fn website_edit_source(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize)]
    struct Args {
        website_source_id: i64,
        #[serde(default)]
        seed_url: Option<String>,
        #[serde(default)]
        website_depth: Option<i64>,
        #[serde(default)]
        website_runner_id: Option<i64>,
        #[serde(default)]
        filter: Option<String>,
        #[serde(default)]
        is_filter_whitelist: Option<bool>,
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("website_edit_source", args)?)
        .map_err(|e| format!("invalid website_edit_source arguments: {e}"))?;
    let id = parsed.website_source_id;
    let db = ctx.db.clone();
    let row = block_on_async(async move {
        let existing = WebsiteSourceEntity::find_by_id(id)
            .one(&db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "source not found".to_string())?;
        let mut am: WebsiteSourceAM = existing.into();
        if let Some(v) = parsed
            .seed_url
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            am.url = Set(v.into());
        }
        if let Some(v) = parsed.website_depth {
            am.depth = Set(v);
        }
        if let Some(v) = parsed.website_runner_id {
            am.website_runner_id = Set(Some(v));
        }
        if let Some(v) = parsed.filter {
            am.filter = Set(v);
        }
        if let Some(v) = parsed.is_filter_whitelist {
            am.is_filter_whitelist = Set(v);
        }
        am.updated_at = Set(Some(Utc::now()));
        am.update(&db).await.map_err(|e| e.to_string())
    })?;
    json_to_rune(json!({"website_source_id": row.id}))
}

fn website_add_worker(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize)]
    struct Args {
        worker_name: String,
        #[serde(default)]
        worker_duration: Option<String>,
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("website_add_worker", args)?)
        .map_err(|e| format!("invalid website_add_worker arguments: {e}"))?;
    let name = parsed.worker_name.trim().to_string();
    if name.is_empty() {
        return Err("worker_name required".into());
    }
    let dur = parse_duration_secs(parsed.worker_duration.as_deref().unwrap_or("5m"))?;
    let now = Utc::now();
    let db = ctx.db.clone();
    let row = block_on_async(async move {
        WebsiteRunnerAM {
            name: Set(name),
            duration_secs: Set(dur),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&db)
        .await
    })
    .map_err(|e| e.to_string())?;
    json_to_rune(json!({"website_runner_id": row.id}))
}

fn website_edit_worker(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize)]
    struct Args {
        website_runner_id: i64,
        #[serde(default)]
        worker_name: Option<String>,
        #[serde(default)]
        worker_duration: Option<String>,
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("website_edit_worker", args)?)
        .map_err(|e| format!("invalid website_edit_worker arguments: {e}"))?;
    let id = parsed.website_runner_id;
    let db = ctx.db.clone();
    let row = block_on_async(async move {
        let existing = WebsiteRunnerEntity::find_by_id(id)
            .one(&db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "runner not found".to_string())?;
        let mut am: WebsiteRunnerAM = existing.into();
        if let Some(v) = parsed
            .worker_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            am.name = Set(v.into());
        }
        if let Some(v) = parsed.worker_duration.as_deref() {
            am.duration_secs = Set(parse_duration_secs(v)?);
        }
        am.updated_at = Set(Some(Utc::now()));
        am.update(&db).await.map_err(|e| e.to_string())
    })?;
    json_to_rune(json!({"website_runner_id": row.id}))
}
