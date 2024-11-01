pub mod authentication;
pub mod numeric_token;

#[cfg(test)]
pub mod tests {
    use super::authentication::CliVer;
    use crate::{
        configuration::{Configuration, ServerState},
        session::{SessionError, SessionTrait},
    };
    use deku::prelude::*;
    use futures::FutureExt;
    use odin_models::{account_charlist::AccountCharlist, uuid::Uuid};
    use odin_networking::WritableResource;
    use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
    use std::{
        collections::HashMap,
        future::Future,
        pin::Pin,
        sync::{Arc, RwLock},
    };

    #[derive(Default, Clone)]
    pub struct MockAccountCharlist {
        pub account_charlist: AccountCharlist,
        pub token: Option<String>,
    }

    #[derive(Default, Clone)]
    pub struct MockAccountRepository {
        accounts: Arc<RwLock<HashMap<String, MockAccountCharlist>>>,
    }
    impl MockAccountRepository {
        pub fn add_account(&mut self, account: MockAccountCharlist) {
            assert_ne!(account.account_charlist.identifier, Uuid::default());
            self.accounts
                .write()
                .unwrap()
                .insert(account.account_charlist.username.clone(), account);
        }

        pub fn get_account(&mut self, username: &str) -> Option<MockAccountCharlist> {
            self.accounts
                .read()
                .unwrap()
                .iter()
                .find_map(|x| (x.0 == username).then_some(x.1))
                .cloned()
        }

        pub fn edit_account<F: FnOnce(&mut MockAccountCharlist)>(
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
            account_repository.add_account(MockAccountCharlist {
                account_charlist: AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: value.0.to_string(),
                    password: value.1.to_string(),
                    ban: None,
                    ..Default::default()
                },
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

                Ok(Some(account.account_charlist.clone()))
            }
            .boxed()
        }

        fn update_token<'a>(
            &'a self,
            id: Uuid,
            new_token: Option<String>,
        ) -> Pin<Box<dyn Future<Output = Result<(), AccountRepositoryError>> + 'a>> {
            async move {
                let mut accounts = self.accounts.write().unwrap();
                let Some((_, account)) = accounts
                    .iter_mut()
                    .find(|account| account.1.account_charlist.identifier == id)
                else {
                    return Ok(());
                };

                assert_ne!(account.token, new_token);
                account.token = new_token;
                Ok(())
            }
            .boxed()
        }

        fn get_token<'a>(
            &'a self,
            id: Uuid,
        ) -> Pin<Box<dyn Future<Output = Result<Option<String>, AccountRepositoryError>> + 'a>>
        {
            async move {
                let accounts = self.accounts.read().unwrap();

                Ok(accounts
                    .iter()
                    .find_map(|account| {
                        (account.1.account_charlist.identifier == id)
                            .then(|| account.1.token.clone())
                    })
                    .flatten())
            }
            .boxed()
        }
    }

    #[derive(Default)]
    pub struct MockSession {
        messages: RwLock<Vec<Vec<u8>>>,
    }
    impl SessionTrait for MockSession {
        fn send<R: WritableResource>(&self, message: R) -> Result<(), SessionError> {
            let message: Vec<u8> = message.write().unwrap().to_bytes().unwrap();
            self.messages.try_write().unwrap().push(message);

            Ok(())
        }
    }

    pub struct MockConfiguration(pub CliVer, pub ServerState);
    impl Configuration for MockConfiguration {
        fn get_current_cliver(&self) -> CliVer {
            self.0
        }

        fn get_server_state(&self) -> ServerState {
            self.1
        }
    }
}
