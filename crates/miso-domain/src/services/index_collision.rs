//! Index collision checking service.
//!
//! Detects potential barcode collisions in pools by calculating
//! Hamming distances between all index pairs.

use crate::entities::Library;
use crate::errors::PoolError;
use crate::value_objects::DnaIndex;

/// Configuration for index collision checking.
#[derive(Debug, Clone)]
pub struct CollisionCheckConfig {
    /// Minimum Hamming distance required between any two indices
    pub min_distance: u32,
    /// Whether to check i7 only or both i7 and i5
    pub check_dual_index: bool,
}

impl Default for CollisionCheckConfig {
    fn default() -> Self {
        Self {
            min_distance: 3,
            check_dual_index: true,
        }
    }
}

impl CollisionCheckConfig {
    /// Creates a strict configuration (distance >= 3).
    pub fn strict() -> Self {
        Self {
            min_distance: 3,
            check_dual_index: true,
        }
    }

    /// Creates a relaxed configuration (distance >= 2).
    pub fn relaxed() -> Self {
        Self {
            min_distance: 2,
            check_dual_index: true,
        }
    }

    /// Creates a single-index only configuration.
    pub fn single_index_only() -> Self {
        Self {
            min_distance: 3,
            check_dual_index: false,
        }
    }
}

/// A detected collision between two indices.
#[derive(Debug, Clone)]
pub struct IndexCollision {
    /// First library name
    pub library1: String,
    /// Second library name
    pub library2: String,
    /// First index
    pub index1: DnaIndex,
    /// Second index
    pub index2: DnaIndex,
    /// Calculated Hamming distance
    pub distance: u32,
    /// Required minimum distance
    pub required_distance: u32,
}

impl IndexCollision {
    /// Converts to a PoolError.
    pub fn to_error(&self) -> PoolError {
        PoolError::IndexCollision {
            lib1: self.library1.clone(),
            lib2: self.library2.clone(),
            distance: self.distance,
            required: self.required_distance,
        }
    }
}

/// Service for checking index collisions in library pools.
#[derive(Debug, Clone, Default)]
pub struct IndexCollisionChecker {
    config: CollisionCheckConfig,
}

impl IndexCollisionChecker {
    /// Creates a new checker with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new checker with custom configuration.
    pub fn with_config(config: CollisionCheckConfig) -> Self {
        Self { config }
    }

    /// Checks a list of libraries for index collisions.
    ///
    /// Returns a list of all detected collisions.
    pub fn check_libraries(&self, libraries: &[Library]) -> Vec<IndexCollision> {
        let mut collisions = Vec::new();

        // Filter to libraries with indices
        let indexed: Vec<_> = libraries
            .iter()
            .filter_map(|lib| lib.index.as_ref().map(|idx| (lib, idx)))
            .collect();

        // Check all pairs
        for (i, (lib1, idx1)) in indexed.iter().enumerate() {
            for (lib2, idx2) in indexed.iter().skip(i + 1) {
                let distance = idx1.hamming_distance(idx2);

                if distance < self.config.min_distance {
                    collisions.push(IndexCollision {
                        library1: lib1.name.clone(),
                        library2: lib2.name.clone(),
                        index1: (*idx1).clone(),
                        index2: (*idx2).clone(),
                        distance,
                        required_distance: self.config.min_distance,
                    });
                }
            }
        }

        collisions
    }

    /// Checks a list of indices directly (without library context).
    pub fn check_indices(&self, indices: &[(String, DnaIndex)]) -> Vec<IndexCollision> {
        let mut collisions = Vec::new();

        for (i, (name1, idx1)) in indices.iter().enumerate() {
            for (name2, idx2) in indices.iter().skip(i + 1) {
                let distance = idx1.hamming_distance(idx2);

                if distance < self.config.min_distance {
                    collisions.push(IndexCollision {
                        library1: name1.clone(),
                        library2: name2.clone(),
                        index1: idx1.clone(),
                        index2: idx2.clone(),
                        distance,
                        required_distance: self.config.min_distance,
                    });
                }
            }
        }

        collisions
    }

    /// Checks if a new index can be added to an existing set without collision.
    pub fn can_add_index(
        &self,
        existing: &[(String, DnaIndex)],
        new_name: &str,
        new_index: &DnaIndex,
    ) -> Result<(), IndexCollision> {
        for (name, idx) in existing {
            let distance = idx.hamming_distance(new_index);

            if distance < self.config.min_distance {
                return Err(IndexCollision {
                    library1: name.clone(),
                    library2: new_name.to_string(),
                    index1: idx.clone(),
                    index2: new_index.clone(),
                    distance,
                    required_distance: self.config.min_distance,
                });
            }
        }

        Ok(())
    }

