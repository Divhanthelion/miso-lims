//! Concentration value object for DNA/RNA measurements.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Units of concentration measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConcentrationUnit {
    /// Nanograms per microliter (ng/µL) - most common for DNA
    NgPerUl,
    /// Picomolar (pM) - common for libraries
    Picomolar,
    /// Nanomolar (nM)
    Nanomolar,
    /// Micrograms per milliliter (µg/mL)
    UgPerMl,
}

impl fmt::Display for ConcentrationUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NgPerUl => write!(f, "ng/µL"),
            Self::Picomolar => write!(f, "pM"),
            Self::Nanomolar => write!(f, "nM"),
            Self::UgPerMl => write!(f, "µg/mL"),
        }
    }
}

/// A concentration measurement with its unit.
///
/// Concentrations are critical for library preparation and pooling calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Concentration {
    value: f64,
    unit: ConcentrationUnit,
}

impl Concentration {
    /// Creates a new concentration.
    ///
    /// # Panics
    ///
    /// Panics if value is negative or NaN.
    pub fn new(value: f64, unit: ConcentrationUnit) -> Self {
        assert!(value >= 0.0 && !value.is_nan(), "Concentration must be non-negative");
        Self { value, unit }
    }

    /// Creates a concentration in ng/µL.
    pub fn ng_per_ul(value: f64) -> Self {
        Self::new(value, ConcentrationUnit::NgPerUl)
    }

    /// Creates a concentration in nM.
    pub fn nanomolar(value: f64) -> Self {
        Self::new(value, ConcentrationUnit::Nanomolar)
    }

    /// Creates a concentration in pM.
    pub fn picomolar(value: f64) -> Self {
        Self::new(value, ConcentrationUnit::Picomolar)
    }

    /// Returns the numeric value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Returns the unit of measurement.
    pub fn unit(&self) -> ConcentrationUnit {
        self.unit
    }

    /// Converts to nanomolar (if applicable).
    ///
    /// Note: This requires additional information (like fragment size) for
    /// accurate ng/µL to nM conversion. This is a simplified version.
    pub fn to_nanomolar(&self, fragment_size_bp: Option<u32>) -> Option<Self> {
        match self.unit {
            ConcentrationUnit::Nanomolar => Some(*self),
            ConcentrationUnit::Picomolar => Some(Self::nanomolar(self.value / 1000.0)),
            ConcentrationUnit::NgPerUl => {
                // Formula: nM = (ng/µL * 1,000,000) / (660 * fragment_size)
                fragment_size_bp.map(|size| {
                    let nm = (self.value * 1_000_000.0) / (660.0 * size as f64);
                    Self::nanomolar(nm)
                })
            }
            ConcentrationUnit::UgPerMl => None, // Would need molecular weight
        }
    }

    /// Checks if this concentration meets a minimum threshold.
    pub fn meets_threshold(&self, threshold: f64, unit: ConcentrationUnit) -> bool {
        if self.unit == unit {
            self.value >= threshold
        } else {
            // Would need conversion logic for cross-unit comparison
            false
        }
    }
}

impl fmt::Display for Concentration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.value, self.unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concentration_creation() {
        let conc = Concentration::ng_per_ul(25.5);
        assert_eq!(conc.value(), 25.5);
        assert_eq!(conc.unit(), ConcentrationUnit::NgPerUl);
    }

    #[test]
    fn test_concentration_display() {
        let conc = Concentration::nanomolar(10.0);
        assert_eq!(conc.to_string(), "10.00 nM");
    }

    #[test]
    fn test_picomolar_to_nanomolar() {
        let pm = Concentration::picomolar(1000.0);
        let nm = pm.to_nanomolar(None).unwrap();
        assert_eq!(nm.value(), 1.0);
        assert_eq!(nm.unit(), ConcentrationUnit::Nanomolar);
    }

    #[test]
    #[should_panic]
    fn test_negative_concentration() {
        Concentration::ng_per_ul(-1.0);
    }
}

