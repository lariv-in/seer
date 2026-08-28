use sea_orm_migration::prelude::*;

use super::SeerWorkerRegistryTag;

mod m20260825_000001_create_worker_run_logs;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260825_000001_create_worker_run_logs::Migration)]
    }
}

lariv_rs::define_register_migrations! {
    plugin: SeerWorkerRegistryTag;
    migrator: Migrator;
}
