use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.create_table(
            Table::create()
                .table(Alias::new("messages"))
                .if_not_exists()
                .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                .col(ColumnDef::new(Alias::new("session_id")).uuid().not_null())
                .col(ColumnDef::new(Alias::new("role")).string().not_null())
                .col(ColumnDef::new(Alias::new("content")).text().not_null())
                .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-messages-session_id")
                        .from(Alias::new("messages"), Alias::new("session_id"))
                        .to(Alias::new("sessions"), Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(Alias::new("messages")).to_owned()).await
    }
}
