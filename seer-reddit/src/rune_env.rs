//! Rune sandbox bindings for Reddit ingestion sources and workers.

use std::sync::Arc;

use chrono::Utc;
use lariv_rs::rune_env::{
    block_on_async, json_to_rune, rune_to_json, NativeBinding, RuneEnvCapability, RuneEnvCtx,
    RuneEnvRegistrar,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use seer_common::parse_duration_secs;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};

use crate::{
    entities::{
        reddit_runner::{ActiveModel as RedditRunnerAM, Entity as RedditRunnerEntity},
        reddit_source::{ActiveModel as RedditSourceAM, Entity as RedditSourceEntity},
    },
    handlers::normalize_subreddit_names,
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
        "reddit_add_source",
        "reddit_add_source(#{ reddit_runner_id?: int, subreddits?: [string], search_query?: string, filter?: string, is_filter_whitelist?: bool, max_fresh_posts?: int, load_websites?: bool }) -> #{ reddit_source_id }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_add_source)),
    );
    rune_env.register_contextual(
        "reddit_edit_source",
        "reddit_edit_source(#{ reddit_source_id: int, reddit_runner_id?: int, subreddits?: [string], search_query?: string, filter?: string, is_filter_whitelist?: bool, max_fresh_posts?: int, load_websites?: bool }) -> #{ reddit_source_id }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_edit_source)),
    );
    rune_env.register_contextual(
        "reddit_add_worker",
        "reddit_add_worker(#{ worker_name: string, worker_duration?: string }) -> #{ reddit_runner_id, duration_secs }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_add_worker)),
    );
    rune_env.register_contextual(
        "reddit_edit_worker",
        "reddit_edit_worker(#{ reddit_runner_id: int, worker_name?: string, worker_duration?: string }) -> #{ reddit_runner_id }",
        |_ctx| NativeBinding::Function(Arc::new(reddit_edit_worker)),
    );
}

fn parse_object_arg(name: &str, args: &[rune::Value]) -> Result<JsonValue, String> {
    let value = args
        .first()
        .ok_or_else(|| format!("{name} requires an object argument"))?;
    rune_to_json(value).map_err(|e| format!("invalid {name} arguments: {e}"))
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
    let now = Utc::now();
    let db = ctx.db.clone();
    let row = block_on_async(async move {
        RedditSourceAM {
            reddit_runner_id: Set(parsed.reddit_runner_id),
            subreddits: Set(json!(subs)),
            search_query: Set(parsed.search_query.unwrap_or_default()),
            filter: Set(parsed.filter.unwrap_or_default()),
            is_filter_whitelist: Set(parsed.is_filter_whitelist.unwrap_or(false)),
            max_fresh_posts: Set(parsed.max_fresh_posts.unwrap_or(25)),
            load_websites: Set(parsed.load_websites.unwrap_or(false)),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(&db)
        .await
    })
    .map_err(|e| e.to_string())?;
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
        if let Some(v) = parsed.reddit_runner_id {
            am.reddit_runner_id = Set(Some(v));
        }
        if let Some(v) = parsed.subreddits {
            am.subreddits = Set(json!(normalize_subreddit_names(&v)));
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

fn reddit_add_worker(
    ctx: &RuneEnvCtx<'_>,
    args: &[rune::Value],
) -> Result<rune::Value, String> {
    #[derive(Debug, Deserialize)]
    struct Args {
        worker_name: String,
        #[serde(default)]
        worker_duration: Option<String>,
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("reddit_add_worker", args)?)
        .map_err(|e| format!("invalid reddit_add_worker arguments: {e}"))?;
    let name = parsed.worker_name.trim().to_string();
    if name.is_empty() {
        return Err("worker_name required".into());
    }
    let dur = parse_duration_secs(parsed.worker_duration.as_deref().unwrap_or("5m"))?;
    let now = Utc::now();
    let db = ctx.db.clone();
    let row = block_on_async(async move {
        RedditRunnerAM {
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
    }

    let parsed: Args = serde_json::from_value(parse_object_arg("reddit_edit_worker", args)?)
        .map_err(|e| format!("invalid reddit_edit_worker arguments: {e}"))?;
    let id = parsed.reddit_runner_id;
    let db = ctx.db.clone();
    let row = block_on_async(async move {
        let existing = RedditRunnerEntity::find_by_id(id)
            .one(&db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "runner not found".to_string())?;
        let mut am: RedditRunnerAM = existing.into();
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
    json_to_rune(json!({"reddit_runner_id": row.id}))
}
