use chrono::Local;
use entity::account::Entity as Account;
use entity::account_ban::Entity as AccountBan;
use futures::FutureExt;
use odin_models::account::{Account as AccountModel, Ban, BanType};
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
use sea_orm::{prelude::*, Database, DatabaseConnection, DbErr, QueryOrder};
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
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?
                .ok_or(AccountRepositoryError::InvalidUsername)?;

            if account.password != password {
                return Err(AccountRepositoryError::InvalidPassword);
            }

            let ban = AccountBan::find()
                .filter(entity::account_ban::Column::AccountId.eq(account.id))
                .filter(entity::account_ban::Column::ExpiresAt.gt(Local::now()))
                .order_by_desc(entity::account_ban::Column::ExpiresAt)
                .one(&self.connection)
                .await
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?;

            Ok(AccountModel {
                username: account.username,
                password: account.password,
                ban: ban.map(|ban| Ban {
                    expiration: ban.expires_at,
                    r#type: match ban.r#type {
                        entity::account_ban::BanType::Analysis => BanType::Analysis,
                        entity::account_ban::BanType::Blocked => BanType::Blocked,
                    },
                }),
            })
        }
        .boxed()
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct FailToOpenDatabase(#[from] DbErr);
