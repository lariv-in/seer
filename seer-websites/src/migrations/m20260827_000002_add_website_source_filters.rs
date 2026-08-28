use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SeerWebsiteSources::Table)
                    .add_column(
                        ColumnDef::new(SeerWebsiteSources::Filter)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .add_column(
                        ColumnDef::new(SeerWebsiteSources::IsFilterWhitelist)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SeerWebsiteSources::Table)
                    .drop_column(SeerWebsiteSources::Filter)
                    .drop_column(SeerWebsiteSources::IsFilterWhitelist)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum SeerWebsiteSources {
    Table,
    Filter,
    IsFilterWhitelist,
}