    /// Returns the configuration.
    pub fn config(&self) -> &CollisionCheckConfig {
        &self.config
    }

    /// Calculates a distance matrix for all index pairs.
    ///
    /// Useful for visualization or detailed analysis.
    pub fn distance_matrix(&self, indices: &[DnaIndex]) -> Vec<Vec<u32>> {
        let n = indices.len();
        let mut matrix = vec![vec![0u32; n]; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let dist = indices[i].hamming_distance(&indices[j]);
                matrix[i][j] = dist;
                matrix[j][i] = dist;
            }
        }

        matrix
    }

    /// Finds the minimum distance between any two indices in a set.
    pub fn min_distance(&self, indices: &[DnaIndex]) -> Option<u32> {
        if indices.len() < 2 {
            return None;
        }

        let mut min = u32::MAX;
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                let dist = indices[i].hamming_distance(&indices[j]);
                if dist < min {
                    min = dist;
                }
            }
        }

        Some(min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{LibraryDesign, LibraryType};
    use crate::value_objects::{Barcode, IndexFamily};

    fn create_library_with_index(id: i32, name: &str, index_seq: &str) -> Library {
        let mut lib = Library::new(
            id,
            name.to_string(),
            Barcode::new(format!("LIB-{:03}", id)).unwrap(),
            1,
            1,
            LibraryDesign::Wgs,
            LibraryType::PairedEnd,
            "Illumina".to_string(),
            "admin".to_string(),
        );
        lib.set_index(DnaIndex::single(name, index_seq, IndexFamily::TruSeq).unwrap());
        lib
    }

    #[test]
    fn test_no_collision() {
        let checker = IndexCollisionChecker::new();
        let libraries = vec![
            create_library_with_index(1, "LIB1", "ATCACG"),
            create_library_with_index(2, "LIB2", "TTAGGC"),
            create_library_with_index(3, "LIB3", "CGATGT"),
        ];

        let collisions = checker.check_libraries(&libraries);
        assert!(collisions.is_empty());
    }

    #[test]
    fn test_collision_detected() {
        let checker = IndexCollisionChecker::new();
        let libraries = vec![
            create_library_with_index(1, "LIB1", "ATCACG"),
            create_library_with_index(2, "LIB2", "ATCACN"), // Only 1 base different
        ];

        let collisions = checker.check_libraries(&libraries);
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].distance, 1);
    }

    #[test]
    fn test_can_add_index() {
        let checker = IndexCollisionChecker::new();

        let existing = vec![(
            "LIB1".to_string(),
            DnaIndex::single("A01", "ATCACG", IndexFamily::TruSeq).unwrap(),
        )];

        // This should succeed (different enough)
        let new_idx = DnaIndex::single("A02", "TTAGGC", IndexFamily::TruSeq).unwrap();
        assert!(checker.can_add_index(&existing, "LIB2", &new_idx).is_ok());

        // This should fail (too similar)
        let similar_idx = DnaIndex::single("A03", "ATCACN", IndexFamily::TruSeq).unwrap();
        assert!(checker.can_add_index(&existing, "LIB3", &similar_idx).is_err());
    }

    #[test]
    fn test_min_distance() {
        let checker = IndexCollisionChecker::new();
        let indices = vec![
            DnaIndex::single("A01", "ATCACG", IndexFamily::TruSeq).unwrap(),
            DnaIndex::single("A02", "TTAGGC", IndexFamily::TruSeq).unwrap(),
            DnaIndex::single("A03", "CGATGT", IndexFamily::TruSeq).unwrap(),
        ];

        let min = checker.min_distance(&indices).unwrap();
        assert!(min >= 3); // TruSeq indices are designed to have sufficient distance
    }

    #[test]
    fn test_relaxed_config() {
        let checker = IndexCollisionChecker::with_config(CollisionCheckConfig::relaxed());

        let indices = vec![
            ("LIB1".to_string(), DnaIndex::single("A01", "ATCACG", IndexFamily::TruSeq).unwrap()),
            ("LIB2".to_string(), DnaIndex::single("A02", "ATCACC", IndexFamily::TruSeq).unwrap()), // 2 bases different
        ];

        let collisions = checker.check_indices(&indices);
        // With min_distance=2, a distance of 2 should not collide
        assert!(collisions.is_empty());
    }
}

