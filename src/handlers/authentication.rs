use crate::session::Session;
use deku::prelude::*;
use odin_networking::{messages::string::FixedSizeString, WritableResourceError};
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
use thiserror::Error;

#[derive(Debug)]
pub struct LoginMessage {
    pub username: String,
    pub password: String,
    pub tid: [u8; 52],
    pub cliver: CliVer,
}
impl LoginMessage {
    pub async fn handle<A: AccountRepository, S: Session>(
        &self,
        _session: &S,
        cliver: CliVer,
        account_repository: A,
    ) -> Result<(), AuthenticationError> {
        if self.cliver != cliver {
            return Err(AuthenticationError::InvalidCliVer(self.cliver.into()));
        }

        let _account = account_repository
            .fetch_account(&self.username, &self.password)
            .await?;

        Ok(())
    }
}

#[derive(Debug, DekuWrite, DekuRead)]
pub struct LoginMessageRaw {
    pub password: FixedSizeString<16>,
    pub username: FixedSizeString<16>,
    pub tid: [u8; 52],
    pub cliver: u32,
    pub force: u32,
    pub mac: [u8; 16],
}
impl TryInto<LoginMessage> for LoginMessageRaw {
    type Error = WritableResourceError;

    fn try_into(self) -> Result<LoginMessage, Self::Error> {
        let username: String = self.username.try_into()?;
        let password: String = self.password.try_into()?;

        Ok(LoginMessage {
            username,
            password,
            tid: self.tid,
            cliver: CliVer::from_encrypted(self.cliver),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CliVer(u32);
impl CliVer {
    pub fn new(cliver: u32) -> Self {
        CliVer(cliver)
    }

    pub fn from_encrypted(cliver: u32) -> Self {
        CliVer(cliver.wrapping_shr((cliver & 28).wrapping_shr(2).wrapping_add(5)))
    }

    pub fn get_version(&self) -> u32 {
        self.0
    }
}
impl From<CliVer> for u32 {
    fn from(value: CliVer) -> Self {
        value.0
    }
}
impl PartialEq<u32> for CliVer {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

#[derive(Debug, Eq, PartialEq, Error)]
pub enum AuthenticationError {
    #[error("The client version {0} is not valid")]
    InvalidCliVer(u32),

    #[error(transparent)]
    AccountRepositoryError(#[from] AccountRepositoryError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::SendError;
    use futures::FutureExt;
    use odin_models::account::Account;
    use odin_networking::WritableResource;
    use std::{
        collections::HashMap,
        future::Future,
        pin::Pin,
        sync::{Arc, RwLock},
    };

    fn get_login_message() -> LoginMessage {
        LoginMessage {
            username: "admin".to_string(),
            password: "admin".to_string(),
            tid: [0; 52],
            cliver: CliVer::new(1u32),
        }
    }

    #[derive(Default, Clone)]
    pub struct MockAccountRepository {
        accounts: Arc<RwLock<HashMap<String, Account>>>,
    }
    impl MockAccountRepository {
        pub fn add_account(&mut self, account: Account) {
            self.accounts
                .write()
                .unwrap()
                .insert(account.username.clone(), account);
        }
    }
    impl From<(&str, &str)> for MockAccountRepository {
        fn from(value: (&str, &str)) -> Self {
            let mut account_repository = MockAccountRepository::default();
            account_repository.add_account(Account {
                username: value.0.to_string(),
                password: value.1.to_string(),
            });

            account_repository
        }
    }
    impl AccountRepository for MockAccountRepository {
        fn fetch_account<'a>(
            &'a self,
            username: &'a str,
            password: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<Account, AccountRepositoryError>> + 'a>> {
            async move {
                let accounts = self.accounts.read().unwrap();
                let Some(account) = accounts.get(username) else {
                    return Err(AccountRepositoryError::InvalidUsernameOrPassword);
                };

                if account.password != password {
                    return Err(AccountRepositoryError::InvalidUsernameOrPassword);
                }

                Ok(account.clone())
            }
            .boxed()
        }
    }

    #[derive(Default)]
    pub struct MockSession {
        messages: RwLock<Vec<Vec<u8>>>,
    }
    impl Session for MockSession {
        fn send<R: WritableResource>(&self, message: R) -> Result<(), SendError> {
            let message: Vec<u8> = message.write().unwrap().to_bytes().unwrap();
            self.messages.try_write().unwrap().push(message);

            Ok(())
        }
    }

    #[tokio::test]
    async fn it_returns_an_error_if_the_cliver_mismatch() {
        let message = get_login_message();
        assert!(matches!(
            message
                .handle(
                    &MockSession::default(),
                    CliVer::new(2),
                    MockAccountRepository::default()
                )
                .await,
            Err(AuthenticationError::InvalidCliVer(_))
        ));
    }

    #[tokio::test]
    async fn it_returns_an_error_if_the_account_does_not_exist_or_the_password_is_invalid() {
        let message = get_login_message();

        // No accounts are registered
        let mut account_repository = MockAccountRepository::default();
        assert!(matches!(
            message
                .handle(
                    &MockSession::default(),
                    CliVer::new(1),
                    account_repository.clone()
                )
                .await
                .unwrap_err(),
            AuthenticationError::AccountRepositoryError(
                AccountRepositoryError::InvalidUsernameOrPassword
            )
        ));

        account_repository.add_account(Account {
            username: "admin".to_string(),
            password: "admin2".to_string(),
        });

        assert!(matches!(
            message
                .handle(&MockSession::default(), CliVer::new(1), account_repository)
                .await,
            Err(AuthenticationError::AccountRepositoryError(
                AccountRepositoryError::InvalidUsernameOrPassword
            ))
        ));
    }

    #[tokio::test]
    async fn it_sends_the_charlist_to_the_user() {
        let message = get_login_message();
        let account_repository = MockAccountRepository::from(("admin", "admin"));

        let session = MockSession::default();
        MockSession::default();
        assert!(message
            .handle(&session, CliVer::new(1), account_repository)
            .await
            .is_ok());

        let message = session
            .messages
            .try_read()
            .unwrap()
            .first()
            .expect("Must have a message");
    }
}
