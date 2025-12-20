//! # MISO Infrastructure Layer
//!
//! This crate provides concrete implementations of the domain interfaces:
//! - **Persistence**: SeaORM-based repository implementations
//! - **Hardware**: Async clients for lab equipment (VisionMate scanners, printers)
//! - **External Services**: LDAP authentication, etc.

pub mod hardware;
pub mod persistence;

// Re-export commonly used types
pub use hardware::scanner::VisionMateClient;
pub use hardware::printer::ZebraPrinter;
pub use persistence::database::Database;

