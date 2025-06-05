#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20250527_094045_roles;
mod m20250527_094216_add_role_ref_to_users;
mod m20250601_172250_tasks;
mod m20250602_133444_accesses;
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
            // inject-above (do not remove this comment)
        ]
    }
}