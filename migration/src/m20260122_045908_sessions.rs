use loco_rs::schema::*;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
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
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "sessions").await
    }
}
