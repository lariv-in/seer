//! Rune sandbox bindings for Reddit ingestion sources and workers.

use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use lariv_rs::rune_env::{
    block_on_async, json_to_rune, rune_to_json, NativeBinding, RuneEnvCapability, RuneEnvCtx,
    RuneEnvRegistrar,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use seer_common::{is_active_worker, parse_duration_secs, RUNNER_KIND_REDDIT};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};

use crate::{
    entities::{
        reddit_runner::{self, ActiveModel as RedditRunnerAM, Entity as RedditRunnerEntity},
        reddit_source::{self, ActiveModel as RedditSourceAM, Entity as RedditSourceEntity},
    },
    handlers::{normalize_subreddit_names, workers::sync_runner_sources},
    workers::{start_runner_pool, stop_runner_pool},
};

/// Registers Reddit monitoring helpers onto the assistant Rune environment.
#[derive(Clone, Copy, Default)]
pub struct Hook;

impl RuneEnvRegistrar for Hook {
    fn register_rune_env(self, rune_env: &mut RuneEnvCapability) {
        register(rune_env);
    }
}

fn register(rune_env: &mut RuneEnvCapability) {
    rune_env.register_contextual(
        "reddit_list_sources",
        "reddit_list_sources(#{ limit?: int }?) -> #{ sources: [#{ id, subreddits, search_query, reddit_runner_id?, filter, is_filter_whitelist, max_fresh_posts, load_websites }] }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_list_sources)),
    );
    rune_env.register_contextual(
        "reddit_list_workers",
        "reddit_list_workers() -> #{ workers: [#{ id, name, duration_secs, source_ids }] }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_list_workers)),
    );
    rune_env.register_contextual(
        "reddit_add_source",
        "reddit_add_source(#{ reddit_runner_id?: int, subreddits?: [string], search_query?: string, filter?: string, is_filter_whitelist?: bool, max_fresh_posts?: int, load_websites?: bool }) -> #{ reddit_source_id }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_add_source)),
    );
    rune_env.register_contextual(
        "reddit_edit_source",
        "reddit_edit_source(#{ reddit_source_id: int, reddit_runner_id?: int, clear_reddit_runner_id?: bool, subreddits?: [string], search_query?: string, filter?: string, is_filter_whitelist?: bool, max_fresh_posts?: int, load_websites?: bool }) -> #{ reddit_source_id }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_edit_source)),
    );
    rune_env.register_contextual(
        "reddit_delete_source",
        "reddit_delete_source(#{ reddit_source_id: int }) -> #{ deleted: bool }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_delete_source)),
    );
    rune_env.register_contextual(
        "reddit_add_worker",
        "reddit_add_worker(#{ worker_name: string, worker_duration?: string, source_ids?: [int] }) -> #{ reddit_runner_id, duration_secs }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_add_worker)),
    );
    rune_env.register_contextual(
        "reddit_edit_worker",
        "reddit_edit_worker(#{ reddit_runner_id: int, worker_name?: string, worker_duration?: string, source_ids?: [int] }) -> #{ reddit_runner_id }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_edit_worker)),
    );
    rune_env.register_contextual(
        "reddit_delete_worker",
        "reddit_delete_worker(#{ reddit_runner_id: int }) -> #{ deleted: bool }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_delete_worker)),
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

async fn validate_runner_id(db: &DatabaseConnection, runner_id: i64) -> Result<(), String> {
    RedditRunnerEntity::find_by_id(runner_id)
        .one(db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("runner {runner_id} not found"))?;
    Ok(())
}

fn reddit_list_sources(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize, Default)]
    struct Args {
        #[serde(default)]
        limit: Option<u64>,
    }

    let parsed: Args =
        serde_json::from_value(parse_optional_object_arg("reddit_list_sources", args)?)
            .map_err(|e| format!("invalid reddit_list_sources arguments: {e}"))?;
    let limit = parsed.limit.unwrap_or(50).min(200);
    let db = ctx.db.clone();
    let rows = block_on_async(async move {
        RedditSourceEntity::find()
            .order_by_desc(reddit_source::Column::Id)
            .limit(limit)
            .all(&db)
            .await
    })
    .map_err(|e| e.to_string())?;
    json_to_rune(json!({
        "sources": rows.into_iter().map(|s| {
            let subreddits: Vec<String> =
                serde_json::from_value(s.subreddits.clone()).unwrap_or_default();
            json!({
                "id": s.id,
                "subreddits": subreddits,
                "search_query": s.search_query,
                "reddit_runner_id": s.reddit_runner_id,
                "filter": s.filter,
                "is_filter_whitelist": s.is_filter_whitelist,
                "max_fresh_posts": s.max_fresh_posts,
                "load_websites": s.load_websites,
            })
        }).collect::<Vec<_>>(),
    }))
}

