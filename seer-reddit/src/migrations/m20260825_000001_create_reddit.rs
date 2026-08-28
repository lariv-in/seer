use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(SeerRedditRunners::Table).if_not_exists()
            .col(ColumnDef::new(SeerRedditRunners::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(SeerRedditRunners::CreatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerRedditRunners::UpdatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerRedditRunners::Name).string_len(64).not_null().unique_key())
            .col(ColumnDef::new(SeerRedditRunners::DurationSecs).big_integer().not_null())
            .to_owned()).await?;
        manager.create_table(Table::create().table(SeerRedditSources::Table).if_not_exists()
            .col(ColumnDef::new(SeerRedditSources::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(SeerRedditSources::CreatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerRedditSources::UpdatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerRedditSources::RedditRunnerId).big_integer())
            .col(ColumnDef::new(SeerRedditSources::Subreddits).json().not_null())
            .col(ColumnDef::new(SeerRedditSources::SearchQuery).text().not_null().default(""))
            .col(ColumnDef::new(SeerRedditSources::Filter).text().not_null().default(""))
            .col(ColumnDef::new(SeerRedditSources::IsFilterWhitelist).boolean().not_null().default(false))
            .col(ColumnDef::new(SeerRedditSources::MaxFreshPosts).big_integer().not_null().default(25))
            .col(ColumnDef::new(SeerRedditSources::LoadWebsites).boolean().not_null().default(false))
            .to_owned()).await?;
        manager.create_table(Table::create().table(SeerRedditPosts::Table).if_not_exists()
            .col(ColumnDef::new(SeerRedditPosts::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(SeerRedditPosts::CreatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerRedditPosts::UpdatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerRedditPosts::RedditRunnerId).big_integer())
            .col(ColumnDef::new(SeerRedditPosts::PostId).string_len(32).not_null().unique_key())
            .col(ColumnDef::new(SeerRedditPosts::Title).text().not_null())
            .col(ColumnDef::new(SeerRedditPosts::Selftext).text().not_null())
            .col(ColumnDef::new(SeerRedditPosts::Author).text().not_null())
            .col(ColumnDef::new(SeerRedditPosts::Subreddit).text().not_null())
            .col(ColumnDef::new(SeerRedditPosts::Permalink).text().not_null())
            .col(ColumnDef::new(SeerRedditPosts::Url).text().not_null())
            .col(ColumnDef::new(SeerRedditPosts::CreatedUtcUnix).double().not_null())
            .col(ColumnDef::new(SeerRedditPosts::Score).big_integer().not_null().default(0))
            .col(ColumnDef::new(SeerRedditPosts::NumComments).big_integer().not_null().default(0))
            .col(ColumnDef::new(SeerRedditPosts::IsSelf).boolean().not_null().default(false))
            .to_owned()).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(SeerRedditPosts::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SeerRedditSources::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SeerRedditRunners::Table).to_owned()).await
    }
}
#[derive(Iden)] enum SeerRedditRunners { Table, Id, CreatedAt, UpdatedAt, Name, DurationSecs }
#[derive(Iden)] enum SeerRedditSources { Table, Id, CreatedAt, UpdatedAt, RedditRunnerId, Subreddits, SearchQuery, Filter, IsFilterWhitelist, MaxFreshPosts, LoadWebsites }
#[derive(Iden)] enum SeerRedditPosts { Table, Id, CreatedAt, UpdatedAt, RedditRunnerId, PostId, Title, Selftext, Author, Subreddit, Permalink, Url, CreatedUtcUnix, Score, NumComments, IsSelf }
