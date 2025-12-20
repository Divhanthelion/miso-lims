//! API route handlers.

pub mod health;
pub mod projects;
pub mod samples;
pub mod scanner;

use axum::{routing::get, Router};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::AppState;
use miso_domain::repositories::{ProjectRepository, SampleRepository};

/// Creates the API router.
pub fn create_router<PR, SR>(state: AppState<PR, SR>) -> Router
where
    PR: ProjectRepository + 'static,
    SR: SampleRepository + 'static,
{
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        // API v1 routes
        .nest("/api/v1", api_v1_routes())
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// API v1 routes.
fn api_v1_routes<PR, SR>() -> Router<AppState<PR, SR>>
where
    PR: ProjectRepository + 'static,
    SR: SampleRepository + 'static,
{
    Router::new()
        .nest("/projects", projects::routes())
        .nest("/samples", samples::routes())
        .nest("/scanner", scanner::routes())
}

