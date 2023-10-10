use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SchoologyRequestTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SchoologyRequestTokens::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SchoologyRequestTokens::Token)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SchoologyRequestTokens::TokenSecret)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SchoologyRequestTokens::ExpiresAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SchoologyRequestTokens::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum SchoologyRequestTokens {
    Table,
    Id,
    Token,
    TokenSecret,
    ExpiresAt,
}
