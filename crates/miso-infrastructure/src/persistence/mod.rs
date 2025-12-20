//! Database persistence layer.
//!
//! Provides SeaORM-based implementations of domain repository traits.

pub mod database;
pub mod entities;
pub mod repositories;

pub use database::Database;

