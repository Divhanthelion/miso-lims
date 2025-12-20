//! Pool entity - a collection of library aliquots for multiplexed sequencing.
//!
//! Pools allow multiple libraries to be sequenced together on a single
//! flow cell lane, with computational demultiplexing afterward.

use crate::errors::PoolError;
use crate::value_objects::{Barcode, Concentration, QcStatus, Volume};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{EntityId, Library};

/// A pool element - a library aliquot in a pool with its proportion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoolElement {
    /// The library aliquot ID
    pub library_aliquot_id: EntityId,
    /// The library ID (for quick access)
    pub library_id: EntityId,
    /// Volume contributed to the pool
    pub volume: Option<Volume>,
    /// Proportion/percentage of the pool (0.0-1.0)
    pub proportion: Option<f64>,
}

/// A pool of library aliquots for multiplexed sequencing.
///
/// Pools are the unit that is loaded onto a sequencer. They must contain
/// libraries with compatible indices (sufficient Hamming distance).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pool {
    /// Unique identifier
    pub id: EntityId,
    /// Human-readable name
    pub name: String,
    /// Unique barcode for physical tracking
    pub barcode: Barcode,
    /// Pool description
    pub description: Option<String>,
    /// The library aliquots in this pool
    pub elements: Vec<PoolElement>,
    /// Target concentration for loading
    pub concentration: Option<Concentration>,
    /// Total volume of the pool
    pub volume: Option<Volume>,
    /// QC status
    pub qc_status: QcStatus,
    /// Platform this pool is designed for
    pub platform: String,
    /// Has this pool been sequenced?
    pub sequenced: bool,
    /// Who created this record
    pub created_by: String,
    /// When this record was created
    pub created_at: DateTime<Utc>,
    /// When this record was last modified
    pub updated_at: DateTime<Utc>,
}

