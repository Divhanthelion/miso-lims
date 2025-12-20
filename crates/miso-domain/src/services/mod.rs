//! Domain Services - business logic that spans multiple entities.
//!
//! These services contain pure domain logic that doesn't belong to a single
//! entity. They are dependency-free and can be tested in isolation.

mod barcode_validation;
mod index_collision;

pub use barcode_validation::BarcodeValidator;
pub use index_collision::IndexCollisionChecker;