fn reddit_list_workers(
    ctx: &RuneEnvCtx<'_>,
    _args: &[rune::Value],
) -> Result<rune::Value, String> {
    let db = ctx.db.clone();
    let rows = block_on_async(async move {
        let runners = RedditRunnerEntity::find()
            .order_by_asc(reddit_runner::Column::Name)
            .all(&db)
            .await?;
        let sources = RedditSourceEntity::find()
            .filter(reddit_source::Column::RedditRunnerId.is_not_null())
            .all(&db)
            .await?;
        let mut source_ids_by_runner: HashMap<i64, Vec<i64>> = HashMap::new();
        for s in sources {
            if let Some(runner_id) = s.reddit_runner_id {
                source_ids_by_runner
                    .entry(runner_id)
                    .or_default()
                    .push(s.id);
            }
        }
        Ok::<_, sea_orm::DbErr>((runners, source_ids_by_runner))
    })
    .map_err(|e| e.to_string())?;
    let (runners, source_ids_by_runner) = rows;
    json_to_rune(json!({
        "workers": runners.into_iter().map(|r| json!({
            "id": r.id,
            "name": r.name,
            "duration_secs": r.duration_secs,
            "source_ids": source_ids_by_runner.get(&r.id).cloned().unwrap_or_default(),
        })).collect::<Vec<_>>(),
    }))
}

fn reddit_add_source(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize, Default)]
    struct Args {
        #[serde(default)]
        reddit_runner_id: Option<i64>,
        #[serde(default)]
        subreddits: Vec<String>,
        #[serde(default)]
        search_query: Option<String>,
        #[serde(default)]
        filter: Option<String>,
        #[serde(default)]
        is_filter_whitelist: Option<bool>,
        #[serde(default)]
        max_fresh_posts: Option<i64>,
        #[serde(default)]
        load_websites: Option<bool>,
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("reddit_add_source", args)?)
        .map_err(|e| format!("invalid reddit_add_source arguments: {e}"))?;
    let subs = normalize_subreddit_names(&parsed.subreddits);
    if subs.is_empty() {
        return Err("at least one subreddit is required".into());
    }
    let runner_id = parsed.reddit_runner_id.filter(|&id| id > 0);
    let now = Utc::now();
    let db = ctx.db.clone();
    let row = block_on_async(async move {
        if let Some(id) = runner_id {
            validate_runner_id(&db, id).await?;
        }
        RedditSourceAM {
            reddit_runner_id: Set(runner_id),
            subreddits: Set(json!(subs)),
            search_query: Set(parsed.search_query.unwrap_or_default()),
            filter: Set(parsed.filter.unwrap_or_default()),
            is_filter_whitelist: Set(parsed.is_filter_whitelist.unwrap_or(false)),
            max_fresh_posts: Set(parsed.max_fresh_posts.unwrap_or(25).max(1)),
            load_websites: Set(parsed.load_websites.unwrap_or(false)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&db)
        .await
        .map_err(|e| e.to_string())
    })?;
    json_to_rune(json!({"reddit_source_id": row.id}))
}

fn reddit_edit_source(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize)]
    struct Args {
        reddit_source_id: i64,
        #[serde(default)]
        reddit_runner_id: Option<i64>,
        #[serde(default)]
        clear_reddit_runner_id: Option<bool>,
        #[serde(default)]
        subreddits: Option<Vec<String>>,
        #[serde(default)]
        search_query: Option<String>,
        #[serde(default)]
        filter: Option<String>,
        #[serde(default)]
        is_filter_whitelist: Option<bool>,
        #[serde(default)]
        max_fresh_posts: Option<i64>,
        #[serde(default)]
        load_websites: Option<bool>,
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("reddit_edit_source", args)?)
        .map_err(|e| format!("invalid reddit_edit_source arguments: {e}"))?;
    let id = parsed.reddit_source_id;
    let db = ctx.db.clone();
    let row = block_on_async(async move {
        let existing = RedditSourceEntity::find_by_id(id)
            .one(&db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "source not found".to_string())?;
        let mut am: RedditSourceAM = existing.into();
        if parsed.clear_reddit_runner_id == Some(true) {
            am.reddit_runner_id = Set(None);
        } else if let Some(v) = parsed.reddit_runner_id {
            let runner_id = if v > 0 { Some(v) } else { None };
            if let Some(rid) = runner_id {
                validate_runner_id(&db, rid).await?;
            }
            am.reddit_runner_id = Set(runner_id);
        }
        if let Some(v) = parsed.subreddits {
            let subs = normalize_subreddit_names(&v);
            if subs.is_empty() {
                return Err("at least one subreddit is required".into());
            }
            am.subreddits = Set(json!(subs));
        }
        if let Some(v) = parsed.search_query {
            am.search_query = Set(v);
        }
        if let Some(v) = parsed.filter {
            am.filter = Set(v);
        }
        if let Some(v) = parsed.is_filter_whitelist {
            am.is_filter_whitelist = Set(v);
        }
        if let Some(v) = parsed.max_fresh_posts {
            am.max_fresh_posts = Set(v);
        }
        if let Some(v) = parsed.load_websites {
            am.load_websites = Set(v);
        }
        am.updated_at = Set(Some(Utc::now()));
        am.update(&db).await.map_err(|e| e.to_string())
    })?;
    json_to_rune(json!({"reddit_source_id": row.id}))
}

