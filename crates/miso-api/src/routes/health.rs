//! Health check endpoints.

use axum::Json;
use serde::Serialize;

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Health check endpoint.
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Readiness check response.
#[derive(Serialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub database: String,
}

/// Readiness check endpoint.
pub async fn readiness_check() -> Json<ReadinessResponse> {
    // TODO: Check database connectivity
    Json(ReadinessResponse {
        ready: true,
        database: "connected".to_string(),
    })
}

