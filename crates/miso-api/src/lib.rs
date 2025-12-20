//! # MISO API Server
//!
//! REST API server for MISO LIMS built with Axum.
//!
//! ## Architecture
//!
//! - **Routes**: HTTP endpoint handlers
//! - **Middleware**: Authentication, logging, CORS
//! - **State**: Shared application state (services, config)
//! - **Error Handling**: Consistent API error responses

pub mod config;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod state;

pub use config::Config;
pub use error::ApiError;
pub use state::AppState;

