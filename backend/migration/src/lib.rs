#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20260122_045908_sessions;
mod m20260124_000001_sessions_uuid;
mod m20260124_000002_users_add_timestamps;
mod m20260125_210610_projects;
mod m20260125_220000_add_project_id_to_sessions;
mod m20260125_221000_users_projects;
mod m20260126_035153_add_blobs;
mod m20260126_043000_add_messages;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20260122_045908_sessions::Migration),
            Box::new(m20260124_000001_sessions_uuid::Migration),
            Box::new(m20260124_000002_users_add_timestamps::Migration),
            Box::new(m20260125_210610_projects::Migration),
            Box::new(m20260125_220000_add_project_id_to_sessions::Migration),
            Box::new(m20260125_221000_users_projects::Migration),
            Box::new(m20260126_035153_add_blobs::Migration),
            Box::new(m20260126_043000_add_messages::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}