//! API error handling.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// API error response body.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// API error type.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Domain error: {0}")]
    Domain(#[from] miso_domain::errors::DomainError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.clone()),
            ApiError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, "validation_error", msg.clone()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "Authentication required".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", "Permission denied".to_string()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            ApiError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "An unexpected error occurred".to_string())
            }
            ApiError::Domain(e) => {
                let (status, error_type) = match e {
                    miso_domain::errors::DomainError::NotFound { .. } => (StatusCode::NOT_FOUND, "not_found"),
                    miso_domain::errors::DomainError::Duplicate { .. } => (StatusCode::CONFLICT, "duplicate"),
                    miso_domain::errors::DomainError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, "validation_error"),
                    _ => (StatusCode::BAD_REQUEST, "domain_error"),
                };
                (status, error_type, e.to_string())
            }
        };

        let body = ErrorResponse {
            error: error_type.to_string(),
            message,
            details: None,
        };

        (status, Json(body)).into_response()
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        ApiError::Validation(format!("Validation failed: {}", errors))
    }
}

