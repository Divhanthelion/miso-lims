//! SeaORM entity for the Project table.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Project database entity.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "project")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(column_type = "String(Some(50))", unique)]
    pub code: String,

    #[sea_orm(column_type = "String(Some(255))")]
    pub name: String,

    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,

    #[sea_orm(column_type = "String(Some(20))")]
    pub status: String,

    #[sea_orm(column_type = "String(Some(255))", nullable)]
    pub pi_name: Option<String>,

    #[sea_orm(column_type = "String(Some(255))", nullable)]
    pub pi_email: Option<String>,

    #[sea_orm(column_type = "String(Some(100))", nullable)]
    pub reference_number: Option<String>,

    #[sea_orm(nullable)]
    pub target_sample_count: Option<i32>,

    #[sea_orm(default_value = "0")]
    pub sample_count: i32,

    pub created_at: DateTimeUtc,

    #[sea_orm(column_type = "String(Some(255))")]
    pub created_by: String,

    pub updated_at: DateTimeUtc,

    #[sea_orm(nullable)]
    pub due_date: Option<DateTimeUtc>,
}

/// Database relations for Project.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::sample::Entity")]
    Sample,
}

impl Related<super::sample::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sample.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for miso_domain::entities::Project {
    fn from(model: Model) -> Self {
        use miso_domain::entities::ProjectStatus;

        let status = match model.status.as_str() {
            "pending" => ProjectStatus::Pending,
            "active" => ProjectStatus::Active,
            "on_hold" => ProjectStatus::OnHold,
            "completed" => ProjectStatus::Completed,
            "cancelled" => ProjectStatus::Cancelled,
            _ => ProjectStatus::Pending,
        };

        Self {
            id: model.id,
            code: model.code,
            name: model.name,
            description: model.description,
            status,
            pi_name: model.pi_name,
            pi_email: model.pi_email,
            reference_number: model.reference_number,
            target_sample_count: model.target_sample_count.map(|v| v as u32),
            sample_count: model.sample_count as u32,
            created_at: model.created_at,
            created_by: model.created_by,
            updated_at: model.updated_at,
            due_date: model.due_date,
        }
    }
}

impl From<&miso_domain::entities::Project> for ActiveModel {
    fn from(project: &miso_domain::entities::Project) -> Self {
        use miso_domain::entities::ProjectStatus;
        use sea_orm::ActiveValue;

        let status = match project.status {
            ProjectStatus::Pending => "pending",
            ProjectStatus::Active => "active",
            ProjectStatus::OnHold => "on_hold",
            ProjectStatus::Completed => "completed",
            ProjectStatus::Cancelled => "cancelled",
        };

        Self {
            id: ActiveValue::Set(project.id),
            code: ActiveValue::Set(project.code.clone()),
            name: ActiveValue::Set(project.name.clone()),
            description: ActiveValue::Set(project.description.clone()),
            status: ActiveValue::Set(status.to_string()),
            pi_name: ActiveValue::Set(project.pi_name.clone()),
            pi_email: ActiveValue::Set(project.pi_email.clone()),
            reference_number: ActiveValue::Set(project.reference_number.clone()),
            target_sample_count: ActiveValue::Set(project.target_sample_count.map(|v| v as i32)),
            sample_count: ActiveValue::Set(project.sample_count as i32),
            created_at: ActiveValue::Set(project.created_at),
            created_by: ActiveValue::Set(project.created_by.clone()),
            updated_at: ActiveValue::Set(project.updated_at),
            due_date: ActiveValue::Set(project.due_date),
        }
    }
}

