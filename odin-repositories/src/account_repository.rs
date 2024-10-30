use odin_models::account_charlist::AccountCharlist;
use std::{future::Future, pin::Pin};
use thiserror::Error;

pub trait AccountRepository: Clone {
    fn fetch_account<'a>(
        &'a self,
        username: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AccountCharlist>, AccountRepositoryError>> + 'a>>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AccountRepositoryError {
    #[error("Fail to load account: {0}")]
    FailToLoad(String),

    #[error("{0}")]
    Generic(String),
}
