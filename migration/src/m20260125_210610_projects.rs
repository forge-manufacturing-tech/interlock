use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(Alias::new("projects"))
                .if_not_exists()
                .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                .col(ColumnDef::new(Alias::new("name")).string().not_null())
                .col(ColumnDef::new(Alias::new("description")).text().null())
                .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                .to_owned()
        ).await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "projects").await
    }
}
