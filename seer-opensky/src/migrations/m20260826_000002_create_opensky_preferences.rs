use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum SeerOpenskyPreferences {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    ClientId,
    ClientSecret,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SeerOpenskyPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SeerOpenskyPreferences::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SeerOpenskyPreferences::CreatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(SeerOpenskyPreferences::UpdatedAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(SeerOpenskyPreferences::ClientId)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(SeerOpenskyPreferences::ClientSecret)
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
                    .table(SeerOpenskyPreferences::Table)
                    .to_owned(),
            )
            .await
    }
}
