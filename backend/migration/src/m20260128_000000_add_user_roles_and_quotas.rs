use loco_rs::schema::*;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // 1. Add columns to users table individually (SQLite compat)
        m.alter_table(
            Table::alter()
                .table(Alias::new("users"))
                .add_column(ColumnDef::new(Alias::new("role")).string().not_null().default("editor"))
                .to_owned(),
        )
        .await?;

        m.alter_table(
            Table::alter()
                .table(Alias::new("users"))
                .add_column(ColumnDef::new(Alias::new("quota_chat_invocations")).big_integer().not_null().default(10000))
                .to_owned(),
        )
        .await?;

        m.alter_table(
            Table::alter()
                .table(Alias::new("users"))
                .add_column(ColumnDef::new(Alias::new("quota_file_uploads")).big_integer().not_null().default(10000))
                .to_owned(),
        )
        .await?;

        m.alter_table(
            Table::alter()
                .table(Alias::new("users"))
                .add_column(ColumnDef::new(Alias::new("usage_chat_invocations")).big_integer().not_null().default(0))
                .to_owned(),
        )
        .await?;

        m.alter_table(
            Table::alter()
                .table(Alias::new("users"))
                .add_column(ColumnDef::new(Alias::new("usage_file_uploads")).big_integer().not_null().default(0))
                .to_owned(),
        )
        .await?;

        // 2. Create groups table
        let mut col = ColumnDef::new(Alias::new("id"));
        col.not_null().auto_increment().primary_key();
        if m.get_database_backend() == DbBackend::Sqlite {
            col.integer();
        } else {
            col.big_integer();
        }

        m.create_table(
            Table::create()
                .table(Alias::new("groups"))
                .if_not_exists()
                .col(&mut col)
                .col(ColumnDef::new(Alias::new("pid")).uuid().not_null())
                .col(ColumnDef::new(Alias::new("name")).string().not_null())
                .col(
                    ColumnDef::new(Alias::new("created_at"))
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .col(
                    ColumnDef::new(Alias::new("updated_at"))
                        .timestamp_with_time_zone()
                        .not_null()
                        .default(Expr::current_timestamp()),
                )
                .to_owned(),
        )
        .await?;

        // 3. Create users_groups table
        m.create_table(
            Table::create()
                .table(Alias::new("users_groups"))
                .if_not_exists()
                .col(
                    ColumnDef::new(Alias::new("user_id"))
                        .big_integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Alias::new("group_id"))
                        .big_integer()
                        .not_null(),
                )
                .primary_key(Index::create().col(Alias::new("user_id")).col(Alias::new("group_id")))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-users_groups-users")
                        .from(Alias::new("users_groups"), Alias::new("user_id"))
                        .to(Alias::new("users"), Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-users_groups-groups")
                        .from(Alias::new("users_groups"), Alias::new("group_id"))
                        .to(Alias::new("groups"), Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(Alias::new("users_groups")).to_owned()).await?;
        m.drop_table(Table::drop().table(Alias::new("groups")).to_owned()).await?;

        // We can't easily drop columns in SQLite without recreating the table.
        // For non-sqlite, we could try dropping columns.
        if m.get_database_backend() != DbBackend::Sqlite {
             m.alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .drop_column(Alias::new("role"))
                    .to_owned(),
            )
            .await?;
             m.alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .drop_column(Alias::new("quota_chat_invocations"))
                    .to_owned(),
            )
            .await?;
             m.alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .drop_column(Alias::new("quota_file_uploads"))
                    .to_owned(),
            )
            .await?;
             m.alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .drop_column(Alias::new("usage_chat_invocations"))
                    .to_owned(),
            )
            .await?;
             m.alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .drop_column(Alias::new("usage_file_uploads"))
                    .to_owned(),
            )
            .await?;
        }

        Ok(())
    }
}
