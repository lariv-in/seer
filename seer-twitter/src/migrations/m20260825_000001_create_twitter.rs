use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)] pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(SeerTwitterRunners::Table).if_not_exists()
            .col(ColumnDef::new(SeerTwitterRunners::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(SeerTwitterRunners::CreatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerTwitterRunners::UpdatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerTwitterRunners::Name).string_len(64).not_null().unique_key())
            .col(ColumnDef::new(SeerTwitterRunners::DurationSecs).big_integer().not_null()).to_owned()).await?;
        manager.create_table(Table::create().table(SeerTwitterSources::Table).if_not_exists()
            .col(ColumnDef::new(SeerTwitterSources::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(SeerTwitterSources::CreatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerTwitterSources::UpdatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerTwitterSources::TwitterRunnerId).big_integer())
            .col(ColumnDef::new(SeerTwitterSources::Usernames).json().not_null())
            .col(ColumnDef::new(SeerTwitterSources::SearchQuery).text().not_null().default(""))
            .col(ColumnDef::new(SeerTwitterSources::Filter).text().not_null().default(""))
            .col(ColumnDef::new(SeerTwitterSources::IsFilterWhitelist).boolean().not_null().default(false))
            .col(ColumnDef::new(SeerTwitterSources::MaxFreshPosts).big_integer().not_null().default(25))
            .col(ColumnDef::new(SeerTwitterSources::LoadWebsites).boolean().not_null().default(false)).to_owned()).await?;
        manager.create_table(Table::create().table(SeerTwitterPosts::Table).if_not_exists()
            .col(ColumnDef::new(SeerTwitterPosts::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(SeerTwitterPosts::CreatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerTwitterPosts::UpdatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerTwitterPosts::TwitterRunnerId).big_integer())
            .col(ColumnDef::new(SeerTwitterPosts::PostId).string_len(64).not_null().unique_key())
            .col(ColumnDef::new(SeerTwitterPosts::Title).text().not_null())
            .col(ColumnDef::new(SeerTwitterPosts::Selftext).text().not_null())
            .col(ColumnDef::new(SeerTwitterPosts::Author).text().not_null())
            .col(ColumnDef::new(SeerTwitterPosts::Permalink).text().not_null())
            .col(ColumnDef::new(SeerTwitterPosts::Url).text().not_null())
            .col(ColumnDef::new(SeerTwitterPosts::CreatedUtcUnix).double().not_null()).to_owned()).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(SeerTwitterPosts::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SeerTwitterSources::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SeerTwitterRunners::Table).to_owned()).await
    }
}
#[derive(Iden)] enum SeerTwitterRunners { Table, Id, CreatedAt, UpdatedAt, Name, DurationSecs }
#[derive(Iden)] enum SeerTwitterSources { Table, Id, CreatedAt, UpdatedAt, TwitterRunnerId, Usernames, SearchQuery, Filter, IsFilterWhitelist, MaxFreshPosts, LoadWebsites }
#[derive(Iden)] enum SeerTwitterPosts { Table, Id, CreatedAt, UpdatedAt, TwitterRunnerId, PostId, Title, Selftext, Author, Permalink, Url, CreatedUtcUnix }
