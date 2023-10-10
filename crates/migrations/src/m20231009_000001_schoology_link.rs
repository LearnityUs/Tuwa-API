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
                    .table(SchoologyLink::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SchoologyLink::UserId)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SchoologyLink::SchoologyId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(SchoologyLink::Email).text())
                    .col(ColumnDef::new(SchoologyLink::PictureUrl).text())
                    .col(ColumnDef::new(SchoologyLink::AccessToken).text())
                    .col(ColumnDef::new(SchoologyLink::TokenSecret).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_schoology_link_user_id")
                            .from(SchoologyLink::Table, SchoologyLink::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_schoology_link_user_id")
                    .table(SchoologyLink::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(SchoologyLink::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum SchoologyLink {
    Table,
    /// FK: User.id
    UserId,
    SchoologyId,
    Email,
    PictureUrl,
    AccessToken,
    TokenSecret,
}
