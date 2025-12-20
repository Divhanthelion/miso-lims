//! Sample entity - the core biological entity in the LIMS.
//!
//! MISO supports two modes:
//! - **Plain Sample Mode**: Flat hierarchy (Sample -> Library -> Pool)
//! - **Detailed Sample Mode**: Deep hierarchy (Identity -> Tissue -> Stock -> Aliquot)

use crate::value_objects::{Barcode, Concentration, QcStatus, Volume};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::EntityId;

/// The class/type of a sample in the hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleClass {
    // Plain mode classes
    /// A basic sample (Plain mode)
    Plain,

    // Detailed mode classes (hierarchical)
    /// The identity of the source organism/patient
    Identity,
    /// A tissue sample from the identity
    Tissue,
    /// Tissue processing (e.g., slide, curls)
    TissueProcessing,
    /// Extracted stock (gDNA, RNA, etc.)
    Stock,
    /// An aliquot of the stock
    Aliquot,

    // Special classes
    /// Single cell sample
    SingleCell,
    /// Whole transcriptome
    WholeTranscriptome,
}

impl SampleClass {
    /// Returns true if this is a detailed sample class.
    pub fn is_detailed(&self) -> bool {
        !matches!(self, Self::Plain)
    }

    /// Returns the expected parent class (if any).
    pub fn expected_parent(&self) -> Option<SampleClass> {
        match self {
            Self::Plain => None,
            Self::Identity => None,
            Self::Tissue => Some(Self::Identity),
            Self::TissueProcessing => Some(Self::Tissue),
            Self::Stock => Some(Self::TissueProcessing),
            Self::Aliquot => Some(Self::Stock),
            Self::SingleCell => Some(Self::Tissue),
            Self::WholeTranscriptome => Some(Self::Aliquot),
        }
    }

    /// Returns true if a library can be created from this sample class.
    pub fn can_create_library(&self) -> bool {
        matches!(self, Self::Plain | Self::Aliquot | Self::WholeTranscriptome)
    }
}

impl std::fmt::Display for SampleClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain => write!(f, "Plain Sample"),
            Self::Identity => write!(f, "Identity"),
            Self::Tissue => write!(f, "Tissue"),
            Self::TissueProcessing => write!(f, "Tissue Processing"),
            Self::Stock => write!(f, "Stock"),
            Self::Aliquot => write!(f, "Aliquot"),
            Self::SingleCell => write!(f, "Single Cell"),
            Self::WholeTranscriptome => write!(f, "Whole Transcriptome"),
        }
    }
}

/// Additional data for plain samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlainSampleData {
    /// The scientific name of the organism
    pub scientific_name: String,
    /// Sample type description
    pub sample_type: Option<String>,
}

/// Additional data for detailed samples.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailedSampleData {
    /// Parent sample ID in the hierarchy
    pub parent_id: Option<EntityId>,
    /// The specific class within the detailed hierarchy
    pub sample_class: SampleClass,
    /// External identifier (e.g., patient ID for Identity)
    pub external_name: Option<String>,
    /// Tissue origin (anatomical source)
    pub tissue_origin: Option<String>,
    /// Tissue type (e.g., Primary Tumor, Metastatic)
    pub tissue_type: Option<String>,
    /// Time point for longitudinal studies
    pub time_point: Option<String>,
    /// Group ID for related samples
    pub group_id: Option<String>,
    /// Group description
    pub group_description: Option<String>,
    /// Passage number (for cell lines)
    pub passage: Option<i32>,
    /// Analyte type for stocks (DNA, RNA, etc.)
    pub analyte_type: Option<String>,
    /// Purpose of the sample
    pub purpose: Option<String>,
}

/// Polymorphic container for sample-type-specific data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum SampleDetails {
    /// Plain sample mode data
    Plain(PlainSampleData),
    /// Detailed sample mode data
    Detailed(DetailedSampleData),
}

impl SampleDetails {
    /// Returns the sample class.
    pub fn sample_class(&self) -> SampleClass {
        match self {
            Self::Plain(_) => SampleClass::Plain,
            Self::Detailed(d) => d.sample_class.clone(),
        }
    }

    /// Returns true if libraries can be created from this sample.
    pub fn can_create_library(&self) -> bool {
        self.sample_class().can_create_library()
    }

    /// Returns the parent ID for detailed samples.
    pub fn parent_id(&self) -> Option<EntityId> {
        match self {
            Self::Plain(_) => None,
            Self::Detailed(d) => d.parent_id,
        }
    }
}

