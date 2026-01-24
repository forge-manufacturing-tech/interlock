use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(Alias::new("users"))
                .if_not_exists()
                .col(
                    ColumnDef::new(Alias::new("id"))
                        .big_integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Alias::new("pid")).uuid().not_null())
                .col(
                    ColumnDef::new(Alias::new("email"))
                        .string()
                        .not_null()
                        .unique_key(),
                )
                .col(ColumnDef::new(Alias::new("password")).string().not_null())
                .col(
                    ColumnDef::new(Alias::new("api_key"))
                        .string()
                        .not_null()
                        .unique_key(),
                )
                .col(ColumnDef::new(Alias::new("name")).string().not_null())
                .col(ColumnDef::new(Alias::new("reset_token")).string().null())
                .col(
                    ColumnDef::new(Alias::new("reset_sent_at"))
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(Alias::new("email_verification_token"))
                        .string()
                        .null(),
                )
                .col(
                    ColumnDef::new(Alias::new("email_verification_sent_at"))
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(Alias::new("email_verified_at"))
                        .timestamp_with_time_zone()
                        .null(),
                )
                .col(
                    ColumnDef::new(Alias::new("magic_link_token"))
                        .string()
                        .null(),
                )
                .col(
                    ColumnDef::new(Alias::new("magic_link_expiration"))
                        .timestamp_with_time_zone()
                        .null(),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "users").await?;
        Ok(())
    }
}
