use sea_orm_migration::prelude::*;
use super::SeerRedditTag;
mod m20260825_000001_create_reddit;
#[derive(Clone, Copy, Default)]
pub struct Migrator;
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260825_000001_create_reddit::Migration)]
    }
}
lariv_rs::define_register_migrations! { plugin: SeerRedditTag; migrator: Migrator; }
