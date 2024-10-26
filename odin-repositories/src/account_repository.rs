use std::{future::Future, pin::Pin};

use odin_models::account::Account;
use thiserror::Error;

pub trait AccountRepository: Clone {
    fn fetch_account<'a>(
        &'a self,
        username: &'a str,
        password: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Account, AccountRepositoryError>> + 'a>>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AccountRepositoryError {
    #[error("The username or password is not valid")]
    InvalidUsernameOrPassword,
}
