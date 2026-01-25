use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        
        db.execute(Statement::from_string(
            m.get_database_backend(),
            r#"ALTER TABLE "users" ADD COLUMN IF NOT EXISTS "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP;"#.to_owned(),
        )).await?;

        db.execute(Statement::from_string(
            m.get_database_backend(),
            r#"ALTER TABLE "users" ADD COLUMN IF NOT EXISTS "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP;"#.to_owned(),
        )).await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(Alias::new("users"))
                .drop_column(Alias::new("created_at"))
                .to_owned(),
        )
        .await?;

        m.alter_table(
            Table::alter()
                .table(Alias::new("users"))
                .drop_column(Alias::new("updated_at"))
                .to_owned(),
        )
        .await
    }
}
