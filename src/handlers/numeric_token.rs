use crate::session::{SessionError, SessionTrait};
use odin_models::uuid::Uuid;
use odin_networking::{
    messages::{
        client::numeric_token::NumericTokenRaw,
        server::numeric_token::{CorrectNumericToken, IncorrectNumericToken},
    },
    WritableResourceError,
};
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct NumericToken {
    token: String,
    changing: bool,
}
impl NumericToken {
    pub async fn handle<A: AccountRepository, S: SessionTrait>(
        &self,
        session: &S,
        account_id: Uuid,
        valid_token: bool,
        account_repository: A,
    ) -> Result<(), NumericTokenError> {
        match self
            .handle_impl(account_id, valid_token, account_repository)
            .await
        {
            Ok(_) => {
                session.send(CorrectNumericToken {
                    token: self.token.clone(),
                    changing: self.changing,
                })?;

                Ok(())
            }
            Err(err) => {
                session.send(IncorrectNumericToken)?;

                Err(err)
            }
        }
    }

    pub async fn handle_impl<A: AccountRepository>(
        &self,
        account_id: Uuid,
        valid_token: bool,
        account_repository: A,
    ) -> Result<(), NumericTokenError> {
        let current_token = account_repository.get_token(account_id).await?;
        match current_token {
            Some(current_token) => match (self.changing, valid_token) {
                (true, false) => return Err(NumericTokenError::IncorrectState),
                (false, false) => {
                    if *current_token != self.token {
                        return Err(NumericTokenError::IncorrectToken(self.token.clone()));
                    }
                }
                (true, true) => {
                    account_repository
                        .update_token(account_id, Some(self.token.clone()))
                        .await?;
                }
                _ => panic!("Invalid state"),
            },
            None => {
                account_repository
                    .update_token(account_id, Some(self.token.clone()))
                    .await?;
            }
        };

        Ok(())
    }
}
impl TryFrom<NumericTokenRaw> for NumericToken {
    type Error = WritableResourceError;

    fn try_from(value: NumericTokenRaw) -> Result<Self, Self::Error> {
        let token = value.token.try_into()?;

        Ok(Self {
            token,
            changing: value.state != 0,
        })
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NumericTokenError {
    #[error("Token provided is incorrect")]
    IncorrectToken(String),

    #[error("You need to input the token before changing the password")]
    IncorrectState,

    #[error(transparent)]
    AccountRepositoryError(#[from] AccountRepositoryError),

    #[error(transparent)]
    SessionError(#[from] SessionError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::tests::{MockAccountCharlist, MockAccountRepository, MockSession};
    use odin_models::account_charlist::AccountCharlist;

    fn account_with_token(token: Option<&str>) -> MockAccountCharlist {
        MockAccountCharlist {
            account_charlist: AccountCharlist {
                identifier: Uuid::new_v4(),
                username: "admin".to_string(),
                password: "admin".to_string(),
                ..Default::default()
            },
            token: token.map(Into::into),
        }
    }

    #[tokio::test]
    async fn it_sets_the_token_on_first_login() {
        let mut account_repository = MockAccountRepository::default();
        account_repository.add_account(account_with_token(None));

        let account = account_repository.get_account("admin").unwrap();
        assert!(NumericToken {
            token: "1208".to_string(),
            changing: false,
        }
        .handle_impl(
            account.account_charlist.identifier,
            false,
            account_repository.clone()
        )
        .await
        .is_ok());

        assert_eq!(
            account_repository.get_account("admin").unwrap().token,
            Some("1208".to_string())
        );
    }

    mod given_an_already_set_token {
        use super::*;

        #[tokio::test]
        async fn when_the_user_tries_to_login_with_incorrect_token_then_it_returns_an_error() {
            let mut account_repository = MockAccountRepository::default();
            account_repository.add_account(account_with_token(Some("1208")));

            let account = account_repository.get_account("admin").unwrap();
            assert!(matches!(
                NumericToken {
                    token: "1111".to_string(),
                    changing: false,
                }
                .handle_impl(
                    account.account_charlist.identifier,
                    false,
                    account_repository.clone()
                )
                .await,
                Err(NumericTokenError::IncorrectToken(_))
            ));
        }

        #[tokio::test]
        async fn then_it_must_not_update_the_token() {
            let mut account_repository = MockAccountRepository::default();
            account_repository.add_account(account_with_token(Some("1208")));

            let account = account_repository.get_account("admin").unwrap();
            assert!(NumericToken {
                token: "1208".to_string(),
                changing: false,
            }
            .handle_impl(
                account.account_charlist.identifier,
                false,
                account_repository.clone()
            )
            .await
            .is_ok());
        }

        mod when_the_user_tries_to_change_the_token {
            use super::*;

            #[tokio::test]
            async fn without_entering_the_correct_token_beforehand() {
                let mut account_repository = MockAccountRepository::default();
                account_repository.add_account(account_with_token(Some("1208")));

                let account = account_repository.get_account("admin").unwrap();
                assert!(matches!(
                    NumericToken {
                        token: "1111".to_string(),
                        changing: true,
                    }
                    .handle(
                        &MockSession::default(),
                        account.account_charlist.identifier,
                        false,
                        account_repository.clone()
                    )
                    .await,
                    Err(NumericTokenError::IncorrectState)
                ));
            }

            #[tokio::test]
            async fn with_previously_correct_token() {
                let mut account_repository = MockAccountRepository::default();
                account_repository.add_account(account_with_token(Some("1208")));

                let account = account_repository.get_account("admin").unwrap();
                assert!(NumericToken {
                    token: "1111".to_string(),
                    changing: true,
                }
                .handle_impl(
                    account.account_charlist.identifier,
                    true,
                    account_repository.clone()
                )
                .await
                .is_ok());

                assert_eq!(
                    account_repository.get_account("admin").unwrap().token,
                    Some("1111".to_string())
                )
            }
        }
    }
}
