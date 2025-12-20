//! MISO LIMS Database Migrations
//!
//! Uses SeaORM migration framework to manage database schema.

pub use sea_orm_migration::prelude::*;

mod m20241215_000001_create_project;
mod m20241215_000002_create_sample;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241215_000001_create_project::Migration),
            Box::new(m20241215_000002_create_sample::Migration),
        ]
    }
}

