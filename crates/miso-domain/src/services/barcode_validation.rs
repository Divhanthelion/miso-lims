//! Barcode validation service.
//!
//! Validates barcode formats and patterns for different entity types.

use crate::errors::BarcodeError;
use crate::value_objects::Barcode;

/// Configurable barcode validation rules.
#[derive(Debug, Clone)]
pub struct BarcodeValidationRules {
    /// Required prefix for the barcode
    pub prefix: Option<String>,
    /// Minimum length
    pub min_length: Option<usize>,
    /// Maximum length
    pub max_length: Option<usize>,
    /// Pattern description (for error messages)
    pub pattern_description: String,
}

impl Default for BarcodeValidationRules {
    fn default() -> Self {
        Self {
            prefix: None,
            min_length: Some(3),
            max_length: Some(50),
            pattern_description: "alphanumeric with hyphens and underscores".to_string(),
        }
    }
}

impl BarcodeValidationRules {
    /// Creates rules for sample barcodes.
    pub fn for_samples() -> Self {
        Self {
            prefix: Some("SAM".to_string()),
            min_length: Some(6),
            max_length: Some(20),
            pattern_description: "SAM-XXXXXX".to_string(),
        }
    }

    /// Creates rules for library barcodes.
    pub fn for_libraries() -> Self {
        Self {
            prefix: Some("LIB".to_string()),
            min_length: Some(6),
            max_length: Some(20),
            pattern_description: "LIB-XXXXXX".to_string(),
        }
    }

    /// Creates rules for pool barcodes.
    pub fn for_pools() -> Self {
        Self {
            prefix: Some("POOL".to_string()),
            min_length: Some(7),
            max_length: Some(20),
            pattern_description: "POOL-XXXXXX".to_string(),
        }
    }

    /// Creates rules for box barcodes.
    pub fn for_boxes() -> Self {
        Self {
            prefix: Some("BOX".to_string()),
            min_length: Some(6),
            max_length: Some(20),
            pattern_description: "BOX-XXXXXX".to_string(),
        }
    }
}

/// A service for validating barcodes according to configurable rules.
#[derive(Debug, Clone, Default)]
pub struct BarcodeValidator {
    /// Default rules when no entity-specific rules are specified
    default_rules: BarcodeValidationRules,
}

impl BarcodeValidator {
    /// Creates a new barcode validator with default rules.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new barcode validator with custom default rules.
    pub fn with_rules(rules: BarcodeValidationRules) -> Self {
        Self {
            default_rules: rules,
        }
    }

    /// Validates a barcode string using default rules.
    pub fn validate(&self, barcode: &str) -> Result<Barcode, BarcodeError> {
        self.validate_with_rules(barcode, &self.default_rules)
    }

    /// Validates a barcode string using specific rules.
    pub fn validate_with_rules(
        &self,
        barcode: &str,
        rules: &BarcodeValidationRules,
    ) -> Result<Barcode, BarcodeError> {
        let barcode_str = barcode.trim();

        // Check empty
        if barcode_str.is_empty() {
            return Err(BarcodeError::Empty);
        }

        // Check length constraints
        if let Some(min) = rules.min_length {
            if barcode_str.len() < min {
                return Err(BarcodeError::InvalidFormat(format!(
                    "Barcode must be at least {} characters",
                    min
                )));
            }
        }

        if let Some(max) = rules.max_length {
            if barcode_str.len() > max {
                return Err(BarcodeError::InvalidFormat(format!(
                    "Barcode must be at most {} characters",
                    max
                )));
            }
        }

        // Check prefix
        if let Some(prefix) = &rules.prefix {
            if !barcode_str.starts_with(prefix) {
                return Err(BarcodeError::PatternMismatch(
                    barcode_str.to_string(),
                    rules.pattern_description.clone(),
                ));
            }
        }

        // Create the validated barcode
        Barcode::new(barcode_str)
    }

    /// Validates a sample barcode.
    pub fn validate_sample(&self, barcode: &str) -> Result<Barcode, BarcodeError> {
        self.validate_with_rules(barcode, &BarcodeValidationRules::for_samples())
    }

    /// Validates a library barcode.
    pub fn validate_library(&self, barcode: &str) -> Result<Barcode, BarcodeError> {
        self.validate_with_rules(barcode, &BarcodeValidationRules::for_libraries())
    }

    /// Validates a pool barcode.
    pub fn validate_pool(&self, barcode: &str) -> Result<Barcode, BarcodeError> {
        self.validate_with_rules(barcode, &BarcodeValidationRules::for_pools())
    }

    /// Generates a unique barcode with the given prefix.
    ///
    /// This is a simple implementation - in production you'd check uniqueness
    /// against the database.
    pub fn generate_barcode(&self, prefix: &str) -> Barcode {
        let unique_part = uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(8)
            .collect::<String>()
            .to_uppercase();

        Barcode::new_unchecked(format!("{}-{}", prefix, unique_part))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_default() {
        let validator = BarcodeValidator::new();
        let result = validator.validate("ABC-123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_sample_valid() {
        let validator = BarcodeValidator::new();
        let result = validator.validate_sample("SAM-12345");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_sample_wrong_prefix() {
        let validator = BarcodeValidator::new();
        let result = validator.validate_sample("LIB-12345");
        assert!(matches!(result, Err(BarcodeError::PatternMismatch(_, _))));
    }

    #[test]
    fn test_validate_sample_too_short() {
        let validator = BarcodeValidator::new();
        let result = validator.validate_sample("SAM-1");
        assert!(matches!(result, Err(BarcodeError::InvalidFormat(_))));
    }

    #[test]
    fn test_generate_barcode() {
        let validator = BarcodeValidator::new();
        let barcode = validator.generate_barcode("SAM");
        assert!(barcode.as_str().starts_with("SAM-"));
        assert!(barcode.len() >= 12); // SAM- + 8 chars
    }
}

