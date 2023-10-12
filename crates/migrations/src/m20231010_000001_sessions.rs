use sea_orm_migration::prelude::*;

use crate::m20230930_000001_create_users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Sessions::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Sessions::UserId).integer().not_null())
                    .col(ColumnDef::new(Sessions::Token).string().not_null())
                    .col(ColumnDef::new(Sessions::InitialIp).string().not_null())
                    .col(ColumnDef::new(Sessions::ExpiresAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        // Foreign key
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_sessions_user_id")
                    .from(Sessions::Table, Sessions::UserId)
                    .to(Users::Table, Users::Id)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop foreign key
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_sessions_user_id")
                    .table(Sessions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Sessions {
    Table,
    Id,
    Token,
    UserId,
    InitialIp,
    ExpiresAt,
}
