//! Project Data Transfer Objects.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Request to create a new project.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[validate(length(min = 2, max = 50))]
    pub code: String,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    pub description: Option<String>,

    #[validate(email)]
    pub pi_email: Option<String>,

    pub pi_name: Option<String>,

    pub reference_number: Option<String>,

    pub target_sample_count: Option<u32>,
}

/// Request to update an existing project.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateProjectRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    pub description: Option<String>,

    #[validate(email)]
    pub pi_email: Option<String>,

    pub pi_name: Option<String>,

    pub reference_number: Option<String>,

    pub target_sample_count: Option<u32>,

    pub status: Option<String>,
}

/// Response containing project details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub pi_name: Option<String>,
    pub pi_email: Option<String>,
    pub reference_number: Option<String>,
    pub target_sample_count: Option<u32>,
    pub sample_count: u32,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
}

impl From<miso_domain::entities::Project> for ProjectResponse {
    fn from(project: miso_domain::entities::Project) -> Self {
        Self {
            id: project.id,
            code: project.code,
            name: project.name,
            description: project.description,
            status: project.status.to_string(),
            pi_name: project.pi_name,
            pi_email: project.pi_email,
            reference_number: project.reference_number,
            target_sample_count: project.target_sample_count,
            sample_count: project.sample_count,
            created_at: project.created_at,
            created_by: project.created_by,
            updated_at: project.updated_at,
            due_date: project.due_date,
        }
    }
}

/// Summary of a project (for list views).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub status: String,
    pub sample_count: u32,
    pub progress_percent: Option<f64>,
}

impl From<miso_domain::entities::Project> for ProjectSummary {
    fn from(project: miso_domain::entities::Project) -> Self {
        Self {
            id: project.id,
            code: project.code.clone(),
            name: project.name.clone(),
            status: project.status.to_string(),
            sample_count: project.sample_count,
            progress_percent: project.progress_percent(),
        }
    }
}

