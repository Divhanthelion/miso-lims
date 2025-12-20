//! Box position and dimension value objects for storage management.

use crate::errors::StorageError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the dimensions of a storage box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dimension {
    rows: u8,
    cols: u8,
}

impl Dimension {
    /// Standard 96-well plate (8x12)
    pub const PLATE_96: Self = Self { rows: 8, cols: 12 };

    /// Standard 384-well plate (16x24)
    pub const PLATE_384: Self = Self { rows: 16, cols: 24 };

    /// Standard 9x9 cryobox (81 positions)
    pub const CRYOBOX_9X9: Self = Self { rows: 9, cols: 9 };

    /// Standard 10x10 cryobox (100 positions)
    pub const CRYOBOX_10X10: Self = Self { rows: 10, cols: 10 };

    /// Creates a new dimension.
    ///
    /// # Panics
    ///
    /// Panics if rows or cols is 0 or greater than 26 (max letter rows).
    pub fn new(rows: u8, cols: u8) -> Self {
        assert!(rows > 0 && rows <= 26, "Rows must be between 1 and 26");
        assert!(cols > 0, "Cols must be greater than 0");
        Self { rows, cols }
    }

    /// Returns the number of rows.
    pub fn rows(&self) -> u8 {
        self.rows
    }

    /// Returns the number of columns.
    pub fn cols(&self) -> u8 {
        self.cols
    }

    /// Returns the total capacity (rows * cols).
    pub fn capacity(&self) -> usize {
        self.rows as usize * self.cols as usize
    }

    /// Checks if a position is valid for these dimensions.
    pub fn is_valid_position(&self, row: char, col: u8) -> bool {
        let row_upper = row.to_ascii_uppercase();
        let row_num = row_upper as u8 - b'A';
        row_num < self.rows && col >= 1 && col <= self.cols
    }

    /// Converts a linear index (0-based) to a position.
    pub fn index_to_position(&self, index: usize) -> Option<BoxPosition> {
        if index >= self.capacity() {
            return None;
        }
        let row = (index / self.cols as usize) as u8;
        let col = (index % self.cols as usize) as u8 + 1;
        let row_char = (b'A' + row) as char;
        Some(BoxPosition::new_unchecked(row_char, col))
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.rows, self.cols)
    }
}

/// Represents a position within a storage box.
///
/// Positions use row letters (A, B, C...) and column numbers (1, 2, 3...).
/// For example: A1, B12, H8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoxPosition {
    row: char,
    col: u8,
}

impl BoxPosition {
    /// Creates a new position, validating against the given dimensions.
    pub fn new(row: char, col: u8, dimension: &Dimension) -> Result<Self, StorageError> {
        let row_upper = row.to_ascii_uppercase();

        if !dimension.is_valid_position(row_upper, col) {
            return Err(StorageError::InvalidPosition {
                row: row_upper,
                col,
                rows: dimension.rows(),
                cols: dimension.cols(),
            });
        }

        Ok(Self {
            row: row_upper,
            col,
        })
    }

    /// Creates a position without validation (for trusted sources).
    pub fn new_unchecked(row: char, col: u8) -> Self {
        Self {
            row: row.to_ascii_uppercase(),
            col,
        }
    }

    /// Returns the row letter.
    pub fn row(&self) -> char {
        self.row
    }

    /// Returns the column number (1-based).
    pub fn col(&self) -> u8 {
        self.col
    }

    /// Returns the row as a 0-based index.
    pub fn row_index(&self) -> u8 {
        self.row as u8 - b'A'
    }

    /// Converts to a linear index (0-based) for the given dimensions.
    pub fn to_index(&self, dimension: &Dimension) -> usize {
        self.row_index() as usize * dimension.cols() as usize + (self.col - 1) as usize
    }

    /// Parses a position string like "A1", "B12", "H08".
    pub fn parse(s: &str, dimension: &Dimension) -> Result<Self, StorageError> {
        let s = s.trim().to_uppercase();
        if s.is_empty() {
            return Err(StorageError::InvalidPosition {
                row: ' ',
                col: 0,
                rows: dimension.rows(),
                cols: dimension.cols(),
            });
        }

        let row = s.chars().next().unwrap();
        let col: u8 = s[1..]
            .parse()
            .map_err(|_| StorageError::InvalidPosition {
                row,
                col: 0,
                rows: dimension.rows(),
                cols: dimension.cols(),
            })?;

        Self::new(row, col, dimension)
    }
}

impl fmt::Display for BoxPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.row, self.col)
    }
}

impl PartialOrd for BoxPosition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BoxPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.row.cmp(&other.row) {
            std::cmp::Ordering::Equal => self.col.cmp(&other.col),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_96_well() {
        let dim = Dimension::PLATE_96;
        assert_eq!(dim.rows(), 8);
        assert_eq!(dim.cols(), 12);
        assert_eq!(dim.capacity(), 96);
    }

    #[test]
    fn test_position_valid() {
        let dim = Dimension::PLATE_96;
        let pos = BoxPosition::new('A', 1, &dim).unwrap();
        assert_eq!(pos.row(), 'A');
        assert_eq!(pos.col(), 1);
    }

    #[test]
    fn test_position_invalid_row() {
        let dim = Dimension::PLATE_96; // 8 rows (A-H)
        let result = BoxPosition::new('I', 1, &dim);
        assert!(result.is_err());
    }

    #[test]
    fn test_position_invalid_col() {
        let dim = Dimension::PLATE_96; // 12 columns
        let result = BoxPosition::new('A', 13, &dim);
        assert!(result.is_err());
    }

    #[test]
    fn test_position_parse() {
        let dim = Dimension::PLATE_96;
        let pos = BoxPosition::parse("B12", &dim).unwrap();
        assert_eq!(pos.row(), 'B');
        assert_eq!(pos.col(), 12);
    }

    #[test]
    fn test_position_to_index() {
        let dim = Dimension::PLATE_96;
        let pos = BoxPosition::new('A', 1, &dim).unwrap();
        assert_eq!(pos.to_index(&dim), 0);

        let pos2 = BoxPosition::new('B', 1, &dim).unwrap();
        assert_eq!(pos2.to_index(&dim), 12);
    }

    #[test]
    fn test_index_to_position() {
        let dim = Dimension::PLATE_96;
        let pos = dim.index_to_position(0).unwrap();
        assert_eq!(pos.to_string(), "A1");

        let pos2 = dim.index_to_position(12).unwrap();
        assert_eq!(pos2.to_string(), "B1");

        let pos3 = dim.index_to_position(95).unwrap();
        assert_eq!(pos3.to_string(), "H12");
    }
}

