//! # MISO Application Layer
//!
//! This crate contains the application logic (use cases) that orchestrate
//! domain entities and repository operations.
//!
//! ## Architecture
//!
//! - **Use Cases**: Specific operations (e.g., CreateSample, PoolLibraries)
//! - **DTOs**: Data Transfer Objects for API boundaries
//! - **Services**: Application services that coordinate complex workflows

pub mod dto;
pub mod services;
pub mod use_cases;

// Re-export commonly used types
pub use dto::*;
pub use services::*;

