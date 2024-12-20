use crate::{
    configuration::{Configuration, ServerState},
    session::{SessionError, SessionTrait},
};
use chrono::{Local, NaiveDateTime};
use odin_models::{account::BanType, account_charlist::AccountCharlist, storage::Storage};
use odin_networking::{
    messages::{
        client::login::LoginMessageRaw,
        server::{
            charlist::{CharlistInfo, FirstCharlist},
            message_panel::MessagePanel,
        },
    },
    WritableResourceError,
};
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
use thiserror::Error;

#[derive(Debug)]
pub struct Authentication {
    pub username: String,
    pub password: String,
    pub tid: [u8; 52],
    pub cliver: CliVer,
}
impl Authentication {
    pub async fn handle<A: AccountRepository, S: SessionTrait, C: Configuration>(
        &self,
        session: &S,
        configuration: &C,
        account_repository: A,
    ) -> Result<AccountCharlist, AuthenticationError> {
        match self
            .handle_impl(session, configuration, account_repository)
            .await
        {
            Ok(charlist) => Ok(charlist),
            Err(err) => {
                log::error!("{:?}", err);

                let message = match err {
                    AuthenticationError::InvalidCliVer(_) => {
                        "Baixe as atualizações pelo launcher ou pelo site"
                    }
                    AuthenticationError::AccountRepositoryError(_)
                    | AuthenticationError::InvalidPassword
                    | AuthenticationError::AccountNotFound
                    | AuthenticationError::SendError(_) => "Usuário ou senha inválidos",
                    AuthenticationError::AccountInAnalysis(_) => "Conta está em análise",
                    AuthenticationError::AccountBlocked(_) => "Conta está banida",
                    AuthenticationError::Maintenance => "Servidor está em manutenção",
                };

                session.send::<MessagePanel>(message.into()).unwrap();
                Err(err)
            }
        }
    }

    async fn handle_impl<A: AccountRepository, S: SessionTrait, C: Configuration>(
        &self,
        session: &S,
        configuration: &C,
        account_repository: A,
    ) -> Result<AccountCharlist, AuthenticationError> {
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
            .iter()
            .map(|(slot, character)| {
                (
                    *slot,
                    CharlistInfo {
                        position: character.position,
                        name: character.name.clone(),
                        status: character.status,
                        equips: character.equipments.clone(),
                        guild: character.guild,
                        coin: character.coin,
                        experience: character.experience,
                    },
                )
            })
            .collect();

        let charlist = FirstCharlist {
            token: vec![0; 16],
            character_info: characters,
            storage: Storage::default(),
            account_name: self.username.clone(),
        };

        session.send(charlist)?;
        Ok(account)
    }
}
impl TryFrom<LoginMessageRaw> for Authentication {
    type Error = WritableResourceError;

    fn try_from(value: LoginMessageRaw) -> Result<Self, Self::Error> {
        let username: String = value.username.try_into()?;
        let password: String = value.password.try_into()?;

        Ok(Authentication {
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

    #[error(transparent)]
    SendError(#[from] SessionError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        configuration::ServerState,
        handlers::tests::{MockConfiguration, MockSession, TestAccountRepository},
    };
    use chrono::{Days, Local};
    use odin_models::{
        account::{AccessLevel, Ban, BanType},
        account_charlist::AccountCharlist,
        character::Character,
        uuid::Uuid,
    };

    fn get_login_message() -> Authentication {
        Authentication {
            username: "admin".to_string(),
            password: "admin".to_string(),
            tid: [0; 52],
            cliver: CliVer::new(1u32),
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
                    TestAccountRepository::new().await.account_repository()
                )
                .await,
            Err(AuthenticationError::InvalidCliVer(_))
        ));
    }

    #[tokio::test]
    async fn it_returns_an_error_if_the_account_does_not_exist_or_the_password_is_invalid() {
        let message = get_login_message();

        // No accounts are registered
        let account_repository = TestAccountRepository::new().await;
        assert!(matches!(
            message
                .handle_impl(
                    &MockSession::default(),
                    &MockConfiguration(CliVer::new(1), ServerState::Open),
                    account_repository.account_repository()
                )
                .await
                .unwrap_err(),
            AuthenticationError::AccountNotFound
        ));

        account_repository
            .add_account(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: "admin".to_string(),
                    password: "admin2".to_string(),
                    ban: None,
                    ..Default::default()
                },
                None,
            )
            .await;

        assert!(matches!(
            message
                .handle_impl(
                    &MockSession::default(),
                    &MockConfiguration(CliVer::new(1), ServerState::Open),
                    account_repository.account_repository()
                )
                .await
                .unwrap_err(),
            AuthenticationError::InvalidPassword
        ));
    }

