use odin_models::account::Account;
use std::{future::Future, pin::Pin};
use thiserror::Error;

pub trait AccountRepository: Clone {
    fn fetch_account<'a>(
        &'a self,
        username: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Account, AccountRepositoryError>> + 'a>>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AccountRepositoryError {
    #[error("The username is not valid")]
    InvalidUsername,

    #[error("Fail to load account: {0}")]
    FailToLoad(String),
}
