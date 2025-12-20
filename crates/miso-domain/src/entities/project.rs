//! Project entity - the administrative root of the LIMS.
//!
//! Every sample, library, and pool belongs to a project. Projects
//! provide the primary boundary for access control and organization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::EntityId;

/// The status of a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    /// Project is being set up
    #[default]
    Pending,
    /// Project is active and accepting samples
    Active,
    /// Project is paused/on hold
    OnHold,
    /// Project is completed
    Completed,
    /// Project was cancelled
    Cancelled,
}

impl ProjectStatus {
    /// Returns true if new samples can be added to this project.
    pub fn accepts_samples(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns true if this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled)
    }
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Active => write!(f, "Active"),
            Self::OnHold => write!(f, "On Hold"),
            Self::Completed => write!(f, "Completed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// A project in the LIMS.
///
/// Projects are the administrative root and access control boundary.
/// All samples, libraries, and pools belong to exactly one project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier
    pub id: EntityId,
    /// Short code/alias (e.g., "PROJ001")
    pub code: String,
    /// Full name of the project
    pub name: String,
    /// Description of the project's goals
    pub description: Option<String>,
    /// Current status
    pub status: ProjectStatus,
    /// Principal Investigator name
    pub pi_name: Option<String>,
    /// Principal Investigator email
    pub pi_email: Option<String>,
    /// Funding/grant reference
    pub reference_number: Option<String>,
    /// Target number of samples
    pub target_sample_count: Option<u32>,
    /// Actual number of samples received
    pub sample_count: u32,
    /// When the project was created
    pub created_at: DateTime<Utc>,
    /// Who created the project
    pub created_by: String,
    /// When the project was last modified
    pub updated_at: DateTime<Utc>,
    /// When the project is due/expected to complete
    pub due_date: Option<DateTime<Utc>>,
}

impl Project {
    /// Creates a new project.
    pub fn new(
        id: EntityId,
        code: String,
        name: String,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            code,
            name,
            description: None,
            status: ProjectStatus::Pending,
            pi_name: None,
            pi_email: None,
            reference_number: None,
            target_sample_count: None,
            sample_count: 0,
            created_at: now,
            created_by,
            updated_at: now,
            due_date: None,
        }
    }

    /// Activates the project.
    pub fn activate(&mut self) {
        self.status = ProjectStatus::Active;
        self.updated_at = Utc::now();
    }

    /// Puts the project on hold.
    pub fn hold(&mut self) {
        self.status = ProjectStatus::OnHold;
        self.updated_at = Utc::now();
    }

    /// Completes the project.
    pub fn complete(&mut self) {
        self.status = ProjectStatus::Completed;
        self.updated_at = Utc::now();
    }

    /// Cancels the project.
    pub fn cancel(&mut self) {
        self.status = ProjectStatus::Cancelled;
        self.updated_at = Utc::now();
    }

    /// Increments the sample count.
    pub fn add_sample(&mut self) {
        self.sample_count += 1;
        self.updated_at = Utc::now();
    }

    /// Returns the progress percentage (0-100) if a target is set.
    pub fn progress_percent(&self) -> Option<f64> {
        self.target_sample_count.map(|target| {
            if target == 0 {
                100.0
            } else {
                (self.sample_count as f64 / target as f64 * 100.0).min(100.0)
            }
        })
    }

    /// Returns true if new samples can be added.
    pub fn can_add_samples(&self) -> bool {
        self.status.accepts_samples()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project::new(
            1,
            "PROJ001".to_string(),
            "Test Project".to_string(),
            "admin".to_string(),
        );
        assert_eq!(project.status, ProjectStatus::Pending);
        assert_eq!(project.sample_count, 0);
    }

    #[test]
    fn test_project_lifecycle() {
        let mut project = Project::new(
            1,
            "PROJ001".to_string(),
            "Test Project".to_string(),
            "admin".to_string(),
        );

        assert!(!project.can_add_samples()); // Pending

        project.activate();
        assert!(project.can_add_samples());

        project.hold();
        assert!(!project.can_add_samples());

        project.activate();
        project.complete();
        assert!(project.status.is_terminal());
    }

    #[test]
    fn test_progress_tracking() {
        let mut project = Project::new(
            1,
            "PROJ001".to_string(),
            "Test Project".to_string(),
            "admin".to_string(),
        );

        // No target set
        assert!(project.progress_percent().is_none());

        // Set target
        project.target_sample_count = Some(100);

        // Add samples
        for _ in 0..50 {
            project.add_sample();
        }

        let progress = project.progress_percent().unwrap();
        assert!((progress - 50.0).abs() < 0.01);
    }
}

