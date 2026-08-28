use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum SeerIntelPreferences {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    GeocodingApiKey,
    TitleModel,
    SummaryModel,
    EmbeddingModel,
    ApiKey,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeerIntelPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SeerIntelPreferences::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SeerIntelPreferences::CreatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(SeerIntelPreferences::UpdatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(SeerIntelPreferences::GeocodingApiKey)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(SeerIntelPreferences::TitleModel)
                            .text()
                            .not_null()
                            .default("gemini-2.0-flash"),
                    )
                    .col(
                        ColumnDef::new(SeerIntelPreferences::SummaryModel)
                            .text()
                            .not_null()
                            .default("gemini-2.0-flash"),
                    )
                    .col(
                        ColumnDef::new(SeerIntelPreferences::EmbeddingModel)
                            .text()
                            .not_null()
                            .default("gemini-embedding-001"),
                    )
                    .col(
                        ColumnDef::new(SeerIntelPreferences::ApiKey)
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
                    .table(SeerIntelPreferences::Table)
                    .to_owned(),
            )
            .await
    }
}
