//! Volume value object for liquid handling.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// Units of volume measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumeUnit {
    /// Microliters (µL) - most common for DNA work
    Microliters,
    /// Milliliters (mL)
    Milliliters,
    /// Nanoliters (nL) - for some liquid handlers
    Nanoliters,
}

impl VolumeUnit {
    /// Conversion factor to microliters.
    fn to_ul_factor(&self) -> f64 {
        match self {
            Self::Microliters => 1.0,
            Self::Milliliters => 1000.0,
            Self::Nanoliters => 0.001,
        }
    }
}

impl fmt::Display for VolumeUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Microliters => write!(f, "µL"),
            Self::Milliliters => write!(f, "mL"),
            Self::Nanoliters => write!(f, "nL"),
        }
    }
}

/// A volume measurement with its unit.
///
/// Volumes are critical for tracking sample consumption during library
/// preparation and pooling.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Volume {
    /// Value stored internally in microliters for consistency
    value_ul: f64,
    /// Original unit for display purposes
    display_unit: VolumeUnit,
}

impl Volume {
    /// Creates a new volume.
    ///
    /// # Panics
    ///
    /// Panics if value is negative or NaN.
    pub fn new(value: f64, unit: VolumeUnit) -> Self {
        assert!(value >= 0.0 && !value.is_nan(), "Volume must be non-negative");
        Self {
            value_ul: value * unit.to_ul_factor(),
            display_unit: unit,
        }
    }

    /// Creates a volume in microliters.
    pub fn microliters(value: f64) -> Self {
        Self::new(value, VolumeUnit::Microliters)
    }

    /// Creates a volume in milliliters.
    pub fn milliliters(value: f64) -> Self {
        Self::new(value, VolumeUnit::Milliliters)
    }

    /// Creates a zero volume.
    pub fn zero() -> Self {
        Self {
            value_ul: 0.0,
            display_unit: VolumeUnit::Microliters,
        }
    }

    /// Returns the value in microliters.
    pub fn as_microliters(&self) -> f64 {
        self.value_ul
    }

    /// Returns the value in milliliters.
    pub fn as_milliliters(&self) -> f64 {
        self.value_ul / 1000.0
    }

    /// Returns the display unit.
    pub fn unit(&self) -> VolumeUnit {
        self.display_unit
    }

    /// Returns the value in the display unit.
    pub fn value(&self) -> f64 {
        self.value_ul / self.display_unit.to_ul_factor()
    }

    /// Returns true if this volume is zero.
    pub fn is_zero(&self) -> bool {
        self.value_ul == 0.0
    }

    /// Returns true if this volume is less than the threshold.
    pub fn is_below(&self, threshold: Volume) -> bool {
        self.value_ul < threshold.value_ul
    }

    /// Checks if there is sufficient volume for a withdrawal.
    pub fn has_sufficient(&self, required: Volume) -> bool {
        self.value_ul >= required.value_ul
    }

    /// Subtracts a volume, returning the remaining volume.
    ///
    /// Returns None if the subtraction would result in negative volume.
    pub fn subtract(&self, amount: Volume) -> Option<Self> {
        let remaining = self.value_ul - amount.value_ul;
        if remaining < 0.0 {
            None
        } else {
            Some(Self {
                value_ul: remaining,
                display_unit: self.display_unit,
            })
        }
    }
}

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.value(), self.display_unit)
    }
}

impl Add for Volume {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value_ul: self.value_ul + rhs.value_ul,
            display_unit: self.display_unit,
        }
    }
}

impl Sub for Volume {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.subtract(rhs)
    }
}

impl PartialOrd for Volume {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value_ul.partial_cmp(&other.value_ul)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_creation() {
        let vol = Volume::microliters(50.0);
        assert_eq!(vol.as_microliters(), 50.0);
    }

    #[test]
    fn test_volume_conversion() {
        let vol = Volume::milliliters(1.0);
        assert_eq!(vol.as_microliters(), 1000.0);
    }

    #[test]
    fn test_volume_subtraction() {
        let vol = Volume::microliters(100.0);
        let remaining = vol.subtract(Volume::microliters(30.0)).unwrap();
        assert_eq!(remaining.as_microliters(), 70.0);
    }

    #[test]
    fn test_volume_insufficient() {
        let vol = Volume::microliters(10.0);
        let result = vol.subtract(Volume::microliters(20.0));
        assert!(result.is_none());
    }

    #[test]
    fn test_volume_addition() {
        let v1 = Volume::microliters(50.0);
        let v2 = Volume::microliters(30.0);
        let total = v1 + v2;
        assert_eq!(total.as_microliters(), 80.0);
    }

    #[test]
    #[should_panic]
    fn test_negative_volume() {
        Volume::microliters(-10.0);
    }
}

