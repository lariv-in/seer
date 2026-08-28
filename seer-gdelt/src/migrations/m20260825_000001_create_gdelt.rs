use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)] pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(SeerGdeltWorkers::Table).if_not_exists()
            .col(ColumnDef::new(SeerGdeltWorkers::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(SeerGdeltWorkers::CreatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerGdeltWorkers::UpdatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerGdeltWorkers::Name).string_len(64).not_null().unique_key())
            .col(ColumnDef::new(SeerGdeltWorkers::DurationSecs).big_integer().not_null()).to_owned()).await?;
        manager.create_table(Table::create().table(SeerGdeltSources::Table).if_not_exists()
            .col(ColumnDef::new(SeerGdeltSources::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(SeerGdeltSources::CreatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerGdeltSources::UpdatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerGdeltSources::GdeltWorkerId).big_integer())
            .col(ColumnDef::new(SeerGdeltSources::Query).text().not_null().default(""))
            .col(ColumnDef::new(SeerGdeltSources::Domain).text().not_null().default(""))
            .col(ColumnDef::new(SeerGdeltSources::ActionCountry).text().not_null().default(""))
            .col(ColumnDef::new(SeerGdeltSources::StartDate).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerGdeltSources::EndDate).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerGdeltSources::MinMentions).big_integer().not_null().default(0))
            .col(ColumnDef::new(SeerGdeltSources::MaxRecords).big_integer().not_null().default(0))
            .col(ColumnDef::new(SeerGdeltSources::Sort).string_len(32).not_null().default(""))
            .col(ColumnDef::new(SeerGdeltSources::NaturalLanguageFilter).text().not_null().default(""))
            .col(ColumnDef::new(SeerGdeltSources::IsBlacklist).boolean().not_null().default(false)).to_owned()).await?;
        manager.create_table(Table::create().table(SeerGdeltEvents::Table).if_not_exists()
            .col(ColumnDef::new(SeerGdeltEvents::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(SeerGdeltEvents::CreatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerGdeltEvents::UpdatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerGdeltEvents::GdeltSourceId).big_integer())
            .col(ColumnDef::new(SeerGdeltEvents::GlobalEventId).text().not_null())
            .col(ColumnDef::new(SeerGdeltEvents::SourceUrl).text().not_null())
            .col(ColumnDef::new(SeerGdeltEvents::Actor1Name).text().not_null())
            .col(ColumnDef::new(SeerGdeltEvents::Actor2Name).text().not_null())
            .col(ColumnDef::new(SeerGdeltEvents::ActionGeoLat).double())
            .col(ColumnDef::new(SeerGdeltEvents::ActionGeoLong).double())
            .col(ColumnDef::new(SeerGdeltEvents::EventDate).timestamp_with_time_zone()).to_owned()).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(SeerGdeltEvents::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SeerGdeltSources::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(SeerGdeltWorkers::Table).to_owned()).await
    }
}
#[derive(Iden)] enum SeerGdeltWorkers { Table, Id, CreatedAt, UpdatedAt, Name, DurationSecs }
#[derive(Iden)] enum SeerGdeltSources { Table, Id, CreatedAt, UpdatedAt, GdeltWorkerId, Query, Domain, ActionCountry, StartDate, EndDate, MinMentions, MaxRecords, Sort, NaturalLanguageFilter, IsBlacklist }
#[derive(Iden)] enum SeerGdeltEvents { Table, Id, CreatedAt, UpdatedAt, GdeltSourceId, GlobalEventId, SourceUrl, Actor1Name, Actor2Name, ActionGeoLat, ActionGeoLong, EventDate }
