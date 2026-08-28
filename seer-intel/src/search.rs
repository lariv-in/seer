use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, EntityTrait, FromQueryResult, Statement};

use crate::{
    config::SeerIntelConfig,
    embed,
    entities::intel::Model as Intel,
};

#[derive(Debug, FromQueryResult)]
struct IntelRow {
    id: i64,
}

pub async fn search_intel_by_similarity(
    db: &DatabaseConnection,
    config: &SeerIntelConfig,
    query: &str,
    limit: u64,
) -> anyhow::Result<Vec<Intel>> {
    let limit = if limit == 0 { 10 } else { limit.min(100) };
    if db.get_database_backend() != DatabaseBackend::Postgres {
        anyhow::bail!("vector search requires postgres");
    }
    let values = embed::embed_query_text(config, query).await?;
    let lit = embed::vector_to_pg_literal(&values);
    let sql = format!(
        "SELECT id FROM seer_intels \
         WHERE embedding IS NOT NULL \
         ORDER BY embedding <=> '{lit}'::vector ASC \
         LIMIT {limit}"
    );
    let rows: Vec<IntelRow> = IntelRow::find_by_statement(Statement::from_string(
        DatabaseBackend::Postgres,
        sql,
    ))
    .all(db)
    .await?;

    // Re-load full models by id for callers.
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        if let Some(m) = crate::entities::intel::Entity::find_by_id(r.id)
            .one(db)
            .await?
        {
            out.push(m);
        }
    }
    Ok(out)
}
