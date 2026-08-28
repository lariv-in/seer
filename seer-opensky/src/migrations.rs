use sea_orm_migration::prelude::*;

use super::SeerOpenskyTag;

mod m20260825_000001_create_opensky;
mod m20260826_000002_create_opensky_preferences;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260825_000001_create_opensky::Migration),
            Box::new(m20260826_000002_create_opensky_preferences::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: SeerOpenskyTag;
    migrator: Migrator;
}
