use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use sea_orm_migration::prelude::*;
use tracing::warn;

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn vector_type_available(conn: &SchemaManagerConnection<'_>) -> bool {
    let Ok(rows) = conn
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            String::from("SELECT 1 FROM pg_type WHERE typname = 'vector' LIMIT 1"),
        ))
        .await
    else {
        return false;
    };
    !rows.is_empty()
}

async fn create_postgres_tables(
    conn: &SchemaManagerConnection<'_>,
    use_vector: bool,
) -> Result<(), DbErr> {
    let embedding_col = if use_vector {
        "embedding vector(3072)"
    } else {
        // Fallback when pgvector is not installed. Vector search requires:
        //   CREATE EXTENSION vector;  (as a DB superuser, outside migrate)
        // then re-migrate / alter column.
        "embedding TEXT"
    };

    conn.execute_unprepared(&format!(
        r#"
CREATE TABLE IF NOT EXISTS seer_intels (
  id BIGSERIAL PRIMARY KEY,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  title TEXT NOT NULL DEFAULT '',
  summary TEXT NOT NULL DEFAULT '',
  datetime TIMESTAMPTZ NOT NULL,
  {embedding_col},
  kind TEXT NOT NULL DEFAULT '',
  kind_id BIGINT NOT NULL DEFAULT 0
)
"#
    ))
    .await?;

    conn.execute_unprepared(
        "CREATE INDEX IF NOT EXISTS idx_seer_intels_kind ON seer_intels (kind)",
    )
    .await?;
    conn.execute_unprepared(
        "CREATE INDEX IF NOT EXISTS idx_seer_intels_kind_id ON seer_intels (kind_id)",
    )
    .await?;

    conn.execute_unprepared(
        r#"
CREATE TABLE IF NOT EXISTS seer_intel_events (
  id BIGSERIAL PRIMARY KEY,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  intel_id BIGINT NOT NULL UNIQUE,
  address TEXT NOT NULL DEFAULT '',
  datetime TIMESTAMPTZ NOT NULL,
  latitude DOUBLE PRECISION,
  longitude DOUBLE PRECISION
)
"#,
    )
    .await?;

    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        if backend == DatabaseBackend::Postgres {
            let conn = manager.get_connection();
            // Do NOT run CREATE EXTENSION inside the migration transaction: a failure
            // aborts the txn and every later statement fails with
            // "current transaction is aborted". Install pgvector out-of-band if needed:
            //   CREATE EXTENSION IF NOT EXISTS vector;
            let use_vector = vector_type_available(conn).await;
            if !use_vector {
                warn!(
                    "seer-intel: pgvector type not found; creating embedding as TEXT. \
                     For vector search, run `CREATE EXTENSION IF NOT EXISTS vector` as a \
                     superuser, then recreate/alter seer_intels.embedding to vector(3072)."
                );
            }
            return create_postgres_tables(conn, use_vector).await;
        }

        // SQLite / other: text embedding column for mount smoke tests.
        manager
            .create_table(
                Table::create()
                    .table(SeerIntels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SeerIntels::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SeerIntels::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(SeerIntels::UpdatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(SeerIntels::Title).text().not_null())
                    .col(ColumnDef::new(SeerIntels::Summary).text().not_null())
                    .col(
                        ColumnDef::new(SeerIntels::Datetime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SeerIntels::Embedding).text())
                    .col(ColumnDef::new(SeerIntels::Kind).string_len(64).not_null())
                    .col(
                        ColumnDef::new(SeerIntels::KindId)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SeerIntelEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SeerIntelEvents::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SeerIntelEvents::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(SeerIntelEvents::UpdatedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(SeerIntelEvents::IntelId)
                            .big_integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(SeerIntelEvents::Address).text().not_null())
                    .col(
                        ColumnDef::new(SeerIntelEvents::Datetime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SeerIntelEvents::Latitude).double())
                    .col(ColumnDef::new(SeerIntelEvents::Longitude).double())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SeerIntelEvents::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SeerIntels::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum SeerIntels {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Title,
    Summary,
    Datetime,
    Embedding,
    Kind,
    KindId,
}

#[derive(Iden)]
enum SeerIntelEvents {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    IntelId,
    Address,
    Datetime,
    Latitude,
    Longitude,
}
