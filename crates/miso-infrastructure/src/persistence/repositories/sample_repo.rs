//! SeaORM implementation of SampleRepository.

use async_trait::async_trait;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use tracing::{debug, instrument};

use miso_domain::entities::{EntityId, Sample};
use miso_domain::errors::DomainError;
use miso_domain::repositories::{QueryOptions, SampleRepository};

use crate::persistence::entities::sample::{self, Entity as SampleEntity};

/// SeaORM-based sample repository.
#[derive(Debug, Clone)]
pub struct SeaOrmSampleRepository {
    db: DatabaseConnection,
}

impl SeaOrmSampleRepository {
    /// Creates a new repository with the given database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Converts a database model to a domain Sample.
    ///
    /// This handles the complexity of Plain vs Detailed sample modes.
    fn model_to_domain(&self, model: sample::Model) -> Sample {
        use miso_domain::entities::{
            DetailedSampleData, PlainSampleData, SampleClass, SampleDetails,
        };
        use miso_domain::value_objects::{Barcode, Concentration, QcStatus, Volume};

        let details = if model.sample_mode == "detailed" {
            let sample_class = match model.sample_class.as_deref() {
                Some("identity") => SampleClass::Identity,
                Some("tissue") => SampleClass::Tissue,
                Some("tissue_processing") => SampleClass::TissueProcessing,
                Some("stock") => SampleClass::Stock,
                Some("aliquot") => SampleClass::Aliquot,
                Some("single_cell") => SampleClass::SingleCell,
                Some("whole_transcriptome") => SampleClass::WholeTranscriptome,
                _ => SampleClass::Plain,
            };

            SampleDetails::Detailed(DetailedSampleData {
                parent_id: model.parent_id,
                sample_class,
                external_name: model.external_name.clone(),
                tissue_origin: model.tissue_origin.clone(),
                tissue_type: model.tissue_type.clone(),
                time_point: None,
                group_id: None,
                group_description: None,
                passage: None,
                analyte_type: model.analyte_type.clone(),
                purpose: None,
            })
        } else {
            SampleDetails::Plain(PlainSampleData {
                scientific_name: model.scientific_name.clone().unwrap_or_default(),
                sample_type: None,
            })
        };

        let qc_status = match model.qc_status.as_str() {
            "not_ready" => QcStatus::NotReady,
            "ready" => QcStatus::Ready,
            "passed" => QcStatus::Passed,
            "failed" => QcStatus::Failed,
            "needs_review" => QcStatus::NeedsReview,
            _ => QcStatus::NotReady,
        };

        let volume = model.volume.map(|v| {
            // Convert Decimal to f64
            use std::str::FromStr;
            let val = f64::from_str(&v.to_string()).unwrap_or(0.0);
            Volume::microliters(val)
        });

        let concentration = model.concentration.map(|c| {
            use std::str::FromStr;
            let val = f64::from_str(&c.to_string()).unwrap_or(0.0);
            Concentration::ng_per_ul(val)
        });

        Sample {
            id: model.id,
            name: model.name,
            barcode: Barcode::new_unchecked(model.barcode),
            project_id: model.project_id,
            description: model.description,
            details,
            volume,
            concentration,
            qc_status,
            received_at: model.received_at,
            created_by: model.created_by,
            created_at: model.created_at,
            updated_at: model.updated_at,
            archived: model.archived,
        }
    }
}

#[async_trait]
impl SampleRepository for SeaOrmSampleRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: EntityId) -> Result<Option<Sample>, DomainError> {
        debug!("Finding sample by ID: {}", id);

        let result = SampleEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(result.map(|m| self.model_to_domain(m)))
    }

    #[instrument(skip(self))]
    async fn find_by_barcode(&self, barcode: &str) -> Result<Option<Sample>, DomainError> {
        debug!("Finding sample by barcode: {}", barcode);

        let result = SampleEntity::find()
            .filter(sample::Column::Barcode.eq(barcode))
            .one(&self.db)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(result.map(|m| self.model_to_domain(m)))
    }

    #[instrument(skip(self))]
    async fn find_by_project(
        &self,
        project_id: EntityId,
        options: QueryOptions,
    ) -> Result<Vec<Sample>, DomainError> {
        debug!("Finding samples by project: {}", project_id);

        let mut query = SampleEntity::find().filter(sample::Column::ProjectId.eq(project_id));

        // Apply sorting
        if let Some(sort_by) = &options.sort_by {
            let order = if options.ascending.unwrap_or(true) {
                sea_orm::Order::Asc
            } else {
                sea_orm::Order::Desc
            };

            query = match sort_by.as_str() {
                "name" => query.order_by(sample::Column::Name, order),
                "barcode" => query.order_by(sample::Column::Barcode, order),
                "created_at" => query.order_by(sample::Column::CreatedAt, order),
                _ => query.order_by(sample::Column::Id, order),
            };
        }

        // Apply pagination
        if let Some(offset) = options.offset {
            query = query.offset(offset);
        }

        if let Some(limit) = options.limit {
            query = query.limit(limit);
        }

        let results = query
            .all(&self.db)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(results.into_iter().map(|m| self.model_to_domain(m)).collect())
    }

    #[instrument(skip(self))]
    async fn find_by_parent(&self, parent_id: EntityId) -> Result<Vec<Sample>, DomainError> {
        debug!("Finding samples by parent: {}", parent_id);

        let results = SampleEntity::find()
            .filter(sample::Column::ParentId.eq(parent_id))
            .all(&self.db)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(results.into_iter().map(|m| self.model_to_domain(m)).collect())
    }

    #[instrument(skip(self))]
    async fn list(&self, options: QueryOptions) -> Result<Vec<Sample>, DomainError> {
        debug!("Listing samples with options: {:?}", options);

        let mut query = SampleEntity::find();

        // Apply pagination
        if let Some(offset) = options.offset {
            query = query.offset(offset);
        }

        if let Some(limit) = options.limit {
            query = query.limit(limit);
        }

        let results = query
            .all(&self.db)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(results.into_iter().map(|m| self.model_to_domain(m)).collect())
    }

    #[instrument(skip(self, _sample))]
    async fn save(&self, _sample: &Sample) -> Result<EntityId, DomainError> {
        // TODO: Implement save with proper ActiveModel conversion
        Err(DomainError::Validation("Not yet implemented".to_string()))
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: EntityId) -> Result<(), DomainError> {
        debug!("Deleting sample: {}", id);

        SampleEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn count_by_project(&self, project_id: EntityId) -> Result<u64, DomainError> {
        let count = SampleEntity::find()
            .filter(sample::Column::ProjectId.eq(project_id))
            .count(&self.db)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(count)
    }
}

