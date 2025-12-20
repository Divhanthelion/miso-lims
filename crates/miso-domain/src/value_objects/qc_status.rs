//! Quality Control status and result value objects.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The overall QC status of an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QcStatus {
    /// QC has not been performed yet
    #[default]
    NotReady,
    /// QC is ready to be performed
    Ready,
    /// QC has passed
    Passed,
    /// QC has failed
    Failed,
    /// QC requires further review
    NeedsReview,
}

impl QcStatus {
    /// Returns true if this status allows progression to the next workflow step.
    pub fn allows_progression(&self) -> bool {
        matches!(self, Self::Passed)
    }

    /// Returns true if this status indicates the entity should be excluded.
    pub fn is_terminal_failure(&self) -> bool {
        matches!(self, Self::Failed)
    }

    /// Returns true if QC has been completed (either passed or failed).
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Passed | Self::Failed)
    }
}

impl fmt::Display for QcStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotReady => write!(f, "Not Ready"),
            Self::Ready => write!(f, "Ready"),
            Self::Passed => write!(f, "Passed"),
            Self::Failed => write!(f, "Failed"),
            Self::NeedsReview => write!(f, "Needs Review"),
        }
    }
}

/// The type of QC test performed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QcTestType {
    /// Qubit fluorometric quantification
    Qubit,
    /// NanoDrop spectrophotometry
    NanoDrop,
    /// TapeStation/Bioanalyzer fragment analysis
    TapeStation,
    /// Bioanalyzer fragment analysis
    Bioanalyzer,
    /// qPCR quantification
    Qpcr,
    /// Visual inspection
    Visual,
    /// Custom/other test
    Custom(String),
}

impl fmt::Display for QcTestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Qubit => write!(f, "Qubit"),
            Self::NanoDrop => write!(f, "NanoDrop"),
            Self::TapeStation => write!(f, "TapeStation"),
            Self::Bioanalyzer => write!(f, "Bioanalyzer"),
            Self::Qpcr => write!(f, "qPCR"),
            Self::Visual => write!(f, "Visual"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// A single QC test result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QcResult {
    /// The type of test performed
    pub test_type: QcTestType,
    /// The result value (interpretation depends on test type)
    pub value: Option<f64>,
    /// The unit of the value (if applicable)
    pub unit: Option<String>,
    /// Pass/Fail/Needs Review
    pub status: QcStatus,
    /// Notes about the result
    pub notes: Option<String>,
    /// When the test was performed
    pub performed_at: DateTime<Utc>,
    /// Who performed the test
    pub performed_by: String,
}

impl QcResult {
    /// Creates a new QC result that passed.
    pub fn passed(
        test_type: QcTestType,
        value: Option<f64>,
        unit: Option<String>,
        performed_by: impl Into<String>,
    ) -> Self {
        Self {
            test_type,
            value,
            unit,
            status: QcStatus::Passed,
            notes: None,
            performed_at: Utc::now(),
            performed_by: performed_by.into(),
        }
    }

    /// Creates a new QC result that failed.
    pub fn failed(
        test_type: QcTestType,
        value: Option<f64>,
        unit: Option<String>,
        performed_by: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            test_type,
            value,
            unit,
            status: QcStatus::Failed,
            notes: Some(reason.into()),
            performed_at: Utc::now(),
            performed_by: performed_by.into(),
        }
    }

    /// Checks if this result meets a numeric threshold.
    pub fn meets_threshold(&self, min: f64, max: f64) -> bool {
        match self.value {
            Some(v) => v >= min && v <= max,
            None => false,
        }
    }
}

impl fmt::Display for QcResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.value, &self.unit) {
            (Some(v), Some(u)) => write!(f, "{}: {:.2} {} - {}", self.test_type, v, u, self.status),
            (Some(v), None) => write!(f, "{}: {:.2} - {}", self.test_type, v, self.status),
            _ => write!(f, "{}: {}", self.test_type, self.status),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qc_status_progression() {
        assert!(QcStatus::Passed.allows_progression());
        assert!(!QcStatus::Failed.allows_progression());
        assert!(!QcStatus::Ready.allows_progression());
    }

    #[test]
    fn test_qc_result_threshold() {
        let result = QcResult::passed(
            QcTestType::Qubit,
            Some(25.0),
            Some("ng/µL".to_string()),
            "lab_user",
        );
        assert!(result.meets_threshold(10.0, 50.0));
        assert!(!result.meets_threshold(30.0, 50.0));
    }

    #[test]
    fn test_qc_result_display() {
        let result = QcResult::passed(
            QcTestType::Qubit,
            Some(25.5),
            Some("ng/µL".to_string()),
            "lab_user",
        );
        assert!(result.to_string().contains("Qubit"));
        assert!(result.to_string().contains("25.50"));
        assert!(result.to_string().contains("Passed"));
    }
}

