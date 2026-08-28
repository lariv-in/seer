use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create().table(SeerWebsites::Table).if_not_exists()
                .col(ColumnDef::new(SeerWebsites::Id).big_integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(SeerWebsites::CreatedAt).timestamp_with_time_zone())
                .col(ColumnDef::new(SeerWebsites::UpdatedAt).timestamp_with_time_zone())
                .col(ColumnDef::new(SeerWebsites::Url).text().not_null())
                .col(ColumnDef::new(SeerWebsites::Markdown).text().not_null())
                .to_owned(),
        ).await?;
        manager.create_table(
            Table::create().table(SeerWebsiteRunners::Table).if_not_exists()
                .col(ColumnDef::new(SeerWebsiteRunners::Id).big_integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(SeerWebsiteRunners::CreatedAt).timestamp_with_time_zone())
                .col(ColumnDef::new(SeerWebsiteRunners::UpdatedAt).timestamp_with_time_zone())
                .col(ColumnDef::new(SeerWebsiteRunners::Name).string_len(64).not_null().unique_key())
                .col(ColumnDef::new(SeerWebsiteRunners::DurationSecs).big_integer().not_null())
                .to_owned(),
        ).await?;
        manager.create_table(
            Table::create().table(SeerWebsiteSources::Table).if_not_exists()
                .col(ColumnDef::new(SeerWebsiteSources::Id).big_integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(SeerWebsiteSources::CreatedAt).timestamp_with_time_zone())
                .col(ColumnDef::new(SeerWebsiteSources::UpdatedAt).timestamp_with_time_zone())
                .col(ColumnDef::new(SeerWebsiteSources::WebsiteRunnerId).big_integer())
                .col(ColumnDef::new(SeerWebsiteSources::Url).text().not_null())
                .col(ColumnDef::new(SeerWebsiteSources::Depth).big_integer().not_null().default(0))
                .to_owned(),
        ).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(SeerWebsiteSources::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SeerWebsiteRunners::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SeerWebsites::Table).to_owned()).await
    }
}

#[derive(Iden)]
enum SeerWebsites { Table, Id, CreatedAt, UpdatedAt, Url, Markdown }
#[derive(Iden)]
enum SeerWebsiteRunners { Table, Id, CreatedAt, UpdatedAt, Name, DurationSecs }
#[derive(Iden)]
enum SeerWebsiteSources { Table, Id, CreatedAt, UpdatedAt, WebsiteRunnerId, Url, Depth }