/// A sample in the LIMS - the core biological entity.
///
/// Samples represent biological material at various stages of processing,
/// from the original source (Identity/Patient) through extraction and
/// preparation for sequencing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    /// Unique identifier
    pub id: EntityId,
    /// Human-readable name
    pub name: String,
    /// Unique barcode for physical tracking
    pub barcode: Barcode,
    /// The project this sample belongs to
    pub project_id: EntityId,
    /// Sample description
    pub description: Option<String>,
    /// Sample-type-specific details (Plain or Detailed)
    pub details: SampleDetails,
    /// Current volume (if applicable)
    pub volume: Option<Volume>,
    /// Current concentration (if applicable)
    pub concentration: Option<Concentration>,
    /// QC status
    pub qc_status: QcStatus,
    /// When the sample was received/created
    pub received_at: Option<DateTime<Utc>>,
    /// Who created this record
    pub created_by: String,
    /// When this record was created
    pub created_at: DateTime<Utc>,
    /// When this record was last modified
    pub updated_at: DateTime<Utc>,
    /// Is this sample archived/discarded?
    pub archived: bool,
}

impl Sample {
    /// Creates a new plain sample.
    pub fn new_plain(
        id: EntityId,
        name: String,
        barcode: Barcode,
        project_id: EntityId,
        scientific_name: String,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            barcode,
            project_id,
            description: None,
            details: SampleDetails::Plain(PlainSampleData {
                scientific_name,
                sample_type: None,
            }),
            volume: None,
            concentration: None,
            qc_status: QcStatus::NotReady,
            received_at: Some(now),
            created_by,
            created_at: now,
            updated_at: now,
            archived: false,
        }
    }

    /// Returns the sample class.
    pub fn sample_class(&self) -> SampleClass {
        self.details.sample_class()
    }

    /// Returns true if this sample is in detailed mode.
    pub fn is_detailed(&self) -> bool {
        matches!(self.details, SampleDetails::Detailed(_))
    }

    /// Returns true if a library can be created from this sample.
    pub fn can_create_library(&self) -> bool {
        self.details.can_create_library() && self.qc_status.allows_progression() && !self.archived
    }

    /// Returns the parent sample ID (for detailed samples).
    pub fn parent_id(&self) -> Option<EntityId> {
        self.details.parent_id()
    }

    /// Archives this sample (marks as discarded/unavailable).
    pub fn archive(&mut self) {
        self.archived = true;
        self.updated_at = Utc::now();
    }

    /// Updates the QC status.
    pub fn set_qc_status(&mut self, status: QcStatus) {
        self.qc_status = status;
        self.updated_at = Utc::now();
    }

    /// Withdraws volume from this sample.
    ///
    /// Returns `Ok(())` if successful, or an error if insufficient volume.
    pub fn withdraw_volume(&mut self, amount: Volume) -> Result<(), &'static str> {
        match &mut self.volume {
            Some(v) => {
                if let Some(remaining) = v.subtract(amount) {
                    *v = remaining;
                    self.updated_at = Utc::now();
                    Ok(())
                } else {
                    Err("Insufficient volume")
                }
            }
            None => Err("Sample has no tracked volume"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_class_hierarchy() {
        assert!(SampleClass::Identity.is_detailed());
        assert!(!SampleClass::Plain.is_detailed());
        assert_eq!(SampleClass::Tissue.expected_parent(), Some(SampleClass::Identity));
        assert!(SampleClass::Aliquot.can_create_library());
        assert!(!SampleClass::Tissue.can_create_library());
    }

    #[test]
    fn test_plain_sample() {
        let sample = Sample::new_plain(
            1,
            "SAM001".to_string(),
            Barcode::new("SAM-001").unwrap(),
            1,
            "Homo sapiens".to_string(),
            "admin".to_string(),
        );
        assert!(!sample.is_detailed());
        assert_eq!(sample.sample_class(), SampleClass::Plain);
    }

    #[test]
    fn test_sample_library_eligibility() {
        let mut sample = Sample::new_plain(
            1,
            "SAM001".to_string(),
            Barcode::new("SAM-001").unwrap(),
            1,
            "Homo sapiens".to_string(),
            "admin".to_string(),
        );

        // Fresh sample cannot create library (QC not passed)
        assert!(!sample.can_create_library());

        // After QC passes
        sample.set_qc_status(QcStatus::Passed);
        assert!(sample.can_create_library());

        // After archiving
        sample.archive();
        assert!(!sample.can_create_library());
    }
}

