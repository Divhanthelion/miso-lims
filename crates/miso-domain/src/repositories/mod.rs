//! Repository Traits - interfaces for data persistence.
//!
//! These traits define the contract for data access. They are implemented
//! in the infrastructure layer using SeaORM or other persistence mechanisms.
//!
//! The domain layer only knows about these traits, not the implementations,
//! which allows for easy testing with mock implementations.

use crate::entities::*;
use crate::errors::DomainError;
use async_trait::async_trait;

/// Common query options for listing entities.
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Maximum number of results to return
    pub limit: Option<u64>,
    /// Number of results to skip (for pagination)
    pub offset: Option<u64>,
    /// Sort field
    pub sort_by: Option<String>,
    /// Sort direction (true = ascending, false = descending)
    pub ascending: Option<bool>,
}

impl QueryOptions {
    /// Creates a new query options builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the limit.
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the offset.
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the sort field.
    pub fn sort_by(mut self, field: impl Into<String>) -> Self {
        self.sort_by = Some(field.into());
        self
    }

    /// Sets ascending order.
    pub fn ascending(mut self) -> Self {
        self.ascending = Some(true);
        self
    }

    /// Sets descending order.
    pub fn descending(mut self) -> Self {
        self.ascending = Some(false);
        self
    }
}

/// Repository for Project entities.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Finds a project by ID.
    async fn find_by_id(&self, id: EntityId) -> Result<Option<Project>, DomainError>;

    /// Finds a project by code.
    async fn find_by_code(&self, code: &str) -> Result<Option<Project>, DomainError>;

    /// Lists all projects.
    async fn list(&self, options: QueryOptions) -> Result<Vec<Project>, DomainError>;

    /// Saves a project (insert or update).
    async fn save(&self, project: &Project) -> Result<EntityId, DomainError>;

    /// Deletes a project.
    async fn delete(&self, id: EntityId) -> Result<(), DomainError>;

    /// Counts projects matching optional criteria.
    async fn count(&self) -> Result<u64, DomainError>;
}

/// Repository for Sample entities.
#[async_trait]
pub trait SampleRepository: Send + Sync {
    /// Finds a sample by ID.
    async fn find_by_id(&self, id: EntityId) -> Result<Option<Sample>, DomainError>;

    /// Finds a sample by barcode.
    async fn find_by_barcode(&self, barcode: &str) -> Result<Option<Sample>, DomainError>;

    /// Finds samples by project.
    async fn find_by_project(
        &self,
        project_id: EntityId,
        options: QueryOptions,
    ) -> Result<Vec<Sample>, DomainError>;

    /// Finds samples by parent (for detailed hierarchy).
    async fn find_by_parent(&self, parent_id: EntityId) -> Result<Vec<Sample>, DomainError>;

    /// Lists samples with optional filtering.
    async fn list(&self, options: QueryOptions) -> Result<Vec<Sample>, DomainError>;

    /// Saves a sample (insert or update).
    async fn save(&self, sample: &Sample) -> Result<EntityId, DomainError>;

    /// Deletes a sample.
    async fn delete(&self, id: EntityId) -> Result<(), DomainError>;

    /// Counts samples in a project.
    async fn count_by_project(&self, project_id: EntityId) -> Result<u64, DomainError>;
}

/// Repository for Library entities.
#[async_trait]
pub trait LibraryRepository: Send + Sync {
    /// Finds a library by ID.
    async fn find_by_id(&self, id: EntityId) -> Result<Option<Library>, DomainError>;

    /// Finds a library by barcode.
    async fn find_by_barcode(&self, barcode: &str) -> Result<Option<Library>, DomainError>;

    /// Finds libraries by sample.
    async fn find_by_sample(&self, sample_id: EntityId) -> Result<Vec<Library>, DomainError>;

    /// Finds libraries by project.
    async fn find_by_project(
        &self,
        project_id: EntityId,
        options: QueryOptions,
    ) -> Result<Vec<Library>, DomainError>;

    /// Finds libraries by IDs (batch load).
    async fn find_by_ids(&self, ids: &[EntityId]) -> Result<Vec<Library>, DomainError>;

    /// Saves a library (insert or update).
    async fn save(&self, library: &Library) -> Result<EntityId, DomainError>;

    /// Deletes a library.
    async fn delete(&self, id: EntityId) -> Result<(), DomainError>;
}

