use odin_models::{account_charlist::AccountCharlist, uuid::Uuid};
use std::{future::Future, pin::Pin};
use thiserror::Error;

pub trait AccountRepository: Clone + 'static {
    fn fetch_account<'a>(
        &'a self,
        username: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AccountCharlist>, AccountRepositoryError>> + 'a>>;

    fn update_token<'a>(
        &'a self,
        id: Uuid,
        new_token: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AccountRepositoryError>> + 'a>>;

    fn get_token<'a>(
        &'a self,
        id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, AccountRepositoryError>> + 'a>>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AccountRepositoryError {
    #[error("Fail to load account: {0}")]
    FailToLoad(String),

    #[error("{0}")]
    Generic(String),
}
