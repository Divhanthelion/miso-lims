//! Value Objects - Immutable objects defined by their attributes.
//!
//! Value objects have no identity; two value objects with the same
//! attributes are considered equal.

mod barcode;
mod concentration;
mod dna_index;
mod position;
mod qc_status;
mod volume;

pub use barcode::Barcode;
pub use concentration::Concentration;
pub use dna_index::{DnaIndex, IndexFamily};
pub use position::{BoxPosition, Dimension};
pub use qc_status::{QcResult, QcStatus};
pub use volume::Volume;

