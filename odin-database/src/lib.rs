pub mod account_repository;

pub use entity;
pub use sea_orm;

use account_repository::DatabaseAccountRepository;
use migration::MigratorTrait;
use sea_orm::{Database, DatabaseConnection, DbErr};
use thiserror::Error;

#[derive(Clone)]
pub struct DatabaseService {
    connection: DatabaseConnection,
}
impl DatabaseService {
    pub async fn new(database_url: &str) -> Result<Self, DbErr> {
        let connection = Database::connect(database_url).await?;

        Ok(Self { connection })
    }

    pub fn from_database(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    pub fn account_repository(&self) -> DatabaseAccountRepository {
        DatabaseAccountRepository::new(self.connection.clone())
    }

    pub fn get_connection(&self) -> DatabaseConnection {
        self.connection.clone()
    }

    pub async fn fresh(&self) -> Result<(), DbErr> {
        migration::Migrator::fresh(&self.connection).await
    }
}
impl std::ops::Deref for DatabaseService {
    type Target = DatabaseConnection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct FailToOpenDatabase(#[from] DbErr);
