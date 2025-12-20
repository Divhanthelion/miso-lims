//! Sample service for sample operations.

use std::sync::Arc;

use miso_domain::entities::{PlainSampleData, Sample, SampleDetails};
use miso_domain::errors::DomainError;
use miso_domain::repositories::{QueryOptions, SampleRepository};
use miso_domain::services::BarcodeValidator;
use tracing::{info, instrument};

use crate::dto::{CreatePlainSampleRequest, SampleResponse, SampleSummary, UpdateSampleRequest};

/// Service for sample operations.
pub struct SampleService<R: SampleRepository> {
    repository: Arc<R>,
    barcode_validator: BarcodeValidator,
}

impl<R: SampleRepository> SampleService<R> {
    /// Creates a new sample service.
    pub fn new(repository: Arc<R>) -> Self {
        Self {
            repository,
            barcode_validator: BarcodeValidator::new(),
        }
    }

    /// Creates a new plain sample.
    #[instrument(skip(self))]
    pub async fn create_plain_sample(
        &self,
        request: CreatePlainSampleRequest,
        created_by: &str,
    ) -> Result<SampleResponse, DomainError> {
        // Generate a unique barcode
        let barcode = self.barcode_validator.generate_barcode("SAM");

        // Check if barcode is unique
        if self.repository.find_by_barcode(barcode.as_str()).await?.is_some() {
            return Err(DomainError::Duplicate {
                entity_type: "Sample".to_string(),
                field: "barcode".to_string(),
                value: barcode.to_string(),
            });
        }

        let sample = Sample::new_plain(
            0,
            request.name,
            barcode,
            request.project_id,
            request.scientific_name,
            created_by.to_string(),
        );

        let id = self.repository.save(&sample).await?;

        info!("Created sample: {} (ID: {})", sample.name, id);

        // Fetch the saved sample to return
        let saved = self.repository.find_by_id(id).await?.ok_or_else(|| {
            DomainError::NotFound {
                entity_type: "Sample".to_string(),
                id: id.to_string(),
            }
        })?;

        Ok(saved.into())
    }

    /// Gets a sample by ID.
    #[instrument(skip(self))]
    pub async fn get_sample(&self, id: i32) -> Result<SampleResponse, DomainError> {
        let sample = self.repository.find_by_id(id).await?.ok_or_else(|| {
            DomainError::NotFound {
                entity_type: "Sample".to_string(),
                id: id.to_string(),
            }
        })?;

        Ok(sample.into())
    }

    /// Gets a sample by barcode.
    #[instrument(skip(self))]
    pub async fn get_sample_by_barcode(&self, barcode: &str) -> Result<SampleResponse, DomainError> {
        let sample = self.repository.find_by_barcode(barcode).await?.ok_or_else(|| {
            DomainError::NotFound {
                entity_type: "Sample".to_string(),
                id: barcode.to_string(),
            }
        })?;

        Ok(sample.into())
    }

    /// Lists samples for a project.
    #[instrument(skip(self))]
    pub async fn list_samples_by_project(
        &self,
        project_id: i32,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<SampleSummary>, DomainError> {
        let options = QueryOptions::new()
            .limit(limit.unwrap_or(100))
            .offset(offset.unwrap_or(0))
            .sort_by("name")
            .ascending();

        let samples = self.repository.find_by_project(project_id, options).await?;

        Ok(samples.into_iter().map(|s| s.into()).collect())
    }

    /// Lists child samples (for detailed sample hierarchy).
    #[instrument(skip(self))]
    pub async fn list_child_samples(&self, parent_id: i32) -> Result<Vec<SampleSummary>, DomainError> {
        let samples = self.repository.find_by_parent(parent_id).await?;

        Ok(samples.into_iter().map(|s| s.into()).collect())
    }

    /// Updates a sample.
    #[instrument(skip(self))]
    pub async fn update_sample(
        &self,
        id: i32,
        request: UpdateSampleRequest,
    ) -> Result<SampleResponse, DomainError> {
        let mut sample = self.repository.find_by_id(id).await?.ok_or_else(|| {
            DomainError::NotFound {
                entity_type: "Sample".to_string(),
                id: id.to_string(),
            }
        })?;

        // Apply updates
        if let Some(desc) = request.description {
            sample.description = Some(desc);
        }
        if let Some(vol) = request.volume_ul {
            sample.volume = Some(miso_domain::value_objects::Volume::microliters(vol));
        }
        if let Some(conc) = request.concentration_ng_ul {
            sample.concentration = Some(miso_domain::value_objects::Concentration::ng_per_ul(conc));
        }
        if let Some(status) = request.qc_status {
            use miso_domain::value_objects::QcStatus;
            let qc = match status.as_str() {
                "not_ready" => QcStatus::NotReady,
                "ready" => QcStatus::Ready,
                "passed" => QcStatus::Passed,
                "failed" => QcStatus::Failed,
                "needs_review" => QcStatus::NeedsReview,
                _ => return Err(DomainError::Validation(format!("Invalid QC status: {}", status))),
            };
            sample.set_qc_status(qc);
        }

        self.repository.save(&sample).await?;

        info!("Updated sample: {} (ID: {})", sample.name, id);

        Ok(sample.into())
    }

    /// Deletes a sample.
    #[instrument(skip(self))]
    pub async fn delete_sample(&self, id: i32) -> Result<(), DomainError> {
        // Check if sample exists
        self.repository.find_by_id(id).await?.ok_or_else(|| {
            DomainError::NotFound {
                entity_type: "Sample".to_string(),
                id: id.to_string(),
            }
        })?;

        self.repository.delete(id).await?;

        info!("Deleted sample: {}", id);

        Ok(())
    }

    /// Counts samples in a project.
    #[instrument(skip(self))]
    pub async fn count_samples_by_project(&self, project_id: i32) -> Result<u64, DomainError> {
        self.repository.count_by_project(project_id).await
    }
}

