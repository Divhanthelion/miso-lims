//! Library entity - a sample prepared for sequencing.
//!
//! A Library represents the DNA/RNA after it has been prepared with
//! adapters and indices for sequencing on a specific platform.

use crate::value_objects::{Barcode, Concentration, DnaIndex, QcStatus, Volume};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::EntityId;

/// The design of the library (what the sequencing is targeting).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryDesign {
    /// Whole Genome Sequencing
    Wgs,
    /// Whole Exome Sequencing
    Wes,
    /// RNA Sequencing
    RnaSeq,
    /// Targeted Sequencing (panel)
    TargetedPanel,
    /// ChIP-Seq
    ChipSeq,
    /// ATAC-Seq
    AtacSeq,
    /// Methylation Sequencing
    Methylation,
    /// Single Cell RNA-Seq
    SingleCellRna,
    /// Single Cell ATAC-Seq
    SingleCellAtac,
    /// Custom/Other
    Custom(String),
}

impl std::fmt::Display for LibraryDesign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wgs => write!(f, "WGS"),
            Self::Wes => write!(f, "WES"),
            Self::RnaSeq => write!(f, "RNA-Seq"),
            Self::TargetedPanel => write!(f, "Targeted Panel"),
            Self::ChipSeq => write!(f, "ChIP-Seq"),
            Self::AtacSeq => write!(f, "ATAC-Seq"),
            Self::Methylation => write!(f, "Methylation"),
            Self::SingleCellRna => write!(f, "scRNA-Seq"),
            Self::SingleCellAtac => write!(f, "scATAC-Seq"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// The type of library (based on preparation method).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryType {
    /// Paired-end sequencing
    PairedEnd,
    /// Single-end sequencing
    SingleEnd,
    /// Mate-pair sequencing
    MatePair,
}

impl std::fmt::Display for LibraryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PairedEnd => write!(f, "Paired End"),
            Self::SingleEnd => write!(f, "Single End"),
            Self::MatePair => write!(f, "Mate Pair"),
        }
    }
}

/// A library prepared for sequencing.
///
/// Libraries are the "pivot point" between biology and technology in the LIMS.
/// They represent the sample after preparation with adapters and indices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Library {
    /// Unique identifier
    pub id: EntityId,
    /// Human-readable name
    pub name: String,
    /// Unique barcode for physical tracking
    pub barcode: Barcode,
    /// The sample this library was created from
    pub sample_id: EntityId,
    /// The project this library belongs to
    pub project_id: EntityId,
    /// Library description
    pub description: Option<String>,
    /// The library design (WGS, WES, RNA-Seq, etc.)
    pub design: LibraryDesign,
    /// The library type (paired-end, single-end)
    pub library_type: LibraryType,
    /// The platform/sequencer this library is designed for
    pub platform: String,
    /// The preparation kit used
    pub kit_name: Option<String>,
    /// The DNA index (barcode) for multiplexing
    pub index: Option<DnaIndex>,
    /// Insert size (fragment length) in base pairs
    pub insert_size: Option<u32>,
    /// Current volume
    pub volume: Option<Volume>,
    /// Current concentration
    pub concentration: Option<Concentration>,
    /// QC status
    pub qc_status: QcStatus,
    /// Number of PCR cycles used in preparation
    pub pcr_cycles: Option<u8>,
    /// Is this library low quality?
    pub low_quality: bool,
    /// Who created this record
    pub created_by: String,
    /// When this record was created
    pub created_at: DateTime<Utc>,
    /// When this record was last modified
    pub updated_at: DateTime<Utc>,
    /// Is this library archived/discarded?
    pub archived: bool,
}

