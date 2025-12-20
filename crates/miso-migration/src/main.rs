//! MISO LIMS Database Migration CLI
//!
//! Usage:
//!   miso-migrate up      - Apply all pending migrations
//!   miso-migrate down    - Rollback last migration
//!   miso-migrate status  - Show migration status

use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    cli::run_cli(miso_migration::Migrator).await;
}