/// Repository for Pool entities.
#[async_trait]
pub trait PoolRepository: Send + Sync {
    /// Finds a pool by ID.
    async fn find_by_id(&self, id: EntityId) -> Result<Option<Pool>, DomainError>;

    /// Finds a pool by barcode.
    async fn find_by_barcode(&self, barcode: &str) -> Result<Option<Pool>, DomainError>;

    /// Lists pools with optional filtering.
    async fn list(&self, options: QueryOptions) -> Result<Vec<Pool>, DomainError>;

    /// Finds pools containing a specific library.
    async fn find_by_library(&self, library_id: EntityId) -> Result<Vec<Pool>, DomainError>;

    /// Saves a pool (insert or update).
    async fn save(&self, pool: &Pool) -> Result<EntityId, DomainError>;

    /// Deletes a pool.
    async fn delete(&self, id: EntityId) -> Result<(), DomainError>;
}

/// Repository for Run entities.
#[async_trait]
pub trait RunRepository: Send + Sync {
    /// Finds a run by ID.
    async fn find_by_id(&self, id: EntityId) -> Result<Option<Run>, DomainError>;

    /// Finds a run by name.
    async fn find_by_name(&self, name: &str) -> Result<Option<Run>, DomainError>;

    /// Finds runs by sequencer.
    async fn find_by_sequencer(&self, sequencer_id: EntityId) -> Result<Vec<Run>, DomainError>;

    /// Finds runs by status.
    async fn find_by_status(&self, status: RunStatus) -> Result<Vec<Run>, DomainError>;

    /// Lists runs with optional filtering.
    async fn list(&self, options: QueryOptions) -> Result<Vec<Run>, DomainError>;

    /// Saves a run (insert or update).
    async fn save(&self, run: &Run) -> Result<EntityId, DomainError>;

    /// Deletes a run.
    async fn delete(&self, id: EntityId) -> Result<(), DomainError>;
}

/// Repository for Sequencer entities.
#[async_trait]
pub trait SequencerRepository: Send + Sync {
    /// Finds a sequencer by ID.
    async fn find_by_id(&self, id: EntityId) -> Result<Option<Sequencer>, DomainError>;

    /// Finds a sequencer by name.
    async fn find_by_name(&self, name: &str) -> Result<Option<Sequencer>, DomainError>;

    /// Lists all sequencers.
    async fn list(&self) -> Result<Vec<Sequencer>, DomainError>;

    /// Finds available sequencers.
    async fn find_available(&self) -> Result<Vec<Sequencer>, DomainError>;

    /// Saves a sequencer (insert or update).
    async fn save(&self, sequencer: &Sequencer) -> Result<EntityId, DomainError>;
}

/// Repository for StorageBox entities.
#[async_trait]
pub trait StorageBoxRepository: Send + Sync {
    /// Finds a box by ID.
    async fn find_by_id(&self, id: EntityId) -> Result<Option<StorageBox>, DomainError>;

    /// Finds a box by barcode.
    async fn find_by_barcode(&self, barcode: &str) -> Result<Option<StorageBox>, DomainError>;

    /// Finds boxes by location.
    async fn find_by_location(&self, freezer: &str) -> Result<Vec<StorageBox>, DomainError>;

    /// Lists all boxes.
    async fn list(&self, options: QueryOptions) -> Result<Vec<StorageBox>, DomainError>;

    /// Finds the box containing a specific item.
    async fn find_by_item(
        &self,
        item_type: crate::entities::box_entity::StorableType,
        item_id: EntityId,
    ) -> Result<Option<(StorageBox, crate::value_objects::BoxPosition)>, DomainError>;

    /// Saves a box (insert or update).
    async fn save(&self, storage_box: &StorageBox) -> Result<EntityId, DomainError>;

    /// Deletes a box.
    async fn delete(&self, id: EntityId) -> Result<(), DomainError>;
}

/// Repository for User entities.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Finds a user by ID.
    async fn find_by_id(&self, id: EntityId) -> Result<Option<User>, DomainError>;

    /// Finds a user by username.
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError>;

    /// Finds a user by email.
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;

    /// Lists all users.
    async fn list(&self, options: QueryOptions) -> Result<Vec<User>, DomainError>;

    /// Saves a user (insert or update).
    async fn save(&self, user: &User) -> Result<EntityId, DomainError>;

    /// Deletes a user.
    async fn delete(&self, id: EntityId) -> Result<(), DomainError>;
}

