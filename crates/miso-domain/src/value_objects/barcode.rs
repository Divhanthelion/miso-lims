//! Barcode value object for sample and container identification.

use crate::errors::BarcodeError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A validated barcode string.
///
/// Barcodes are used throughout the LIMS to uniquely identify samples,
/// libraries, pools, containers, and other physical entities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Barcode(String);

impl Barcode {
    /// Creates a new barcode after validation.
    ///
    /// # Errors
    ///
    /// Returns an error if the barcode is empty or contains invalid characters.
    pub fn new(value: impl Into<String>) -> Result<Self, BarcodeError> {
        let value = value.into();

        if value.is_empty() {
            return Err(BarcodeError::Empty);
        }

        // Allow alphanumeric, hyphens, and underscores
        if !value
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(BarcodeError::InvalidCharacters(value));
        }

        Ok(Self(value))
    }

    /// Creates a barcode without validation (for trusted sources like DB).
    ///
    /// # Safety
    ///
    /// This bypasses validation. Only use when loading from trusted sources.
    pub fn new_unchecked(value: String) -> Self {
        Self(value)
    }

    /// Returns the barcode as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the barcode and returns the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Checks if this barcode matches a given pattern (prefix).
    pub fn matches_prefix(&self, prefix: &str) -> bool {
        self.0.starts_with(prefix)
    }

    /// Returns the length of the barcode.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the barcode is empty (should not happen with validation).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Barcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Barcode {
    type Error = BarcodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Barcode {
    type Error = BarcodeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl AsRef<str> for Barcode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_barcode() {
        let barcode = Barcode::new("SAM-001_A").unwrap();
        assert_eq!(barcode.as_str(), "SAM-001_A");
    }

    #[test]
    fn test_empty_barcode() {
        let result = Barcode::new("");
        assert!(matches!(result, Err(BarcodeError::Empty)));
    }

    #[test]
    fn test_invalid_characters() {
        let result = Barcode::new("SAM@001");
        assert!(matches!(result, Err(BarcodeError::InvalidCharacters(_))));
    }

    #[test]
    fn test_matches_prefix() {
        let barcode = Barcode::new("LIB-001").unwrap();
        assert!(barcode.matches_prefix("LIB"));
        assert!(!barcode.matches_prefix("SAM"));
    }
}

