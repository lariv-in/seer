use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum SeerTwitterPreferences {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    NitterInstanceUrl,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeerTwitterPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SeerTwitterPreferences::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SeerTwitterPreferences::CreatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(SeerTwitterPreferences::UpdatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(SeerTwitterPreferences::NitterInstanceUrl)
                            .text()
                            .not_null()
                            .default("https://nitter.net"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SeerTwitterPreferences::Table)
                    .to_owned(),
            )
            .await
    }
}
