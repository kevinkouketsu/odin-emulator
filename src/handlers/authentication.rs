use crate::{
    configuration::{Configuration, ServerState},
    session::Session,
};
use chrono::{Local, NaiveDateTime};
use odin_models::{account::BanType, storage::Storage};
use odin_networking::{
    messages::{
        client::login::LoginMessageRaw,
        server::{
            charlist::{Charlist, CharlistInfo},
            message_panel::MessagePanel,
        },
    },
    WritableResourceError,
};
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
    pub async fn handle<A: AccountRepository, S: Session, C: Configuration>(
        &self,
        session: &S,
        configuration: &C,
        account_repository: A,
    ) {
        match self
            .handle_impl(session, configuration, account_repository)
            .await
        {
            Ok(charlist) => session.send(charlist).unwrap(),
            Err(err) => {
                log::error!("{:?}", err);

                let message = match err {
                    AuthenticationError::InvalidCliVer(_) => {
                        "Baixe as atualizações pelo launcher ou pelo site"
                    }
                    AuthenticationError::AccountRepositoryError(_)
                    | AuthenticationError::InvalidPassword
                    | AuthenticationError::AccountNotFound => "Usuário ou senha inválidos",
                    AuthenticationError::AccountInAnalysis(_) => "Conta está em análise",
                    AuthenticationError::AccountBlocked(_) => "Conta está banida",
                    AuthenticationError::Maintenance => "Servidor está em manutenção",
                };

                session.send::<MessagePanel>(message.into()).unwrap();
            }
        }
    }

    async fn handle_impl<A: AccountRepository, S: Session, C: Configuration>(
        &self,
        _session: &S,
        configuration: &C,
        account_repository: A,
    ) -> Result<Charlist, AuthenticationError> {
        if self.cliver != configuration.get_current_cliver() {
            return Err(AuthenticationError::InvalidCliVer(self.cliver.into()));
        }

        let account = account_repository
            .fetch_account(&self.username)
            .await?
            .ok_or(AuthenticationError::AccountNotFound)?;

        if account.password != self.password {
            return Err(AuthenticationError::InvalidPassword);
        }

        if configuration.get_server_state() == ServerState::Maintenance && account.access.is_none()
        {
            return Err(AuthenticationError::Maintenance);
        }

        if let Some(ban) = &account.ban {
            if ban.expiration > Local::now().naive_local() {
                return Err(match ban.r#type {
                    BanType::Analysis => AuthenticationError::AccountInAnalysis(ban.expiration),
                    BanType::Blocked => AuthenticationError::AccountBlocked(ban.expiration),
                });
            }
        }

        let characters = account
            .charlist
            .into_iter()
            .map(|(slot, character)| {
                (
                    slot,
                    CharlistInfo {
                        position: character.position,
                        name: character.name,
                        status: character.status,
                        equips: character.equipments,
                        guild: character.guild,
                        coin: character.coin,
                        experience: character.experience,
                    },
                )
            })
            .collect();

        let charlist = Charlist {
            token: vec![0; 16],
            character_info: characters,
            storage: Storage::default(),
            account_name: self.username.clone(),
        };

        Ok(charlist)
    }
}
impl TryFrom<LoginMessageRaw> for LoginMessage {
    type Error = WritableResourceError;

