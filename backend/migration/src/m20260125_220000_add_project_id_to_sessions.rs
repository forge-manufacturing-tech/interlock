use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                .table(Alias::new("sessions"))
                .add_column(
                    ColumnDef::new(Alias::new("project_id"))
                        .uuid()
                        .null(),
                )
                .add_foreign_key(
                    TableForeignKey::new()
                        .name("fk_sessions_project_id")
                        .from_tbl(Alias::new("sessions"))
                        .from_col(Alias::new("project_id"))
                        .to_tbl(Alias::new("projects"))
                        .to_col(Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
            )
            .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("sessions"))
                .drop_column(Alias::new("project_id"))
                .to_owned(),
        )
        .await
    }
}
