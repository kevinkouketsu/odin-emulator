pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241026_013531_create_accounts_table::Migration),
            Box::new(m20241029_210508_characters::Migration),
        ]
    }
}
mod m20241026_013531_create_accounts_table;
mod m20241029_210508_characters;