fn reddit_delete_source(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize)]
    struct Args {
        reddit_source_id: i64,
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("reddit_delete_source", args)?)
        .map_err(|e| format!("invalid reddit_delete_source arguments: {e}"))?;
    let id = parsed.reddit_source_id;
    let db = ctx.db.clone();
    block_on_async(async move {
        RedditSourceEntity::find_by_id(id)
            .one(&db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "source not found".to_string())?;
        RedditSourceEntity::delete_by_id(id)
            .exec(&db)
            .await
            .map_err(|e| e.to_string())
    })?;
    json_to_rune(json!({"deleted": true}))
}

fn reddit_add_worker(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize)]
    struct Args {
        worker_name: String,
        #[serde(default)]
        worker_duration: Option<String>,
        #[serde(default)]
        source_ids: Vec<i64>,
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("reddit_add_worker", args)?)
        .map_err(|e| format!("invalid reddit_add_worker arguments: {e}"))?;
    let name = parsed.worker_name.trim().to_string();
    if name.is_empty() {
        return Err("worker_name required".into());
    }
    let dur = parse_duration_secs(parsed.worker_duration.as_deref().unwrap_or("5m"))?;
    let source_ids = parsed.source_ids;
    let now = Utc::now();
    let db = ctx.db.clone();
    let row = block_on_async(async move {
        let row = RedditRunnerAM {
            name: Set(name),
            duration_secs: Set(dur),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&db)
        .await
        .map_err(|e| e.to_string())?;
        sync_runner_sources(&db, row.id, &source_ids)
            .await
            .map_err(|e| e.to_string())?;
        Ok::<_, String>(row)
    })?;
    json_to_rune(json!({
        "reddit_runner_id": row.id,
        "duration_secs": row.duration_secs,
    }))
}

fn reddit_edit_worker(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize)]
    struct Args {
        reddit_runner_id: i64,
        #[serde(default)]
        worker_name: Option<String>,
        #[serde(default)]
        worker_duration: Option<String>,
        #[serde(default)]
        source_ids: Option<Vec<i64>>,
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("reddit_edit_worker", args)?)
        .map_err(|e| format!("invalid reddit_edit_worker arguments: {e}"))?;
    let id = parsed.reddit_runner_id;
    let source_ids = parsed.source_ids;
    let db = ctx.db.clone();
    let row = block_on_async(async move {
        let existing = RedditRunnerEntity::find_by_id(id)
            .one(&db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "runner not found".to_string())?;
        let mut am: RedditRunnerAM = existing.clone().into();
        let mut name = existing.name.clone();
        let mut duration_secs = existing.duration_secs;
        if let Some(v) = parsed
            .worker_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            name = v.into();
            am.name = Set(name.clone());
        }
        if let Some(v) = parsed.worker_duration.as_deref() {
            duration_secs = parse_duration_secs(v)?;
            am.duration_secs = Set(duration_secs);
        }
        am.updated_at = Set(Some(Utc::now()));
        let row = am.update(&db).await.map_err(|e| e.to_string())?;
        if let Some(source_ids) = source_ids {
            sync_runner_sources(&db, id, &source_ids)
                .await
                .map_err(|e| e.to_string())?;
        }
        if is_active_worker(RUNNER_KIND_REDDIT, id) {
            start_runner_pool(db.clone(), id, name, duration_secs);
        }
        Ok::<_, String>(row)
    })?;
    json_to_rune(json!({"reddit_runner_id": row.id}))
}

fn reddit_delete_worker(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize)]
    struct Args {
        reddit_runner_id: i64,
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("reddit_delete_worker", args)?)
        .map_err(|e| format!("invalid reddit_delete_worker arguments: {e}"))?;
    let id = parsed.reddit_runner_id;
    let db = ctx.db.clone();
    block_on_async(async move {
        RedditRunnerEntity::find_by_id(id)
            .one(&db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "runner not found".to_string())?;
        let linked = RedditSourceEntity::find()
            .filter(reddit_source::Column::RedditRunnerId.eq(id))
            .count(&db)
            .await
            .map_err(|e| e.to_string())?;
        if linked > 0 {
            return Err(format!(
                "cannot delete runner {id}: {linked} source(s) still assigned"
            ));
        }
        stop_runner_pool(id);
        RedditRunnerEntity::delete_by_id(id)
            .exec(&db)
            .await
            .map_err(|e| e.to_string())
    })?;
    json_to_rune(json!({"deleted": true}))
}