    #[tokio::test]
    async fn checks_if_account_is_banned_or_in_analysis() {
        let account_repository = TestAccountRepository::new().await;
        let expiration = Local::now()
            .checked_add_days(Days::new(3))
            .unwrap()
            .naive_local();

        account_repository
            .add_account(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: "admin".to_string(),
                    password: "admin".to_string(),
                    ban: Some(Ban {
                        expiration,
                        r#type: BanType::Analysis,
                    }),
                    ..Default::default()
                },
                None,
            )
            .await;

        let message = get_login_message();
        match message
            .handle_impl(
                &MockSession::default(),
                &MockConfiguration(CliVer::new(1), ServerState::Open),
                account_repository.account_repository(),
            )
            .await
        {
            Err(AuthenticationError::AccountInAnalysis(_)) => {}
            Err(e) => panic!("Expected AccountInAnalysis, got {e:?}"),
            Ok(_) => panic!("Expected AccountInAnalysis, got Ok"),
        }

        let account_repository = TestAccountRepository::new().await;
        account_repository
            .add_account(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: "admin".to_string(),
                    password: "admin".to_string(),
                    ban: Some(Ban {
                        expiration,
                        r#type: BanType::Blocked,
                    }),
                    ..Default::default()
                },
                None,
            )
            .await;

        match message
            .handle_impl(
                &MockSession::default(),
                &MockConfiguration(CliVer::new(1), ServerState::Open),
                account_repository.account_repository(),
            )
            .await
        {
            Err(AuthenticationError::AccountBlocked(_)) => {}
            Err(e) => panic!("Expected AccountBlocked, got {e:?}"),
            Ok(_) => panic!("Expected AccountBlocked, got Ok"),
        }
    }

    #[tokio::test]
    async fn successful_login_returns_the_charlist() {
        let account_repository = TestAccountRepository::new().await;
        let character = Character {
            identifier: Uuid::new_v4(),
            last_pos: (2100, 2100).into(),
            name: "charlist".to_string(),
            ..Default::default()
        };

        let account_id = Uuid::new_v4();
        account_repository
            .add_account(
                AccountCharlist {
                    identifier: account_id,
                    username: "admin".to_string(),
                    password: "admin".to_string(),
                    ..Default::default()
                },
                None,
            )
            .await;

        account_repository
            .add_character(account_id, character)
            .await;

        let session = MockSession::default();
        let charlist = get_login_message()
            .handle_impl(
                &session,
                &MockConfiguration(CliVer::new(1), ServerState::Open),
                account_repository.account_repository(),
            )
            .await
            .expect("Must login successfully");

        let (index, character) = &charlist.charlist[0];
        assert_eq!(*index, 0);
        assert_eq!(character.name, "charlist");
        assert_eq!(character.position, (2100, 2100).into());
    }

    #[tokio::test]
    async fn it_cant_login_when_server_is_closed_for_maintenance_and_the_access_is_normal() {
        let account_repository = TestAccountRepository::new().await;
        account_repository
            .add_account(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: "admin".to_string(),
                    password: "admin".to_string(),
                    access: None,
                    ..Default::default()
                },
                None,
            )
            .await;

        assert_eq!(
            get_login_message()
                .handle_impl(
                    &MockSession::default(),
                    &MockConfiguration(CliVer::new(1), ServerState::Maintenance),
                    account_repository.account_repository()
                )
                .await
                .unwrap_err(),
            AuthenticationError::Maintenance
        )
    }

    #[tokio::test]
    async fn it_can_login_when_server_is_closed_for_maintenance_and_the_access_is_gamemaster_or_admin(
    ) {
        let account_repository = TestAccountRepository::new().await;
        account_repository
            .add_account(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: "admin".to_string(),
                    password: "admin".to_string(),
                    access: Some(AccessLevel::Administrator),
                    ..Default::default()
                },
                None,
            )
            .await;

        account_repository
            .add_account(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: "admin2".to_string(),
                    password: "admin".to_string(),
                    access: Some(AccessLevel::GameMaster(1)),
                    ..Default::default()
                },
                None,
            )
            .await;

        let result = get_login_message()
            .handle_impl(
                &MockSession::default(),
                &MockConfiguration(CliVer::new(1), ServerState::Maintenance),
                account_repository.account_repository(),
            )
            .await;

        assert!(result.is_ok());

        let result = Authentication {
            username: "admin2".to_string(),
            password: "admin".to_string(),
            tid: [0; 52],
            cliver: CliVer::new(1u32),
        }
        .handle_impl(
            &MockSession::default(),
            &MockConfiguration(CliVer::new(1), ServerState::Maintenance),
            account_repository.account_repository(),
        )
        .await;

        assert!(result.is_ok())
    }
}
