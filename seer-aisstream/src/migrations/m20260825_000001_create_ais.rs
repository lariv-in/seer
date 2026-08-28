use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)] pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(SeerAisstreamPositionReports::Table).if_not_exists()
            .col(ColumnDef::new(SeerAisstreamPositionReports::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(SeerAisstreamPositionReports::CreatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerAisstreamPositionReports::UpdatedAt).timestamp_with_time_zone())
            .col(ColumnDef::new(SeerAisstreamPositionReports::Mmsi).big_integer().not_null())
            .col(ColumnDef::new(SeerAisstreamPositionReports::Latitude).double())
            .col(ColumnDef::new(SeerAisstreamPositionReports::Longitude).double())
            .col(ColumnDef::new(SeerAisstreamPositionReports::Sog).double())
            .col(ColumnDef::new(SeerAisstreamPositionReports::Cog).double())
            .col(ColumnDef::new(SeerAisstreamPositionReports::Heading).double())
            .col(ColumnDef::new(SeerAisstreamPositionReports::ShipName).text().not_null()).to_owned()).await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(SeerAisstreamPositionReports::Table).to_owned()).await
    }
}
#[derive(Iden)]
enum SeerAisstreamPositionReports { Table, Id, CreatedAt, UpdatedAt, Mmsi, Latitude, Longitude, Sog, Cog, Heading, ShipName }
