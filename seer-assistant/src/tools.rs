//! Seer LLM assistant tools.

use async_trait::async_trait;
use lariv_rs::{
    genai::FunctionDeclaration,
    llm_tools::{LlmTool, LlmToolsCapability, ToolCtx, ToolsRegistrar},
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_json::{json, Value};

use seer_intel::entities::intel::{self, Entity as IntelEntity};

#[derive(Clone, Copy, Default)]
pub struct Hook;

impl ToolsRegistrar for Hook {
    fn register_tools(self, tools: &mut LlmToolsCapability) {
        tools.register(IntelSearchTool);
    }
}

pub struct IntelSearchTool;
#[async_trait]
impl LlmTool for IntelSearchTool {
    fn name(&self) -> &str {
        "intel_search"
    }
    fn declaration(&self) -> FunctionDeclaration {
        FunctionDeclaration {
            name: "intel_search".into(),
            description: "Search Seer intel by title/summary keyword.".into(),
            parameters: Some(json!({
                "type":"object",
                "properties":{"query":{"type":"string"},"limit":{"type":"integer"}},
                "required":["query"]
            })),
        }
    }
    async fn run(&self, ctx: &ToolCtx<'_>, args: Value) -> Result<Value, String> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if query.is_empty() {
            return Err("query required".into());
        }
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20).min(50);
        let rows = IntelEntity::find()
            .filter(
                Condition::any()
                    .add(intel::Column::Title.contains(&query))
                    .add(intel::Column::Summary.contains(&query)),
            )
            .order_by_desc(intel::Column::Id)
            .limit(limit)
            .all(ctx.db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(json!({
            "results": rows.into_iter().map(|r| json!({
                "id": r.id, "title": r.title, "kind": r.kind, "kind_id": r.kind_id,
                "summary": r.summary.chars().take(400).collect::<String>(),
            })).collect::<Vec<_>>()
        }))
    }
}
