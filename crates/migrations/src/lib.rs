pub use sea_orm_migration::prelude::*;

mod m20230930_000001_create_users;
mod m20231008_000001_schoology_request_tokens;
mod m20231009_000001_schoology_link;
mod m20231010_000001_sessions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230930_000001_create_users::Migration),
            Box::new(m20231008_000001_schoology_request_tokens::Migration),
            Box::new(m20231009_000001_schoology_link::Migration),
            Box::new(m20231010_000001_sessions::Migration),
        ]
    }
}
