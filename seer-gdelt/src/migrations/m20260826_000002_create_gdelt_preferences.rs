use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum SeerGdeltPreferences {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    ProjectId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeerGdeltPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SeerGdeltPreferences::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SeerGdeltPreferences::CreatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(SeerGdeltPreferences::UpdatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(SeerGdeltPreferences::ProjectId)
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
                    .table(SeerGdeltPreferences::Table)
                    .to_owned(),
            )
            .await
    }
}
