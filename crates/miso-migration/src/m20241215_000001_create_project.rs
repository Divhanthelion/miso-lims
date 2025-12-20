//! Create the project table.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Project::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Project::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Project::Code)
                            .string_len(50)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Project::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Project::Description).text())
                    .col(
                        ColumnDef::new(Project::Status)
                            .string_len(20)
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(Project::PiName).string_len(255))
                    .col(ColumnDef::new(Project::PiEmail).string_len(255))
                    .col(ColumnDef::new(Project::ReferenceNumber).string_len(100))
                    .col(ColumnDef::new(Project::TargetSampleCount).integer())
                    .col(
                        ColumnDef::new(Project::SampleCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Project::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Project::CreatedBy)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Project::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Project::DueDate).timestamp())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Project::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Project {
    Table,
    Id,
    Code,
    Name,
    Description,
    Status,
    PiName,
    PiEmail,
    ReferenceNumber,
    TargetSampleCount,
    SampleCount,
    CreatedAt,
    CreatedBy,
    UpdatedAt,
    DueDate,
}

