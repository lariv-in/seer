use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeerWorkerRunLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SeerWorkerRunLogs::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SeerWorkerRunLogs::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(SeerWorkerRunLogs::UpdatedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(SeerWorkerRunLogs::RunnerKind)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SeerWorkerRunLogs::RunnerId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SeerWorkerRunLogs::RunnerName)
                            .string_len(128)
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(SeerWorkerRunLogs::Status)
                            .string_len(16)
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(SeerWorkerRunLogs::StartedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SeerWorkerRunLogs::FinishedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(SeerWorkerRunLogs::DurationMs)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SeerWorkerRunLogs::ErrorMessage)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_wrl_lookup")
                    .table(SeerWorkerRunLogs::Table)
                    .col(SeerWorkerRunLogs::RunnerKind)
                    .col(SeerWorkerRunLogs::RunnerId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SeerWorkerRunLogs::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum SeerWorkerRunLogs {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    RunnerKind,
    RunnerId,
    RunnerName,
    Status,
    StartedAt,
    FinishedAt,
    DurationMs,
    ErrorMessage,
}
