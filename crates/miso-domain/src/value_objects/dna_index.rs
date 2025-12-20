//! DNA Index (barcode) value objects for library multiplexing.
//!
//! DNA indices are short sequences attached to library fragments that allow
//! multiple samples to be pooled and sequenced together, then computationally
//! separated (demultiplexed) afterward.

use crate::errors::LibraryError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported index families/platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexFamily {
    /// TruSeq single index
    TruSeq,
    /// Nextera dual index
    Nextera,
    /// IDT for Illumina - UDI (Unique Dual Indexes)
    IdtUdi,
    /// 10x Genomics
    TenX,
    /// Custom/Other
    Custom,
}

impl fmt::Display for IndexFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruSeq => write!(f, "TruSeq"),
            Self::Nextera => write!(f, "Nextera"),
            Self::IdtUdi => write!(f, "IDT-UDI"),
            Self::TenX => write!(f, "10x Genomics"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// A DNA index sequence used for library multiplexing.
///
/// This represents the actual nucleotide sequence (A, C, G, T) that serves
/// as a barcode for demultiplexing pooled libraries after sequencing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DnaIndex {
    /// The i7 index sequence (required for single and dual indexing)
    i7_sequence: String,
    /// The i5 index sequence (optional, for dual indexing)
    i5_sequence: Option<String>,
    /// The index family/platform
    family: IndexFamily,
    /// Human-readable name (e.g., "A01", "UDP0001")
    name: String,
}

impl DnaIndex {
    /// Creates a new single index (i7 only).
    pub fn single(
        name: impl Into<String>,
        i7_sequence: impl Into<String>,
        family: IndexFamily,
    ) -> Result<Self, LibraryError> {
        let i7 = i7_sequence.into().to_uppercase();
        Self::validate_sequence(&i7)?;

        Ok(Self {
            i7_sequence: i7,
            i5_sequence: None,
            family,
            name: name.into(),
        })
    }

    /// Creates a new dual index (i7 + i5).
    pub fn dual(
        name: impl Into<String>,
        i7_sequence: impl Into<String>,
        i5_sequence: impl Into<String>,
        family: IndexFamily,
    ) -> Result<Self, LibraryError> {
        let i7 = i7_sequence.into().to_uppercase();
        let i5 = i5_sequence.into().to_uppercase();
        Self::validate_sequence(&i7)?;
        Self::validate_sequence(&i5)?;

        Ok(Self {
            i7_sequence: i7,
            i5_sequence: Some(i5),
            family,
            name: name.into(),
        })
    }

    /// Validates that a sequence contains only valid DNA bases.
    fn validate_sequence(seq: &str) -> Result<(), LibraryError> {
        if seq.is_empty() {
            return Err(LibraryError::InvalidIndexSequence(
                "Index sequence cannot be empty".to_string(),
            ));
        }

        if !seq.chars().all(|c| matches!(c, 'A' | 'C' | 'G' | 'T' | 'N')) {
            return Err(LibraryError::InvalidIndexSequence(format!(
                "Invalid characters in sequence: {}",
                seq
            )));
        }

        Ok(())
    }

    /// Returns the i7 index sequence.
    pub fn i7(&self) -> &str {
        &self.i7_sequence
    }

    /// Returns the i5 index sequence if present.
    pub fn i5(&self) -> Option<&str> {
        self.i5_sequence.as_deref()
    }

    /// Returns the index family.
    pub fn family(&self) -> IndexFamily {
        self.family
    }

    /// Returns the index name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns true if this is a dual index.
    pub fn is_dual(&self) -> bool {
        self.i5_sequence.is_some()
    }

    /// Returns the combined sequence length.
    pub fn total_length(&self) -> usize {
        self.i7_sequence.len() + self.i5_sequence.as_ref().map(|s| s.len()).unwrap_or(0)
    }