    fn try_from(value: LoginMessageRaw) -> Result<Self, Self::Error> {
        let username: String = value.username.try_into()?;
        let password: String = value.password.try_into()?;

        Ok(LoginMessage {
            username,
            password,
            tid: value.tid,
            cliver: CliVer::from_encrypted(value.cliver),
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

    #[error("The password is invalid")]
    InvalidPassword,

    #[error("The account was not found")]
    AccountNotFound,

    #[error(transparent)]
    AccountRepositoryError(#[from] AccountRepositoryError),

    #[error("Account is in analysis until {0}")]
    AccountInAnalysis(NaiveDateTime),

    #[error("Account is blocked until {0}")]
    AccountBlocked(NaiveDateTime),

    #[error("Server is under maintenance")]
    Maintenance,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{configuration::ServerState, session::SendError};
    use chrono::{Days, Local};
    use deku::prelude::*;
    use futures::FutureExt;
    use odin_models::{
        account::{AccessLevel, Ban, BanType},
        account_charlist::{AccountCharlist, CharacterInfo},
    };
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
        accounts: Arc<RwLock<HashMap<String, AccountCharlist>>>,
    }
    impl MockAccountRepository {
        pub fn add_account(&mut self, account: AccountCharlist) {
            self.accounts
                .write()
                .unwrap()
                .insert(account.username.clone(), account);
        }

        pub fn edit_account<F: FnOnce(&mut AccountCharlist)>(
            &mut self,
            username: &str,
            callback: F,
        ) {
            let mut accounts = self.accounts.write().unwrap();
            let account = accounts.get_mut(username);

            callback(account.unwrap());
        }
    }
    impl From<(&str, &str)> for MockAccountRepository {
        fn from(value: (&str, &str)) -> Self {
            let mut account_repository = MockAccountRepository::default();
            account_repository.add_account(AccountCharlist {
                username: value.0.to_string(),
                password: value.1.to_string(),
                ban: None,
                ..Default::default()
            });

            account_repository
        }
    }
    impl AccountRepository for MockAccountRepository {
        fn fetch_account<'a>(
            &'a self,
            username: &'a str,
        ) -> Pin<
            Box<dyn Future<Output = Result<Option<AccountCharlist>, AccountRepositoryError>> + 'a>,
        > {
            async move {
                let accounts = self.accounts.read().unwrap();
                let Some(account) = accounts.get(username) else {
                    return Ok(None);
                };

                Ok(Some(account.clone()))
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

    struct MockConfiguration(CliVer, ServerState);
    impl Configuration for MockConfiguration {
        fn get_current_cliver(&self) -> CliVer {
            self.0
        }

        fn get_server_state(&self) -> ServerState {
            self.1
        }
    }

    #[tokio::test]
    async fn it_returns_an_error_if_the_cliver_mismatch() {
        let message = get_login_message();
        assert!(matches!(
            message
                .handle_impl(
                    &MockSession::default(),
                    &MockConfiguration(CliVer::new(2), ServerState::Open),
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
                .handle_impl(
                    &MockSession::default(),
                    &MockConfiguration(CliVer::new(1), ServerState::Open),
                    account_repository.clone()
                )
                .await
                .unwrap_err(),
            AuthenticationError::AccountNotFound
        ));

        account_repository.add_account(AccountCharlist {
            username: "admin".to_string(),
            password: "admin2".to_string(),
            ban: None,
            ..Default::default()
        });

        assert!(matches!(
            message
                .handle_impl(
                    &MockSession::default(),
                    &MockConfiguration(CliVer::new(1), ServerState::Open),
                    account_repository
                )
                .await,
            Err(AuthenticationError::InvalidPassword)
        ));
    }

    #[tokio::test]
    async fn checks_if_account_is_banned_or_in_analysis() {
        let mut account_repository = MockAccountRepository::default();
        let expiration = Local::now()
            .checked_add_days(Days::new(3))
            .unwrap()
            .naive_local();

        account_repository.add_account(AccountCharlist {
            username: "admin".to_string(),
            password: "admin".to_string(),
            ban: Some(Ban {
                expiration,
                r#type: BanType::Analysis,
            }),
            ..Default::default()
        });

        account_repository.edit_account("admin", |account| {
            account.ban.as_mut().unwrap().r#type = BanType::Blocked
        });

        let message = get_login_message();
        assert!(matches!(
            message
                .handle_impl(
                    &MockSession::default(),
                    &MockConfiguration(CliVer::new(1), ServerState::Open),
                    account_repository
                )
                .await,
            Err(AuthenticationError::AccountBlocked(_))
        ));
    }

    #[tokio::test]
    async fn successful_login_returns_the_charlist() {
        let mut account_repository = MockAccountRepository::default();
        let charlist_info = CharacterInfo {
            position: (2100, 2100).into(),
            name: "charlist".to_string(),
            ..Default::default()
        };

        account_repository.add_account(AccountCharlist {
            username: "admin".to_string(),
            password: "admin".to_string(),
            charlist: vec![(0, charlist_info)],
            ..Default::default()
        });

        let session = MockSession::default();
        let charlist = get_login_message()
            .handle_impl(
                &session,
                &MockConfiguration(CliVer::new(1), ServerState::Open),
                account_repository,
            )
            .await
            .expect("Must login successfully");

        let (index, character) = &charlist.character_info[0];
        assert_eq!(*index, 0);
        assert_eq!(character.name, "charlist");
        assert_eq!(character.position, (2100, 2100).into());
    }

    #[tokio::test]
    async fn it_cant_login_when_server_is_closed_for_maintenance_and_the_access_is_normal() {
        let mut account_repository = MockAccountRepository::default();
        account_repository.add_account(AccountCharlist {
            username: "admin".to_string(),
            password: "admin".to_string(),
            access: None,
            ..Default::default()
        });

        assert_eq!(
            get_login_message()
                .handle_impl(
                    &MockSession::default(),
                    &MockConfiguration(CliVer::new(1), ServerState::Maintenance),
                    account_repository
                )
                .await
                .unwrap_err(),
            AuthenticationError::Maintenance
        )
    }

    #[tokio::test]
    async fn it_cant_login_when_server_is_closed_for_maintenance_and_the_access_is_gamemaster_or_admin(
    ) {
        let mut account_repository = MockAccountRepository::default();
        account_repository.add_account(AccountCharlist {
            username: "admin".to_string(),
            password: "admin".to_string(),
            access: Some(AccessLevel::Administrator),
            ..Default::default()
        });
        account_repository.add_account(AccountCharlist {
            username: "admin2".to_string(),
            password: "admin".to_string(),
            access: Some(AccessLevel::GameMaster(1)),
            ..Default::default()
        });

        let result = get_login_message()
            .handle_impl(
                &MockSession::default(),
                &MockConfiguration(CliVer::new(1), ServerState::Maintenance),
                account_repository.clone(),
            )
            .await;

        assert!(result.is_ok());

        let result = LoginMessage {
            username: "admin2".to_string(),
            password: "admin".to_string(),
            tid: [0; 52],
            cliver: CliVer::new(1u32),
        }
        .handle_impl(
            &MockSession::default(),
            &MockConfiguration(CliVer::new(1), ServerState::Maintenance),
            account_repository,
        )
        .await;

        assert!(result.is_ok())
    }
}
