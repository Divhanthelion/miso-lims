//! # MISO Domain Layer
//!
//! This crate contains the core domain logic for MISO LIMS.
//! It is intentionally dependency-free from database and web frameworks
//! to ensure pure business logic that is easily testable.
//!
//! ## Architecture
//!
//! The domain layer follows Domain-Driven Design (DDD) principles:
//! - **Entities**: Objects with identity (Sample, Library, Pool, Run)
//! - **Value Objects**: Immutable objects defined by their attributes (Barcode, Concentration)
//! - **Repository Traits**: Interfaces for data persistence (implemented in infrastructure)
//! - **Domain Services**: Business logic that doesn't belong to a single entity
//! - **Domain Errors**: Semantic errors representing domain rule violations

pub mod entities;
pub mod errors;
pub mod repositories;
pub mod services;
pub mod value_objects;

// Re-export commonly used types
pub use entities::*;
pub use errors::DomainError;
pub use value_objects::*;