impl Pool {
    /// Creates a new empty pool.
    pub fn new(
        id: EntityId,
        name: String,
        barcode: Barcode,
        platform: String,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            barcode,
            description: None,
            elements: Vec::new(),
            concentration: None,
            volume: None,
            qc_status: QcStatus::NotReady,
            platform,
            sequenced: false,
            created_by,
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns the number of library aliquots in this pool.
    pub fn size(&self) -> usize {
        self.elements.len()
    }

    /// Returns true if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Adds a library aliquot to the pool.
    ///
    /// Note: Index collision checking should be done before calling this.
    pub fn add_element(&mut self, element: PoolElement) -> Result<(), PoolError> {
        // Check for duplicates
        if self.elements.iter().any(|e| e.library_id == element.library_id) {
            return Err(PoolError::DuplicateLibrary(element.library_id.to_string()));
        }

        self.elements.push(element);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Removes a library aliquot from the pool.
    pub fn remove_element(&mut self, library_aliquot_id: EntityId) -> Result<(), PoolError> {
        if self.sequenced {
            return Err(PoolError::AlreadySequenced(self.name.clone()));
        }

        let len_before = self.elements.len();
        self.elements.retain(|e| e.library_aliquot_id != library_aliquot_id);

        if self.elements.len() == len_before {
            return Err(PoolError::DuplicateLibrary(library_aliquot_id.to_string()));
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Validates index compatibility for all libraries in the pool.
    ///
    /// Returns a list of index collision errors if any pairs have
    /// insufficient Hamming distance.
    pub fn validate_indices(
        &self,
        libraries: &[Library],
        min_distance: u32,
    ) -> Vec<PoolError> {
        let mut errors = Vec::new();

        // Build a map of library_id to library
        let lib_map: std::collections::HashMap<_, _> = libraries
            .iter()
            .map(|l| (l.id, l))
            .collect();

        // Check all pairs
        for (i, elem1) in self.elements.iter().enumerate() {
            for elem2 in self.elements.iter().skip(i + 1) {
                if let (Some(lib1), Some(lib2)) = (
                    lib_map.get(&elem1.library_id),
                    lib_map.get(&elem2.library_id),
                ) {
                    if let Some(distance) = lib1.index_distance(lib2) {
                        if distance < min_distance {
                            errors.push(PoolError::IndexCollision {
                                lib1: lib1.name.clone(),
                                lib2: lib2.name.clone(),
                                distance,
                                required: min_distance,
                            });
                        }
                    }
                }
            }
        }

        errors
    }

    /// Returns true if this pool can be sequenced.
    pub fn can_sequence(&self) -> bool {
        !self.is_empty() && self.qc_status.allows_progression() && !self.sequenced
    }

    /// Marks the pool as sequenced.
    pub fn mark_sequenced(&mut self) {
        self.sequenced = true;
        self.updated_at = Utc::now();
    }

    /// Sets the QC status.
    pub fn set_qc_status(&mut self, status: QcStatus) {
        self.qc_status = status;
        self.updated_at = Utc::now();
    }

    /// Returns the library IDs in this pool.
    pub fn library_ids(&self) -> Vec<EntityId> {
        self.elements.iter().map(|e| e.library_id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::LibraryDesign;
    use crate::entities::LibraryType;
    use crate::value_objects::IndexFamily;
    use crate::value_objects::DnaIndex;

    fn create_test_library(id: EntityId, index_seq: &str) -> Library {
        let mut lib = Library::new(
            id,
            format!("LIB{:03}", id),
            Barcode::new(format!("LIB-{:03}", id)).unwrap(),
            1,
            1,
            LibraryDesign::Wgs,
            LibraryType::PairedEnd,
            "Illumina".to_string(),
            "admin".to_string(),
        );
        lib.set_index(DnaIndex::single(format!("A{:02}", id), index_seq, IndexFamily::TruSeq).unwrap());
        lib
    }

    #[test]
    fn test_pool_creation() {
        let pool = Pool::new(
            1,
            "POOL001".to_string(),
            Barcode::new("POOL-001").unwrap(),
            "Illumina".to_string(),
            "admin".to_string(),
        );
        assert!(pool.is_empty());
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_pool_add_element() {
        let mut pool = Pool::new(
            1,
            "POOL001".to_string(),
            Barcode::new("POOL-001").unwrap(),
            "Illumina".to_string(),
            "admin".to_string(),
        );

        pool.add_element(PoolElement {
            library_aliquot_id: 1,
            library_id: 1,
            volume: None,
            proportion: Some(0.5),
        }).unwrap();

        assert_eq!(pool.size(), 1);
    }

    #[test]
    fn test_pool_duplicate_detection() {
        let mut pool = Pool::new(
            1,
            "POOL001".to_string(),
            Barcode::new("POOL-001").unwrap(),
            "Illumina".to_string(),
            "admin".to_string(),
        );

        pool.add_element(PoolElement {
            library_aliquot_id: 1,
            library_id: 1,
            volume: None,
            proportion: None,
        }).unwrap();

        // Try to add the same library again
        let result = pool.add_element(PoolElement {
            library_aliquot_id: 2,
            library_id: 1, // Same library
            volume: None,
            proportion: None,
        });

        assert!(matches!(result, Err(PoolError::DuplicateLibrary(_))));
    }

    #[test]
    fn test_index_collision_detection() {
        let mut pool = Pool::new(
            1,
            "POOL001".to_string(),
            Barcode::new("POOL-001").unwrap(),
            "Illumina".to_string(),
            "admin".to_string(),
        );

        // Add two libraries with similar indices
        pool.add_element(PoolElement {
            library_aliquot_id: 1,
            library_id: 1,
            volume: None,
            proportion: None,
        }).unwrap();

        pool.add_element(PoolElement {
            library_aliquot_id: 2,
            library_id: 2,
            volume: None,
            proportion: None,
        }).unwrap();

        // Create libraries with indices that differ by only 1 base
        let lib1 = create_test_library(1, "ATCACG");
        let lib2 = create_test_library(2, "ATCACN"); // Only 1 base different

        let errors = pool.validate_indices(&[lib1, lib2], 3);
        assert!(!errors.is_empty());
        assert!(matches!(errors[0], PoolError::IndexCollision { .. }));
    }

    #[test]
    fn test_index_no_collision() {
        let mut pool = Pool::new(
            1,
            "POOL001".to_string(),
            Barcode::new("POOL-001").unwrap(),
            "Illumina".to_string(),
            "admin".to_string(),
        );

        pool.add_element(PoolElement {
            library_aliquot_id: 1,
            library_id: 1,
            volume: None,
            proportion: None,
        }).unwrap();

        pool.add_element(PoolElement {
            library_aliquot_id: 2,
            library_id: 2,
            volume: None,
            proportion: None,
        }).unwrap();

        // Create libraries with very different indices
        let lib1 = create_test_library(1, "ATCACG");
        let lib2 = create_test_library(2, "TTAGGC"); // Completely different

        let errors = pool.validate_indices(&[lib1, lib2], 3);
        assert!(errors.is_empty());
    }
}

