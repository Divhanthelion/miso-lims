//! SeaORM implementation of ProjectRepository.

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use tracing::{debug, instrument};

use miso_domain::entities::{EntityId, Project};
use miso_domain::errors::DomainError;
use miso_domain::repositories::{ProjectRepository, QueryOptions};

use crate::persistence::entities::project::{self, Entity as ProjectEntity};

/// SeaORM-based project repository.
#[derive(Debug, Clone)]
pub struct SeaOrmProjectRepository {
    db: DatabaseConnection,
}

impl SeaOrmProjectRepository {
    /// Creates a new repository with the given database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ProjectRepository for SeaOrmProjectRepository {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: EntityId) -> Result<Option<Project>, DomainError> {
        debug!("Finding project by ID: {}", id);

        let result = ProjectEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(result.map(|m| m.into()))
    }

    #[instrument(skip(self))]
    async fn find_by_code(&self, code: &str) -> Result<Option<Project>, DomainError> {
        debug!("Finding project by code: {}", code);

        let result = ProjectEntity::find()
            .filter(project::Column::Code.eq(code))
            .one(&self.db)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(result.map(|m| m.into()))
    }

    #[instrument(skip(self))]
    async fn list(&self, options: QueryOptions) -> Result<Vec<Project>, DomainError> {
        debug!("Listing projects with options: {:?}", options);

        let mut query = ProjectEntity::find();

        // Apply sorting
        if let Some(sort_by) = &options.sort_by {
            let order = if options.ascending.unwrap_or(true) {
                sea_orm::Order::Asc
            } else {
                sea_orm::Order::Desc
            };

            query = match sort_by.as_str() {
                "name" => query.order_by(project::Column::Name, order),
                "code" => query.order_by(project::Column::Code, order),
                "created_at" => query.order_by(project::Column::CreatedAt, order),
                _ => query.order_by(project::Column::Id, order),
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

        Ok(results.into_iter().map(|m| m.into()).collect())
    }

    #[instrument(skip(self))]
    async fn save(&self, project: &Project) -> Result<EntityId, DomainError> {
        debug!("Saving project: {}", project.code);

        let active_model: project::ActiveModel = project.into();

        // Check if this is an insert or update
        let result = if project.id == 0 {
            // Insert
            let model = active_model
                .insert(&self.db)
                .await
                .map_err(|e| DomainError::Validation(e.to_string()))?;
            model.id
        } else {
            // Update
            let model = active_model
                .update(&self.db)
                .await
                .map_err(|e| DomainError::Validation(e.to_string()))?;
            model.id
        };

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: EntityId) -> Result<(), DomainError> {
        debug!("Deleting project: {}", id);

        ProjectEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn count(&self) -> Result<u64, DomainError> {
        let count = ProjectEntity::find()
            .count(&self.db)
            .await
            .map_err(|e| DomainError::Validation(e.to_string()))?;

        Ok(count)
    }
}

