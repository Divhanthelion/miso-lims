//! Domain Entities - Objects with identity and lifecycle.
//!
//! Entities are distinguished by their identity (ID), not their attributes.
//! Two samples with identical attributes but different IDs are different entities.

mod box_entity;
mod library;
mod pool;
mod project;
mod run;
mod sample;
mod sequencer;
mod user;

pub use box_entity::{StorageBox, StorageLocation};
pub use library::{Library, LibraryAliquot, LibraryDesign, LibraryType};
pub use pool::Pool;
pub use project::Project;
pub use run::{Run, RunPartition, RunStatus};
pub use sample::{DetailedSampleData, PlainSampleData, Sample, SampleClass, SampleDetails};
pub use sequencer::{ContainerModel, InstrumentModel, Platform, Sequencer};
pub use user::{Role, User};

/// Type alias for entity IDs.
pub type EntityId = i32;

