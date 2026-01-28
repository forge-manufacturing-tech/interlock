use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("sessions"))
                .add_column(
                    ColumnDef::new(Alias::new("status"))
                        .string()
                        .not_null()
                        .default("idle"),
                )
                .to_owned(),
        )
        .await?;

        m.alter_table(
            Table::alter()
                .table(Alias::new("sessions"))
                .add_column(
                    ColumnDef::new(Alias::new("pending_tasks"))
                        .json_binary()
                        .not_null()
                        .default("[]"),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("sessions"))
                .drop_column(Alias::new("status"))
                .to_owned(),
        )
        .await?;

        m.alter_table(
            Table::alter()
                .table(Alias::new("sessions"))
                .drop_column(Alias::new("pending_tasks"))
                .to_owned(),
        )
        .await
    }
}
