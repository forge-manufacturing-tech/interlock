use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

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
                .to_owned(),
            )
            .await?;

        if manager.get_database_backend() != DbBackend::Sqlite {
            manager
                .create_foreign_key(
                    ForeignKey::create()
                        .name("fk_sessions_project_id")
                        .from(Alias::new("sessions"), Alias::new("project_id"))
                        .to(Alias::new("projects"), Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
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
