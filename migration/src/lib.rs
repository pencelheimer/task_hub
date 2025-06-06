#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
use loco_oauth2::migration;
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users;
mod m20250527_094045_roles;
mod m20250527_094216_add_role_ref_to_users;
mod m20250601_172250_tasks;
mod m20250602_133444_accesses;
mod m20250605_151704_attachments;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20250527_094045_roles::Migration),
            Box::new(m20250527_094216_add_role_ref_to_users::Migration),
            Box::new(m20250601_172250_tasks::Migration),
            Box::new(m20250602_133444_accesses::Migration),
            Box::new(m20250605_151704_attachments::Migration),
            // inject-above (do not remove this comment)

            // Register OAuth2 sessions migration
            Box::new(migration::m20240101_000000_oauth2_sessions::Migration),
        ]
    }
}