impl Library {
    /// Creates a new library.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: EntityId,
        name: String,
        barcode: Barcode,
        sample_id: EntityId,
        project_id: EntityId,
        design: LibraryDesign,
        library_type: LibraryType,
        platform: String,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            barcode,
            sample_id,
            project_id,
            description: None,
            design,
            library_type,
            platform,
            kit_name: None,
            index: None,
            insert_size: None,
            volume: None,
            concentration: None,
            qc_status: QcStatus::NotReady,
            pcr_cycles: None,
            low_quality: false,
            created_by,
            created_at: now,
            updated_at: now,
            archived: false,
        }
    }

    /// Sets the DNA index for this library.
    pub fn set_index(&mut self, index: DnaIndex) {
        self.index = Some(index);
        self.updated_at = Utc::now();
    }

    /// Returns true if this library has an index assigned.
    pub fn has_index(&self) -> bool {
        self.index.is_some()
    }

    /// Returns true if this library can be pooled.
    pub fn can_pool(&self) -> bool {
        self.has_index()
            && self.qc_status.allows_progression()
            && !self.archived
            && !self.low_quality
    }

    /// Calculates the Hamming distance to another library's index.
    ///
    /// Returns None if either library lacks an index.
    pub fn index_distance(&self, other: &Library) -> Option<u32> {
        match (&self.index, &other.index) {
            (Some(a), Some(b)) => Some(a.hamming_distance(b)),
            _ => None,
        }
    }

    /// Archives this library.
    pub fn archive(&mut self) {
        self.archived = true;
        self.updated_at = Utc::now();
    }

    /// Sets the QC status.
    pub fn set_qc_status(&mut self, status: QcStatus) {
        self.qc_status = status;
        self.updated_at = Utc::now();
    }
}

/// A library aliquot - a portion of a library used for pooling.
///
/// This allows a single library to be pooled multiple times without
/// duplicating the library record itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryAliquot {
    /// Unique identifier
    pub id: EntityId,
    /// The parent library
    pub library_id: EntityId,
    /// Optional separate barcode for this aliquot
    pub barcode: Option<Barcode>,
    /// Volume of this aliquot
    pub volume: Option<Volume>,
    /// Concentration of this aliquot
    pub concentration: Option<Concentration>,
    /// When this aliquot was created
    pub created_at: DateTime<Utc>,
    /// Who created this aliquot
    pub created_by: String,
}

impl LibraryAliquot {
    /// Creates a new library aliquot.
    pub fn new(
        id: EntityId,
        library_id: EntityId,
        volume: Option<Volume>,
        concentration: Option<Concentration>,
        created_by: String,
    ) -> Self {
        Self {
            id,
            library_id,
            barcode: None,
            volume,
            concentration,
            created_at: Utc::now(),
            created_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::IndexFamily;

    #[test]
    fn test_library_creation() {
        let lib = Library::new(
            1,
            "LIB001".to_string(),
            Barcode::new("LIB-001").unwrap(),
            1,
            1,
            LibraryDesign::Wgs,
            LibraryType::PairedEnd,
            "Illumina".to_string(),
            "admin".to_string(),
        );
        assert!(!lib.has_index());
        assert!(!lib.can_pool()); // No index yet
    }

    #[test]
    fn test_library_pooling_eligibility() {
        let mut lib = Library::new(
            1,
            "LIB001".to_string(),
            Barcode::new("LIB-001").unwrap(),
            1,
            1,
            LibraryDesign::Wgs,
            LibraryType::PairedEnd,
            "Illumina".to_string(),
            "admin".to_string(),
        );

        // Add index
        lib.set_index(DnaIndex::single("A01", "ATCACG", IndexFamily::TruSeq).unwrap());
        assert!(lib.has_index());
        assert!(!lib.can_pool()); // QC not passed

        // Pass QC
        lib.set_qc_status(QcStatus::Passed);
        assert!(lib.can_pool());

        // Archive
        lib.archive();
        assert!(!lib.can_pool());
    }

    #[test]
    fn test_index_distance() {
        let mut lib1 = Library::new(
            1,
            "LIB001".to_string(),
            Barcode::new("LIB-001").unwrap(),
            1,
            1,
            LibraryDesign::Wgs,
            LibraryType::PairedEnd,
            "Illumina".to_string(),
            "admin".to_string(),
        );
        let mut lib2 = Library::new(
            2,
            "LIB002".to_string(),
            Barcode::new("LIB-002").unwrap(),
            2,
            1,
            LibraryDesign::Wgs,
            LibraryType::PairedEnd,
            "Illumina".to_string(),
            "admin".to_string(),
        );

        // No indices - should return None
        assert!(lib1.index_distance(&lib2).is_none());

        // Add indices
        lib1.set_index(DnaIndex::single("A01", "ATCACG", IndexFamily::TruSeq).unwrap());
        lib2.set_index(DnaIndex::single("A02", "TTAGGC", IndexFamily::TruSeq).unwrap());

        let distance = lib1.index_distance(&lib2).unwrap();
        assert!(distance > 0);
    }
}

