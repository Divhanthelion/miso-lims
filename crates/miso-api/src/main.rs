//! MISO LIMS API Server
//!
//! A high-performance REST API server for the MISO Laboratory Information
//! Management System, built with Axum and Tokio.

use std::sync::Arc;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use miso_api::{routes, AppState, Config};
use miso_infrastructure::persistence::{
    database::{Database, DatabaseConfig},
    repositories::{SeaOrmProjectRepository, SeaOrmSampleRepository},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting MISO LIMS API Server v{}", env!("CARGO_PKG_VERSION"));

    // Connect to database
    let db = Database::connect(DatabaseConfig::new(&config.database_url))
        .await
        .expect("Failed to connect to database");

    // Create repositories
    let project_repo = Arc::new(SeaOrmProjectRepository::new(db.connection().clone()));
    let sample_repo = Arc::new(SeaOrmSampleRepository::new(db.connection().clone()));

    // Create application state
    let state = AppState::new(config.clone(), project_repo, sample_repo);

    // Create router
    let app = routes::create_router(state);

    // Start server
    let addr = config.address();
    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

