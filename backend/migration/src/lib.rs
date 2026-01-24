#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20260122_045908_sessions;
mod m20260124_000001_sessions_uuid;
mod m20260124_000002_users_add_timestamps;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260122_045908_sessions::Migration),
            Box::new(m20260124_000001_sessions_uuid::Migration),
            Box::new(m20260124_000002_users_add_timestamps::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
