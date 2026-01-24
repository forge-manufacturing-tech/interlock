use loco_rs::schema::*;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Drop existing table
        drop_table(m, "sessions").await?;

        // Create new table with UUID id
        m.create_table(
            Table::create()
                .table(Alias::new("sessions"))
                .if_not_exists()
                .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                .col(ColumnDef::new(Alias::new("title")).string().null())
                .col(ColumnDef::new(Alias::new("content")).text().null())
                .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "sessions").await?;

        // Recreate old table structure
        let mut col = ColumnDef::new(Alias::new("id"));
        col.not_null().auto_increment().primary_key();

        if m.get_database_backend() == DbBackend::Sqlite {
            col.integer();
        } else {
            col.big_integer();
        }

        m.create_table(
            Table::create()
                .table(Alias::new("sessions"))
                .if_not_exists()
                .col(&mut col)
                .col(ColumnDef::new(Alias::new("title")).string().null())
                .col(ColumnDef::new(Alias::new("content")).text().null())
                .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                .to_owned(),
        )
        .await
    }
}
