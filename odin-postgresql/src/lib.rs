mod account;
pub mod entity;

use crate::entity::account::Entity as Account;
use futures::FutureExt;
use odin_models::account::Account as AccountModel;
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
use sea_orm::{ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::{future::Future, pin::Pin};
use thiserror::Error;

#[derive(Clone)]
pub struct PostgresqlService {
    connection: DatabaseConnection,
}
impl PostgresqlService {
    pub async fn new(database_url: &str) -> Result<Self, DbErr> {
        let connection = Database::connect(database_url).await?;

        Ok(Self { connection })
    }
}
impl AccountRepository for PostgresqlService {
    fn fetch_account<'a>(
        &'a self,
        username: &'a str,
        password: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<AccountModel, AccountRepositoryError>> + 'a>> {
        async move {
            let account = Account::find()
                .filter(entity::account::Column::Username.eq(username))
                .one(&self.connection)
                .await
                .map_err(|_| AccountRepositoryError::InvalidUsernameOrPassword)?
                .ok_or(AccountRepositoryError::InvalidUsernameOrPassword)?;

            if account.password != password {
                return Err(AccountRepositoryError::InvalidUsernameOrPassword);
            }

            Ok(account.into())
        }
        .boxed()
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct FailToOpenDatabase(#[from] DbErr);
