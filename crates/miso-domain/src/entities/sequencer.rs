//! Sequencer/Instrument entities.
//!
//! Represents the sequencing instruments and their associated
//! container types (flow cells, chips, etc.).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::EntityId;

/// The sequencing platform/manufacturer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    /// Illumina (HiSeq, NovaSeq, MiSeq, NextSeq, etc.)
    Illumina,
    /// Oxford Nanopore Technologies (MinION, PromethION, etc.)
    OxfordNanopore,
    /// Pacific Biosciences (Sequel, Revio)
    PacBio,
    /// Ion Torrent
    IonTorrent,
    /// Element Biosciences
    Element,
    /// MGI/BGI
    Mgi,
    /// Ultima Genomics
    Ultima,
    /// Other/Custom
    Other,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Illumina => write!(f, "Illumina"),
            Self::OxfordNanopore => write!(f, "Oxford Nanopore"),
            Self::PacBio => write!(f, "PacBio"),
            Self::IonTorrent => write!(f, "Ion Torrent"),
            Self::Element => write!(f, "Element"),
            Self::Mgi => write!(f, "MGI"),
            Self::Ultima => write!(f, "Ultima"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// The instrument model.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstrumentModel {
    /// Platform
    pub platform: Platform,
    /// Model name (e.g., "NovaSeq 6000", "PromethION 48")
    pub name: String,
    /// Number of partitions (lanes/cells) per run
    pub partitions: u8,
    /// Description of the model's capabilities
    pub description: Option<String>,
}

impl InstrumentModel {
    /// Creates a new instrument model.
    pub fn new(platform: Platform, name: String, partitions: u8) -> Self {
        Self {
            platform,
            name,
            partitions,
            description: None,
        }
    }

    /// Common Illumina NovaSeq 6000 model.
    pub fn novaseq_6000() -> Self {
        Self::new(Platform::Illumina, "NovaSeq 6000".to_string(), 4)
    }

    /// Common Illumina NovaSeq X model.
    pub fn novaseq_x() -> Self {
        Self::new(Platform::Illumina, "NovaSeq X".to_string(), 2)
    }

    /// Common Illumina MiSeq model.
    pub fn miseq() -> Self {
        Self::new(Platform::Illumina, "MiSeq".to_string(), 1)
    }

    /// Common Illumina NextSeq 2000 model.
    pub fn nextseq_2000() -> Self {
        Self::new(Platform::Illumina, "NextSeq 2000".to_string(), 1)
    }

    /// Common Oxford Nanopore PromethION model.
    pub fn promethion() -> Self {
        Self::new(Platform::OxfordNanopore, "PromethION 48".to_string(), 48)
    }
}

impl std::fmt::Display for InstrumentModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A container model (flow cell type).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContainerModel {
    /// Unique identifier
    pub id: EntityId,
    /// Name of the container type (e.g., "S4 Flow Cell", "Flongle")
    pub name: String,
    /// Compatible platform
    pub platform: Platform,
    /// Number of partitions (lanes/cells)
    pub partitions: u8,
    /// Description
    pub description: Option<String>,
}

impl ContainerModel {
    /// Creates a new container model.
    pub fn new(
        id: EntityId,
        name: String,
        platform: Platform,
        partitions: u8,
    ) -> Self {
        Self {
            id,
            name,
            platform,
            partitions,
            description: None,
        }
    }
}

/// The operational status of a sequencer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SequencerStatus {
    /// Operational and available
    #[default]
    Available,
    /// Currently running a sequence
    Running,
    /// Under maintenance
    Maintenance,
    /// Out of service
    OutOfService,
    /// Retired/decommissioned
    Retired,
}

impl SequencerStatus {
    /// Returns true if the sequencer can accept new runs.
    pub fn can_run(&self) -> bool {
        matches!(self, Self::Available)
    }
}

impl std::fmt::Display for SequencerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Available => write!(f, "Available"),
            Self::Running => write!(f, "Running"),
            Self::Maintenance => write!(f, "Under Maintenance"),
            Self::OutOfService => write!(f, "Out of Service"),
            Self::Retired => write!(f, "Retired"),
        }
    }
}

/// A physical sequencing instrument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sequencer {
    /// Unique identifier
    pub id: EntityId,
    /// Name/label of this specific instrument
    pub name: String,
    /// Serial number
    pub serial_number: Option<String>,
    /// The model of this instrument
    pub model: InstrumentModel,
    /// Current operational status
    pub status: SequencerStatus,
    /// Location in the facility
    pub location: Option<String>,
    /// IP address for Run Scanner monitoring
    pub ip_address: Option<String>,
    /// Date of purchase/installation
    pub date_commissioned: Option<DateTime<Utc>>,
    /// Date of last service
    pub last_service_date: Option<DateTime<Utc>>,
    /// When this record was created
    pub created_at: DateTime<Utc>,
    /// When this record was last modified
    pub updated_at: DateTime<Utc>,
}

impl Sequencer {
    /// Creates a new sequencer.
    pub fn new(
        id: EntityId,
        name: String,
        model: InstrumentModel,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            serial_number: None,
            model,
            status: SequencerStatus::Available,
            location: None,
            ip_address: None,
            date_commissioned: None,
            last_service_date: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns the platform of this sequencer.
    pub fn platform(&self) -> Platform {
        self.model.platform
    }

    /// Returns the number of partitions per run.
    pub fn num_partitions(&self) -> u8 {
        self.model.partitions
    }

    /// Returns true if this sequencer can accept new runs.
    pub fn can_run(&self) -> bool {
        self.status.can_run()
    }

    /// Sets the sequencer as running.
    pub fn start_run(&mut self) {
        self.status = SequencerStatus::Running;
        self.updated_at = Utc::now();
    }

    /// Sets the sequencer as available.
    pub fn complete_run(&mut self) {
        self.status = SequencerStatus::Available;
        self.updated_at = Utc::now();
    }

    /// Sets the sequencer to maintenance mode.
    pub fn start_maintenance(&mut self) {
        self.status = SequencerStatus::Maintenance;
        self.updated_at = Utc::now();
    }

    /// Records a service date.
    pub fn record_service(&mut self) {
        self.last_service_date = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequencer_creation() {
        let seq = Sequencer::new(
            1,
            "NovaSeq01".to_string(),
            InstrumentModel::novaseq_6000(),
        );
        assert!(seq.can_run());
        assert_eq!(seq.platform(), Platform::Illumina);
        assert_eq!(seq.num_partitions(), 4);
    }

    #[test]
    fn test_sequencer_lifecycle() {
        let mut seq = Sequencer::new(
            1,
            "NovaSeq01".to_string(),
            InstrumentModel::novaseq_6000(),
        );

        assert!(seq.can_run());

        seq.start_run();
        assert!(!seq.can_run());
        assert_eq!(seq.status, SequencerStatus::Running);

        seq.complete_run();
        assert!(seq.can_run());

        seq.start_maintenance();
        assert!(!seq.can_run());
    }
}

