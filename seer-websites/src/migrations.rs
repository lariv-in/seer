use sea_orm_migration::prelude::*;
use super::SeerWebsitesTag;
mod m20260825_000001_create_websites;
mod m20260827_000002_add_website_source_filters;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260825_000001_create_websites::Migration),
            Box::new(m20260827_000002_add_website_source_filters::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: SeerWebsitesTag;
    migrator: Migrator;
}
