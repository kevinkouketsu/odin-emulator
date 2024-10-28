pub mod account_repository;

use account_repository::PostgreSqlAccountRepository;
use sea_orm::{Database, DatabaseConnection, DbErr};
use thiserror::Error;

#[derive(Clone)]
pub struct PostgreSqlService {
    connection: DatabaseConnection,
}
impl PostgreSqlService {
    pub async fn new(database_url: &str) -> Result<Self, DbErr> {
        let connection = Database::connect(database_url).await?;

        Ok(Self { connection })
    }

    pub fn account_repository(&self) -> PostgreSqlAccountRepository {
        PostgreSqlAccountRepository::new(self.connection.clone())
    }
}
#[derive(Debug, Error)]
#[error(transparent)]
pub struct FailToOpenDatabase(#[from] DbErr);
