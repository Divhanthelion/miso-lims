//! Project service for project operations.

use std::sync::Arc;

use miso_domain::entities::Project;
use miso_domain::errors::DomainError;
use miso_domain::repositories::{ProjectRepository, QueryOptions};
use tracing::{info, instrument};

use crate::dto::{CreateProjectRequest, ProjectResponse, ProjectSummary, UpdateProjectRequest};

/// Service for project operations.
pub struct ProjectService<R: ProjectRepository> {
    repository: Arc<R>,
}

impl<R: ProjectRepository> ProjectService<R> {
    /// Creates a new project service.
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Creates a new project.
    #[instrument(skip(self))]
    pub async fn create_project(
        &self,
        request: CreateProjectRequest,
        created_by: &str,
    ) -> Result<ProjectResponse, DomainError> {
        // Check if code is unique
        if self.repository.find_by_code(&request.code).await?.is_some() {
            return Err(DomainError::Duplicate {
                entity_type: "Project".to_string(),
                field: "code".to_string(),
                value: request.code,
            });
        }

        let mut project = Project::new(0, request.code, request.name, created_by.to_string());

        project.description = request.description;
        project.pi_name = request.pi_name;
        project.pi_email = request.pi_email;
        project.reference_number = request.reference_number;
        project.target_sample_count = request.target_sample_count;

        let id = self.repository.save(&project).await?;
        project.id = id;

        info!("Created project: {} (ID: {})", project.code, id);

        Ok(project.into())
    }

    /// Gets a project by ID.
    #[instrument(skip(self))]
    pub async fn get_project(&self, id: i32) -> Result<ProjectResponse, DomainError> {
        let project = self.repository.find_by_id(id).await?.ok_or_else(|| {
            DomainError::NotFound {
                entity_type: "Project".to_string(),
                id: id.to_string(),
            }
        })?;

        Ok(project.into())
    }

    /// Gets a project by code.
    #[instrument(skip(self))]
    pub async fn get_project_by_code(&self, code: &str) -> Result<ProjectResponse, DomainError> {
        let project = self.repository.find_by_code(code).await?.ok_or_else(|| {
            DomainError::NotFound {
                entity_type: "Project".to_string(),
                id: code.to_string(),
            }
        })?;

        Ok(project.into())
    }

    /// Lists all projects.
    #[instrument(skip(self))]
    pub async fn list_projects(
        &self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<ProjectSummary>, DomainError> {
        let options = QueryOptions::new()
            .limit(limit.unwrap_or(100))
            .offset(offset.unwrap_or(0))
            .sort_by("name")
            .ascending();

        let projects = self.repository.list(options).await?;

        Ok(projects.into_iter().map(|p| p.into()).collect())
    }

    /// Updates a project.
    #[instrument(skip(self))]
    pub async fn update_project(
        &self,
        id: i32,
        request: UpdateProjectRequest,
    ) -> Result<ProjectResponse, DomainError> {
        let mut project = self.repository.find_by_id(id).await?.ok_or_else(|| {
            DomainError::NotFound {
                entity_type: "Project".to_string(),
                id: id.to_string(),
            }
        })?;

        // Apply updates
        if let Some(name) = request.name {
            project.name = name;
        }
        if let Some(desc) = request.description {
            project.description = Some(desc);
        }
        if let Some(email) = request.pi_email {
            project.pi_email = Some(email);
        }
        if let Some(name) = request.pi_name {
            project.pi_name = Some(name);
        }
        if let Some(ref_num) = request.reference_number {
            project.reference_number = Some(ref_num);
        }
        if let Some(target) = request.target_sample_count {
            project.target_sample_count = Some(target);
        }
        if let Some(status) = request.status {
            match status.as_str() {
                "active" => project.activate(),
                "on_hold" => project.hold(),
                "completed" => project.complete(),
                "cancelled" => project.cancel(),
                _ => {}
            }
        }

        project.updated_at = chrono::Utc::now();

        self.repository.save(&project).await?;

        info!("Updated project: {} (ID: {})", project.code, id);

        Ok(project.into())
    }

    /// Deletes a project.
    #[instrument(skip(self))]
    pub async fn delete_project(&self, id: i32) -> Result<(), DomainError> {
        // Check if project exists
        self.repository.find_by_id(id).await?.ok_or_else(|| {
            DomainError::NotFound {
                entity_type: "Project".to_string(),
                id: id.to_string(),
            }
        })?;

        self.repository.delete(id).await?;

        info!("Deleted project: {}", id);

        Ok(())
    }

    /// Gets the total count of projects.
    #[instrument(skip(self))]
    pub async fn count_projects(&self) -> Result<u64, DomainError> {
        self.repository.count().await
    }
}