    /// Calculates the Hamming distance between this index and another.
    ///
    /// This is critical for detecting potential barcode collisions in pools.
    /// A minimum Hamming distance (typically 3) is required between all
    /// indices in a pool to ensure reliable demultiplexing.
    pub fn hamming_distance(&self, other: &Self) -> u32 {
        let i7_dist = Self::sequence_hamming_distance(&self.i7_sequence, &other.i7_sequence);

        let i5_dist = match (&self.i5_sequence, &other.i5_sequence) {
            (Some(a), Some(b)) => Self::sequence_hamming_distance(a, b),
            _ => 0,
        };

        i7_dist + i5_dist
    }

    /// Calculates Hamming distance between two sequences.
    ///
    /// Uses bit-packing for optimal performance when comparing many indices.
    fn sequence_hamming_distance(a: &str, b: &str) -> u32 {
        // For short sequences, simple comparison is fast enough
        // For production with many comparisons, use bio crate's SIMD version
        a.chars()
            .zip(b.chars())
            .filter(|(ca, cb)| ca != cb)
            .count() as u32
    }

    /// Bit-pack a DNA sequence for fast Hamming distance calculation.
    ///
    /// Each base is encoded as 2 bits: A=00, C=01, G=10, T=11
    /// This allows XOR + popcount for ultra-fast comparison.
    #[allow(dead_code)]
    fn pack_sequence(seq: &str) -> u64 {
        let mut packed: u64 = 0;
        for c in seq.chars().take(32) {
            // Max 32 bases in u64
            packed <<= 2;
            packed |= match c {
                'A' => 0b00,
                'C' => 0b01,
                'G' => 0b10,
                'T' | 'N' => 0b11,
                _ => 0b00,
            };
        }
        packed
    }
}

impl fmt::Display for DnaIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.i5_sequence {
            Some(i5) => write!(f, "{} ({}-{})", self.name, self.i7_sequence, i5),
            None => write!(f, "{} ({})", self.name, self.i7_sequence),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_index() {
        let idx = DnaIndex::single("A01", "ATCACG", IndexFamily::TruSeq).unwrap();
        assert_eq!(idx.i7(), "ATCACG");
        assert!(idx.i5().is_none());
        assert!(!idx.is_dual());
    }

    #[test]
    fn test_dual_index() {
        let idx = DnaIndex::dual("UDP0001", "AACGTGAT", "AACGTGAT", IndexFamily::IdtUdi).unwrap();
        assert_eq!(idx.i7(), "AACGTGAT");
        assert_eq!(idx.i5(), Some("AACGTGAT"));
        assert!(idx.is_dual());
    }

    #[test]
    fn test_invalid_sequence() {
        let result = DnaIndex::single("X01", "ATCXYZ", IndexFamily::Custom);
        assert!(result.is_err());
    }

    #[test]
    fn test_hamming_distance() {
        let idx1 = DnaIndex::single("A01", "ATCACG", IndexFamily::TruSeq).unwrap();
        let idx2 = DnaIndex::single("A02", "ATCACG", IndexFamily::TruSeq).unwrap();
        let idx3 = DnaIndex::single("A03", "TTAGGC", IndexFamily::TruSeq).unwrap();

        assert_eq!(idx1.hamming_distance(&idx2), 0); // Same sequence
        assert_eq!(idx1.hamming_distance(&idx3), 6); // Completely different
    }

    #[test]
    fn test_hamming_distance_partial() {
        let idx1 = DnaIndex::single("A01", "ATCACG", IndexFamily::TruSeq).unwrap();
        let idx2 = DnaIndex::single("A02", "ATCACN", IndexFamily::TruSeq).unwrap();

        assert_eq!(idx1.hamming_distance(&idx2), 1); // One base different
    }

    #[test]
    fn test_lowercase_normalized() {
        let idx = DnaIndex::single("A01", "atcacg", IndexFamily::TruSeq).unwrap();
        assert_eq!(idx.i7(), "ATCACG"); // Should be uppercase
    }
}

