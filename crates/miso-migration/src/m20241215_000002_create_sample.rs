//! Create the sample table.

use sea_orm_migration::prelude::*;

use super::m20241215_000001_create_project::Project;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Sample::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sample::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Sample::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Sample::Barcode)
                            .string_len(50)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Sample::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Sample::Description).text())
                    .col(
                        ColumnDef::new(Sample::SampleMode)
                            .string_len(20)
                            .not_null()
                            .default("plain"),
                    )
                    .col(ColumnDef::new(Sample::SampleClass).string_len(50))
                    .col(ColumnDef::new(Sample::ParentId).integer())
                    .col(ColumnDef::new(Sample::ScientificName).string_len(255))
                    .col(ColumnDef::new(Sample::Volume).decimal_len(10, 2))
                    .col(ColumnDef::new(Sample::Concentration).decimal_len(10, 2))
                    .col(
                        ColumnDef::new(Sample::QcStatus)
                            .string_len(20)
                            .not_null()
                            .default("not_ready"),
                    )
                    .col(ColumnDef::new(Sample::ReceivedAt).timestamp())
                    .col(
                        ColumnDef::new(Sample::CreatedBy)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Sample::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Sample::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Sample::Archived)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Sample::ExternalName).string_len(255))
                    .col(ColumnDef::new(Sample::TissueOrigin).string_len(100))
                    .col(ColumnDef::new(Sample::TissueType).string_len(100))
                    .col(ColumnDef::new(Sample::AnalyteType).string_len(50))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sample_project")
                            .from(Sample::Table, Sample::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sample_parent")
                            .from(Sample::Table, Sample::ParentId)
                            .to(Sample::Table, Sample::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_sample_project")
                    .table(Sample::Table)
                    .col(Sample::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sample_parent")
                    .table(Sample::Table)
                    .col(Sample::ParentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sample_qc_status")
                    .table(Sample::Table)
                    .col(Sample::QcStatus)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Sample::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Sample {
    Table,
    Id,
    Name,
    Barcode,
    ProjectId,
    Description,
    SampleMode,
    SampleClass,
    ParentId,
    ScientificName,
    Volume,
    Concentration,
    QcStatus,
    ReceivedAt,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
    Archived,
    ExternalName,
    TissueOrigin,
    TissueType,
    AnalyteType,
}

