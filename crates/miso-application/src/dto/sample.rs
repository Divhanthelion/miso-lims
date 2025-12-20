//! Sample Data Transfer Objects.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request to create a new plain sample.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePlainSampleRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    pub project_id: i32,

    #[validate(length(min = 1, max = 255))]
    pub scientific_name: String,

    pub description: Option<String>,

    pub sample_type: Option<String>,
}

/// Request to create a detailed sample (with hierarchy).
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDetailedSampleRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    pub project_id: i32,

    pub parent_id: Option<i32>,

    #[validate(length(min = 1, max = 50))]
    pub sample_class: String,

    pub external_name: Option<String>,

    pub tissue_origin: Option<String>,

    pub tissue_type: Option<String>,

    pub analyte_type: Option<String>,

    pub description: Option<String>,
}

/// Request to update an existing sample.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateSampleRequest {
    pub description: Option<String>,

    pub volume_ul: Option<f64>,

    pub concentration_ng_ul: Option<f64>,

    pub qc_status: Option<String>,
}

/// Response containing sample details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleResponse {
    pub id: i32,
    pub name: String,
    pub barcode: String,
    pub project_id: i32,
    pub description: Option<String>,
    pub sample_mode: String,
    pub sample_class: String,
    pub parent_id: Option<i32>,
    pub volume_ul: Option<f64>,
    pub concentration_ng_ul: Option<f64>,
    pub qc_status: String,
    pub received_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived: bool,
}

impl From<miso_domain::entities::Sample> for SampleResponse {
    fn from(sample: miso_domain::entities::Sample) -> Self {
        use miso_domain::entities::SampleDetails;

        let (sample_mode, sample_class) = match &sample.details {
            SampleDetails::Plain(_) => ("plain".to_string(), "plain".to_string()),
            SampleDetails::Detailed(d) => ("detailed".to_string(), d.sample_class.to_string()),
        };

        Self {
            id: sample.id,
            name: sample.name,
            barcode: sample.barcode.to_string(),
            project_id: sample.project_id,
            description: sample.description,
            sample_mode,
            sample_class,
            parent_id: sample.parent_id(),
            volume_ul: sample.volume.map(|v| v.as_microliters()),
            concentration_ng_ul: sample.concentration.map(|c| c.value()),
            qc_status: sample.qc_status.to_string(),
            received_at: sample.received_at,
            created_by: sample.created_by,
            created_at: sample.created_at,
            updated_at: sample.updated_at,
            archived: sample.archived,
        }
    }
}

/// Summary of a sample (for list views).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleSummary {
    pub id: i32,
    pub name: String,
    pub barcode: String,
    pub sample_class: String,
    pub qc_status: String,
    pub can_create_library: bool,
}

impl From<miso_domain::entities::Sample> for SampleSummary {
    fn from(sample: miso_domain::entities::Sample) -> Self {
        Self {
            id: sample.id,
            name: sample.name.clone(),
            barcode: sample.barcode.to_string(),
            sample_class: sample.sample_class().to_string(),
            qc_status: sample.qc_status.to_string(),
            can_create_library: sample.can_create_library(),
        }
    }
}

/// Scan result from VisionMate scanner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RackScanResult {
    pub rack_barcode: Option<String>,
    pub tubes: Vec<TubeScanResult>,
    pub empty_count: usize,
    pub error_count: usize,
}

/// Individual tube from a rack scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TubeScanResult {
    pub position: String,
    pub barcode: String,
    pub sample_id: Option<i32>,
    pub sample_name: Option<String>,
}

