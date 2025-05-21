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
    use crate::handlers::tests::{MockSession, TestAccountRepository};
    use odin_models::account_charlist::AccountCharlist;

    fn new_account() -> AccountCharlist {
        AccountCharlist {
            identifier: Uuid::new_v4(),
            username: "admin".to_string(),
            password: "admin".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn it_sets_the_token_on_first_login() {
        let repository = TestAccountRepository::new().await;

        let account = new_account();
        repository.add_account(account.clone(), None).await;

        NumericToken {
            token: "1208".to_string(),
            changing: false,
        }
        .handle_impl(account.identifier, false, repository.account_repository())
        .await
        .unwrap();

        assert_eq!(
            repository
                .account_repository()
                .get_token(account.identifier)
                .await
                .unwrap(),
            Some("1208".to_string())
        );
    }

    mod given_an_already_set_token {
        use super::*;

        #[tokio::test]
        async fn when_the_user_tries_to_login_with_incorrect_token_then_it_returns_an_error() {
            let repository = TestAccountRepository::new().await;
            let account = new_account();
            repository
                .add_account(account.clone(), Some("1208".to_string()))
                .await;

            assert!(matches!(
                NumericToken {
                    token: "1111".to_string(),
                    changing: false,
                }
                .handle_impl(account.identifier, false, repository.account_repository(),)
                .await,
                Err(NumericTokenError::IncorrectToken(_))
            ));
        }

        #[tokio::test]
        async fn when_the_user_logs_in_with_correct_token_then_it_does_not_update_the_token() {
            let repository = TestAccountRepository::new().await;
            let account = new_account();
            let original_token = "1208".to_string();
            repository
                .add_account(account.clone(), Some(original_token.clone()))
                .await;

            // Get token before operation
            let token_before = repository
                .account_repository()
                .get_token(account.identifier)
                .await
                .unwrap()
                .unwrap();

            assert!(NumericToken {
                token: original_token.clone(),
                changing: false,
            }
            .handle_impl(
                account.identifier,
                false,
                repository.account_repository().clone()
            )
            .await
            .is_ok());

            let token_after = repository
                .account_repository()
                .get_token(account.identifier)
                .await
                .unwrap()
                .unwrap();

            assert_eq!(token_before, token_after);
            assert_eq!(token_after, original_token);
        }

        mod when_the_user_tries_to_change_the_token {
            use super::*;

            #[tokio::test]
            async fn without_entering_the_correct_token_beforehand_it_returns_an_error() {
                let repository = TestAccountRepository::new().await;
                let account = new_account();
                repository
                    .add_account(account.clone(), Some("1208".to_string()))
                    .await;

                assert!(matches!(
                    NumericToken {
                        token: "1111".to_string(),
                        changing: true,
                    }
                    .handle(
                        &MockSession::default(),
                        account.identifier,
                        false,
                        repository.account_repository()
                    )
                    .await,
                    Err(NumericTokenError::IncorrectState)
                ));
            }

            #[tokio::test]
            async fn with_previously_correct_token_it_changes_the_token() {
                let repository = TestAccountRepository::new().await;
                let account = new_account();
                repository
                    .add_account(account.clone(), Some("1208".to_string()))
                    .await;

                NumericToken {
                    token: "1111".to_string(),
                    changing: true,
                }
                .handle_impl(account.identifier, true, repository.account_repository())
                .await
                .unwrap();

                assert_eq!(
                    repository
                        .account_repository()
                        .get_token(account.identifier)
                        .await
                        .unwrap(),
                    Some("1111".to_string())
                )
            }
        }
    }
}
