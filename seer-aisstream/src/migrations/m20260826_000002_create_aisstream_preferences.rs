use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum SeerAisstreamPreferences {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Enabled,
    ApiKey,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeerAisstreamPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SeerAisstreamPreferences::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SeerAisstreamPreferences::CreatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(SeerAisstreamPreferences::UpdatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(SeerAisstreamPreferences::Enabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(SeerAisstreamPreferences::ApiKey)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SeerAisstreamPreferences::Table)
                    .to_owned(),
            )
            .await
    }
}
