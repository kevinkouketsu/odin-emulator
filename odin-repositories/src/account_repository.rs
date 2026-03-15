use odin_models::{
    account_charlist::{AccountCharlist, CharacterInfo},
    character::{Character, Class},
    nickname::Nickname,
    uuid::Uuid,
};
use std::future::Future;
use thiserror::Error;

pub trait AccountRepository: Clone + 'static {
    fn fetch_account(
        &self,
        username: &str,
    ) -> impl Future<Output = Result<Option<AccountCharlist>, AccountRepositoryError>> + Send;

    fn fetch_charlist(
        &self,
        account_id: Uuid,
    ) -> impl Future<Output = Result<Vec<(usize, CharacterInfo)>, AccountRepositoryError>> + Send;

    fn fetch_character(
        &self,
        account_id: Uuid,
        slot: usize,
    ) -> impl Future<Output = Result<Option<Character>, AccountRepositoryError>> + Send;

    fn update_token(
        &self,
        id: Uuid,
        new_token: Option<String>,
    ) -> impl Future<Output = Result<(), AccountRepositoryError>> + Send;

    fn get_token(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<String>, AccountRepositoryError>> + Send;

    fn create_character(
        &self,
        account_id: Uuid,
        slot: u32,
        name: &Nickname,
        class: Class,
    ) -> impl Future<Output = Result<Uuid, AccountRepositoryError>> + Send;

    fn name_exists(
        &self,
        name: &Nickname,
    ) -> impl Future<Output = Result<bool, AccountRepositoryError>> + Send;

    fn delete_character(
        &self,
        account_id: Uuid,
        slot: usize,
    ) -> impl Future<Output = Result<(), AccountRepositoryError>> + Send;

    fn check_password(
        &self,
        account_id: Uuid,
        password: &str,
    ) -> impl Future<Output = Result<bool, AccountRepositoryError>> + Send;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AccountRepositoryError {
    #[error("Fail to query: {0}")]
    FailToLoad(String),

    #[error("{0}")]
    Generic(String),

    #[error("The entity has not been found")]
    EntityNotFound,

    #[error("Character is not valid")]
    CharacterNotValid(String),
}
