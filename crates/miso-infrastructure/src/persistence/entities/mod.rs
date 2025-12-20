//! SeaORM Entity Definitions
//!
//! These entities map directly to the MySQL database tables.
//! They are generated/maintained to match the legacy MISO schema.

pub mod project;
pub mod sample;

// Re-export entity types
pub use project::Entity as ProjectEntity;
pub use sample::Entity as SampleEntity;

