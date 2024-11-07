use futures::future::BoxFuture;
use odin_models::{
    account_charlist::{AccountCharlist, CharacterInfo},
    character::{Character, Class},
    nickname::Nickname,
    uuid::Uuid,
};
use thiserror::Error;

pub trait AccountRepository: Clone + 'static {
    fn fetch_account<'a>(
        &'a self,
        username: &'a str,
    ) -> BoxFuture<'a, Result<Option<AccountCharlist>, AccountRepositoryError>>;

    fn fetch_charlist(
        &self,
        account_id: Uuid,
    ) -> BoxFuture<Result<Vec<(usize, CharacterInfo)>, AccountRepositoryError>>;

    fn fetch_character(
        &self,
        account_id: Uuid,
        slot: usize,
    ) -> BoxFuture<Result<Option<Character>, AccountRepositoryError>>;

    fn update_token(
        &self,
        id: Uuid,
        new_token: Option<String>,
    ) -> BoxFuture<Result<(), AccountRepositoryError>>;

    fn get_token(&self, id: Uuid) -> BoxFuture<Result<Option<String>, AccountRepositoryError>>;

    fn create_character<'a>(
        &'a self,
        account_id: Uuid,
        slot: u32,
        name: &'a Nickname,
        class: Class,
    ) -> BoxFuture<'a, Result<Uuid, AccountRepositoryError>>;

    fn name_exists<'a>(
        &'a self,
        name: &'a Nickname,
    ) -> BoxFuture<'a, Result<bool, AccountRepositoryError>>;

    fn delete_character(
        &self,
        account_id: Uuid,
        slot: usize,
    ) -> BoxFuture<Result<(), AccountRepositoryError>>;

    fn check_password<'a>(
        &'a self,
        account_id: Uuid,
        password: &'a str,
    ) -> BoxFuture<'a, Result<bool, AccountRepositoryError>>;
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
