//! Storage box entity for sample location tracking.
//!
//! The LIMS tracks the physical location of samples through a hierarchy:
//! Freezer -> Shelf -> Rack -> Box -> Position

use crate::errors::StorageError;
use crate::value_objects::{BoxPosition, Dimension};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::EntityId;

/// The type of item that can be stored in a box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorableType {
    Sample,
    Library,
    LibraryAliquot,
    Pool,
}

impl std::fmt::Display for StorableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sample => write!(f, "Sample"),
            Self::Library => write!(f, "Library"),
            Self::LibraryAliquot => write!(f, "Library Aliquot"),
            Self::Pool => write!(f, "Pool"),
        }
    }
}

/// A storable item reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StorableItem {
    /// The type of item
    pub item_type: StorableType,
    /// The item's ID
    pub item_id: EntityId,
}

impl StorableItem {
    /// Creates a new storable item reference.
    pub fn new(item_type: StorableType, item_id: EntityId) -> Self {
        Self { item_type, item_id }
    }

    /// Creates a sample reference.
    pub fn sample(id: EntityId) -> Self {
        Self::new(StorableType::Sample, id)
    }

    /// Creates a library reference.
    pub fn library(id: EntityId) -> Self {
        Self::new(StorableType::Library, id)
    }

    /// Creates a pool reference.
    pub fn pool(id: EntityId) -> Self {
        Self::new(StorableType::Pool, id)
    }
}

/// A storage location in the hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageLocation {
    /// Freezer/Room name
    pub freezer: Option<String>,
    /// Shelf within the freezer
    pub shelf: Option<String>,
    /// Rack on the shelf
    pub rack: Option<String>,
    /// Temperature (°C)
    pub temperature: Option<i8>,
}

impl StorageLocation {
    /// Creates a new storage location.
    pub fn new() -> Self {
        Self {
            freezer: None,
            shelf: None,
            rack: None,
            temperature: None,
        }
    }

    /// Sets the full path.
    pub fn with_path(
        freezer: impl Into<String>,
        shelf: impl Into<String>,
        rack: impl Into<String>,
    ) -> Self {
        Self {
            freezer: Some(freezer.into()),
            shelf: Some(shelf.into()),
            rack: Some(rack.into()),
            temperature: None,
        }
    }

    /// Returns a formatted path string.
    pub fn path(&self) -> String {
        let parts: Vec<&str> = [
            self.freezer.as_deref(),
            self.shelf.as_deref(),
            self.rack.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();

        parts.join(" / ")
    }
}

impl Default for StorageLocation {
    fn default() -> Self {
        Self::new()
    }
}

/// A storage box containing samples/libraries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageBox {
    /// Unique identifier
    pub id: EntityId,
    /// Box name/label
    pub name: String,
    /// Barcode for the box itself
    pub barcode: Option<String>,
    /// Box dimensions (rows x cols)
    pub dimension: Dimension,
    /// Location in the storage hierarchy
    pub location: StorageLocation,
    /// The type of items this box can hold
    pub storable_type: StorableType,
    /// Map of position -> item
    contents: HashMap<BoxPosition, StorableItem>,
    /// Description/notes
    pub description: Option<String>,
    /// When this record was created
    pub created_at: DateTime<Utc>,
    /// When this record was last modified
    pub updated_at: DateTime<Utc>,
}

