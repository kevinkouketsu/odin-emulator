use chrono::Local;
use entity::account::Entity as Account;
use entity::account_ban::Entity as AccountBan;
use futures::FutureExt;
use odin_models::account::{Account as AccountModel, Ban, BanType};
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
use sea_orm::{prelude::*, DatabaseConnection, QueryOrder};
use std::{future::Future, pin::Pin};

#[derive(Clone)]
pub struct PostgreSqlAccountRepository {
    connection: DatabaseConnection,
}
impl PostgreSqlAccountRepository {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }
}
impl AccountRepository for PostgreSqlAccountRepository {
    fn fetch_account<'a>(
        &'a self,
        username: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AccountModel>, AccountRepositoryError>> + 'a>>
    {
        async move {
            let Some(account) = Account::find()
                .filter(entity::account::Column::Username.eq(username))
                .one(&self.connection)
                .await
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?
            else {
                return Ok(None);
            };

            let ban = AccountBan::find()
                .filter(entity::account_ban::Column::AccountId.eq(account.id))
                .filter(entity::account_ban::Column::ExpiresAt.gt(Local::now()))
                .order_by_desc(entity::account_ban::Column::ExpiresAt)
                .one(&self.connection)
                .await
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?;

            Ok(Some(AccountModel {
                username: account.username,
                password: account.password,
                ban: ban.map(|ban| Ban {
                    expiration: ban.expires_at,
                    r#type: match ban.r#type {
                        entity::account_ban::BanType::Analysis => BanType::Analysis,
                        entity::account_ban::BanType::Blocked => BanType::Blocked,
                    },
                }),
            }))
        }
        .boxed()
    }
}
