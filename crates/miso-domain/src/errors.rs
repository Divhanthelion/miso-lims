//! Domain-specific errors representing business rule violations.
//!
//! These errors are semantic - they describe what went wrong in domain terms,
//! not infrastructure terms (no "database connection failed" here).

use thiserror::Error;

/// The root error type for all domain operations.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Sample error: {0}")]
    Sample(#[from] SampleError),

    #[error("Library error: {0}")]
    Library(#[from] LibraryError),

    #[error("Pool error: {0}")]
    Pool(#[from] PoolError),

    #[error("Run error: {0}")]
    Run(#[from] RunError),

    #[error("Box/Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Barcode error: {0}")]
    Barcode(#[from] BarcodeError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound { entity_type: String, id: String },

    #[error("Duplicate entity: {entity_type} with {field}={value}")]
    Duplicate {
        entity_type: String,
        field: String,
        value: String,
    },

    #[error("Invalid state transition: cannot transition {entity} from {from} to {to}")]
    InvalidStateTransition {
        entity: String,
        from: String,
        to: String,
    },
}

/// Errors specific to Sample operations.
#[derive(Debug, Error)]
pub enum SampleError {
    #[error("Invalid sample name: {0}")]
    InvalidName(String),

    #[error("Sample {0} has not been extracted and cannot be used for library preparation")]
    NotExtracted(String),

    #[error("Sample {0} has failed QC and cannot proceed")]
    FailedQc(String),

    #[error("Sample {0} is archived and cannot be modified")]
    Archived(String),

    #[error("Invalid sample class: {0}")]
    InvalidClass(String),

    #[error("Parent sample {0} not found")]
    ParentNotFound(String),

    #[error("Invalid tissue origin: {0}")]
    InvalidTissueOrigin(String),

    #[error("Invalid tissue type: {0}")]
    InvalidTissueType(String),
}

/// Errors specific to Library operations.
#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("Invalid library design: {0}")]
    InvalidDesign(String),

    #[error("Library {0} requires an index for pooling")]
    MissingIndex(String),

    #[error("Invalid index sequence: {0}")]
    InvalidIndexSequence(String),

    #[error("Library {0} has already been exhausted (no volume remaining)")]
    Exhausted(String),

    #[error("Invalid kit type: {0}")]
    InvalidKitType(String),

    #[error("Library {0} is already in pool {1}")]
    AlreadyPooled(String, String),
}

/// Errors specific to Pool operations.
#[derive(Debug, Error)]
pub enum PoolError {
    #[error("Index collision detected: libraries {lib1} and {lib2} have indices with Hamming distance {distance} (minimum required: {required})")]
    IndexCollision {
        lib1: String,
        lib2: String,
        distance: u32,
        required: u32,
    },

    #[error("Pool {0} is empty - at least one library aliquot is required")]
    EmptyPool(String),

    #[error("Pool {0} exceeds maximum capacity of {1} libraries")]
    CapacityExceeded(String, usize),

    #[error("Incompatible library types in pool: {0} and {1}")]
    IncompatibleLibraryTypes(String, String),

    #[error("Pool {0} has already been sequenced and cannot be modified")]
    AlreadySequenced(String),

    #[error("Duplicate library in pool: {0}")]
    DuplicateLibrary(String),
}

/// Errors specific to Run/Sequencing operations.
#[derive(Debug, Error)]
pub enum RunError {
    #[error("Invalid sequencer: {0}")]
    InvalidSequencer(String),

    #[error("Run {0} is already complete")]
    AlreadyComplete(String),

    #[error("Run {0} has failed and cannot be resumed")]
    Failed(String),

    #[error("Invalid run parameters: {0}")]
    InvalidParameters(String),

    #[error("Container {0} is not compatible with sequencer {1}")]
    IncompatibleContainer(String, String),

    #[error("Run {0} is missing required QC metrics")]
    MissingQcMetrics(String),
}

/// Errors specific to Box/Storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Position {row}{col} is already occupied in box {box_name}")]
    PositionOccupied {
        box_name: String,
        row: char,
        col: u8,
    },

    #[error("Invalid position {row}{col} for box with dimensions {rows}x{cols}")]
    InvalidPosition {
        row: char,
        col: u8,
        rows: u8,
        cols: u8,
    },

    #[error("Box {0} is full")]
    BoxFull(String),

    #[error("Item {0} is not in box {1}")]
    ItemNotInBox(String, String),

    #[error("Cannot move items between incompatible storage types")]
    IncompatibleStorageTypes,
}

/// Errors specific to Barcode operations.
#[derive(Debug, Error)]
pub enum BarcodeError {
    #[error("Invalid barcode format: {0}")]
    InvalidFormat(String),

    #[error("Barcode {0} is already in use")]
    AlreadyInUse(String),

    #[error("Barcode {0} does not match expected pattern {1}")]
    PatternMismatch(String, String),

    #[error("Barcode cannot be empty")]
    Empty,

    #[error("Barcode {0} contains invalid characters")]
    InvalidCharacters(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_display() {
        let error = DomainError::NotFound {
            entity_type: "Sample".to_string(),
            id: "SAM123".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Entity not found: Sample with id SAM123"
        );
    }

    #[test]
    fn test_pool_error_index_collision() {
        let error = PoolError::IndexCollision {
            lib1: "LIB001".to_string(),
            lib2: "LIB002".to_string(),
            distance: 1,
            required: 3,
        };
        assert!(error.to_string().contains("Hamming distance"));
    }
}