impl StorageBox {
    /// Creates a new empty storage box.
    pub fn new(
        id: EntityId,
        name: String,
        dimension: Dimension,
        storable_type: StorableType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            barcode: None,
            dimension,
            location: StorageLocation::new(),
            storable_type,
            contents: HashMap::new(),
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a standard 9x9 sample box.
    pub fn sample_box_9x9(id: EntityId, name: String) -> Self {
        Self::new(id, name, Dimension::CRYOBOX_9X9, StorableType::Sample)
    }

    /// Creates a standard 96-well plate.
    pub fn plate_96(id: EntityId, name: String, storable_type: StorableType) -> Self {
        Self::new(id, name, Dimension::PLATE_96, storable_type)
    }

    /// Returns the number of items in the box.
    pub fn item_count(&self) -> usize {
        self.contents.len()
    }

    /// Returns the total capacity.
    pub fn capacity(&self) -> usize {
        self.dimension.capacity()
    }

    /// Returns true if the box is full.
    pub fn is_full(&self) -> bool {
        self.contents.len() >= self.capacity()
    }

    /// Returns true if the box is empty.
    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    /// Returns the fill percentage (0-100).
    pub fn fill_percent(&self) -> f64 {
        (self.contents.len() as f64 / self.capacity() as f64) * 100.0
    }

    /// Checks if a position is occupied.
    pub fn is_occupied(&self, position: &BoxPosition) -> bool {
        self.contents.contains_key(position)
    }

    /// Gets the item at a position.
    pub fn get_item(&self, position: &BoxPosition) -> Option<&StorableItem> {
        self.contents.get(position)
    }

    /// Places an item in the box at the specified position.
    pub fn place_item(
        &mut self,
        position: BoxPosition,
        item: StorableItem,
    ) -> Result<(), StorageError> {
        // Validate item type
        if item.item_type != self.storable_type {
            return Err(StorageError::IncompatibleStorageTypes);
        }

        // Validate position
        if !self.dimension.is_valid_position(position.row(), position.col()) {
            return Err(StorageError::InvalidPosition {
                row: position.row(),
                col: position.col(),
                rows: self.dimension.rows(),
                cols: self.dimension.cols(),
            });
        }

        // Check if occupied
        if self.is_occupied(&position) {
            return Err(StorageError::PositionOccupied {
                box_name: self.name.clone(),
                row: position.row(),
                col: position.col(),
            });
        }

        self.contents.insert(position, item);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Removes an item from the specified position.
    pub fn remove_item(&mut self, position: &BoxPosition) -> Option<StorableItem> {
        let item = self.contents.remove(position);
        if item.is_some() {
            self.updated_at = Utc::now();
        }
        item
    }

    /// Moves an item from one position to another within this box.
    pub fn move_item(
        &mut self,
        from: &BoxPosition,
        to: BoxPosition,
    ) -> Result<(), StorageError> {
        if self.is_occupied(&to) {
            return Err(StorageError::PositionOccupied {
                box_name: self.name.clone(),
                row: to.row(),
                col: to.col(),
            });
        }

        let item = self.contents.remove(from).ok_or_else(|| {
            StorageError::ItemNotInBox(
                format!("{}{}", from.row(), from.col()),
                self.name.clone(),
            )
        })?;

        self.contents.insert(to, item);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Finds the first empty position.
    pub fn find_empty_position(&self) -> Option<BoxPosition> {
        for idx in 0..self.capacity() {
            if let Some(pos) = self.dimension.index_to_position(idx) {
                if !self.is_occupied(&pos) {
                    return Some(pos);
                }
            }
        }
        None
    }

    /// Returns all contents as a vector of (position, item) tuples.
    pub fn all_contents(&self) -> Vec<(&BoxPosition, &StorableItem)> {
        self.contents.iter().collect()
    }

    /// Returns positions of all items of a specific ID.
    pub fn find_item(&self, item_id: EntityId) -> Vec<BoxPosition> {
        self.contents
            .iter()
            .filter(|(_, item)| item.item_id == item_id)
            .map(|(pos, _)| *pos)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_creation() {
        let storage_box = StorageBox::sample_box_9x9(1, "BOX001".to_string());
        assert_eq!(storage_box.capacity(), 81);
        assert!(storage_box.is_empty());
    }

    #[test]
    fn test_place_and_retrieve_item() {
        let mut storage_box = StorageBox::sample_box_9x9(1, "BOX001".to_string());
        let pos = BoxPosition::new('A', 1, &storage_box.dimension).unwrap();

        storage_box
            .place_item(pos, StorableItem::sample(1))
            .unwrap();

        assert!(storage_box.is_occupied(&pos));
        assert_eq!(storage_box.item_count(), 1);

        let item = storage_box.get_item(&pos).unwrap();
        assert_eq!(item.item_id, 1);
    }

    #[test]
    fn test_position_occupied_error() {
        let mut storage_box = StorageBox::sample_box_9x9(1, "BOX001".to_string());
        let pos = BoxPosition::new('A', 1, &storage_box.dimension).unwrap();

        storage_box
            .place_item(pos, StorableItem::sample(1))
            .unwrap();

        let result = storage_box.place_item(pos, StorableItem::sample(2));
        assert!(matches!(result, Err(StorageError::PositionOccupied { .. })));
    }

    #[test]
    fn test_move_item() {
        let mut storage_box = StorageBox::sample_box_9x9(1, "BOX001".to_string());
        let from = BoxPosition::new('A', 1, &storage_box.dimension).unwrap();
        let to = BoxPosition::new('B', 2, &storage_box.dimension).unwrap();

        storage_box
            .place_item(from, StorableItem::sample(1))
            .unwrap();

        storage_box.move_item(&from, to).unwrap();

        assert!(!storage_box.is_occupied(&from));
        assert!(storage_box.is_occupied(&to));
    }

    #[test]
    fn test_find_empty_position() {
        let mut storage_box = StorageBox::new(
            1,
            "BOX".to_string(),
            Dimension::new(2, 2),
            StorableType::Sample,
        );

        // Fill first two positions
        storage_box
            .place_item(
                BoxPosition::new('A', 1, &storage_box.dimension).unwrap(),
                StorableItem::sample(1),
            )
            .unwrap();
        storage_box
            .place_item(
                BoxPosition::new('A', 2, &storage_box.dimension).unwrap(),
                StorableItem::sample(2),
            )
            .unwrap();

        let empty = storage_box.find_empty_position().unwrap();
        assert_eq!(empty.row(), 'B');
        assert_eq!(empty.col(), 1);
    }

    #[test]
    fn test_storage_location() {
        let loc = StorageLocation::with_path("Freezer-1", "Shelf-A", "Rack-01");
        assert_eq!(loc.path(), "Freezer-1 / Shelf-A / Rack-01");
    }
}

