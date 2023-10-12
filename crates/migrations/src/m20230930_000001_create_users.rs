//! This migration creates the table `users`.
//! The `users` table is used to mark the existence of a user.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::IsAdmin)
                            .boolean()
                            .default(Expr::value(false))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::IsRoot)
                            .boolean()
                            .default(Expr::value(false))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .date_time()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Users {
    Table,
    Id,
    IsAdmin,
    IsRoot,
    CreatedAt,
}
