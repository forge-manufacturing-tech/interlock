use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Alias::new("users_projects"))
                .if_not_exists()
                .col(ColumnDef::new(Alias::new("user_id")).big_integer().not_null())
                .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                .primary_key(
                    Index::create()
                        .col(Alias::new("user_id"))
                        .col(Alias::new("project_id")),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_users_projects_user_id")
                        .from(Alias::new("users_projects"), Alias::new("user_id"))
                        .to(Alias::new("users"), Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_users_projects_project_id")
                        .from(Alias::new("users_projects"), Alias::new("project_id"))
                        .to(Alias::new("projects"), Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.drop_table(Table::drop().table(Alias::new("users_projects")).to_owned())
            .await
    }
}
