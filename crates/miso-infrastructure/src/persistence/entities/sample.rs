//! SeaORM entity for the Sample table.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Sample database entity.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sample")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(column_type = "String(Some(255))")]
    pub name: String,

    #[sea_orm(column_type = "String(Some(50))", unique)]
    pub barcode: String,

    pub project_id: i32,

    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,

    /// "plain" or "detailed"
    #[sea_orm(column_type = "String(Some(20))")]
    pub sample_mode: String,

    /// Sample class (for detailed mode)
    #[sea_orm(column_type = "String(Some(50))", nullable)]
    pub sample_class: Option<String>,

    /// Parent sample ID (for detailed hierarchy)
    #[sea_orm(nullable)]
    pub parent_id: Option<i32>,

    /// Scientific name (for plain mode)
    #[sea_orm(column_type = "String(Some(255))", nullable)]
    pub scientific_name: Option<String>,

    /// Volume in microliters
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", nullable)]
    pub volume: Option<Decimal>,

    /// Concentration in ng/µL
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", nullable)]
    pub concentration: Option<Decimal>,

    /// QC status
    #[sea_orm(column_type = "String(Some(20))")]
    pub qc_status: String,

    #[sea_orm(nullable)]
    pub received_at: Option<DateTimeUtc>,

    #[sea_orm(column_type = "String(Some(255))")]
    pub created_by: String,

    pub created_at: DateTimeUtc,

    pub updated_at: DateTimeUtc,

    #[sea_orm(default_value = "false")]
    pub archived: bool,

    // Detailed sample fields
    #[sea_orm(column_type = "String(Some(255))", nullable)]
    pub external_name: Option<String>,

    #[sea_orm(column_type = "String(Some(100))", nullable)]
    pub tissue_origin: Option<String>,

    #[sea_orm(column_type = "String(Some(100))", nullable)]
    pub tissue_type: Option<String>,

    #[sea_orm(column_type = "String(Some(50))", nullable)]
    pub analyte_type: Option<String>,
}

/// Database relations for Sample.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,

    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id"
    )]
    Parent,

    #[sea_orm(has_many = "Entity")]
    Children,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

