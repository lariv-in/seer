use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)] pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(SeerOpenskyStates::Table).if_not_exists()
            .col(ColumnDef::new(SeerOpenskyStates::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(SeerOpenskyStates::CreatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerOpenskyStates::UpdatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerOpenskyStates::SnapshotTime).big_integer().not_null())
            .col(ColumnDef::new(SeerOpenskyStates::Icao24).string_len(8).not_null())
            .col(ColumnDef::new(SeerOpenskyStates::LastContact).big_integer().not_null())
            .col(ColumnDef::new(SeerOpenskyStates::Callsign).string_len(8))
            .col(ColumnDef::new(SeerOpenskyStates::OriginCountry).string_len(64))
            .col(ColumnDef::new(SeerOpenskyStates::TimePosition).big_integer())
            .col(ColumnDef::new(SeerOpenskyStates::Latitude).double())
            .col(ColumnDef::new(SeerOpenskyStates::Longitude).double())
            .col(ColumnDef::new(SeerOpenskyStates::BaroAltitude).double())
            .col(ColumnDef::new(SeerOpenskyStates::OnGround).boolean())
            .col(ColumnDef::new(SeerOpenskyStates::Velocity).double())
            .col(ColumnDef::new(SeerOpenskyStates::TrueTrack).double())
            .col(ColumnDef::new(SeerOpenskyStates::VerticalRate).double())
            .col(ColumnDef::new(SeerOpenskyStates::GeoAltitude).double())
            .col(ColumnDef::new(SeerOpenskyStates::Squawk).string_len(8)).to_owned()).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(SeerOpenskyStates::Table).to_owned()).await
    }
}
#[derive(Iden)]
enum SeerOpenskyStates { Table, Id, CreatedAt, UpdatedAt, SnapshotTime, Icao24, LastContact, Callsign, OriginCountry, TimePosition, Latitude, Longitude, BaroAltitude, OnGround, Velocity, TrueTrack, VerticalRate, GeoAltitude, Squawk }
