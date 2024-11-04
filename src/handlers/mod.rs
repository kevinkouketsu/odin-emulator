pub mod authentication;
pub mod create_character;
pub mod numeric_token;

#[cfg(test)]
pub mod tests {
    use super::authentication::CliVer;
    use crate::{
        configuration::{Configuration, ServerState},
        session::{SessionError, SessionTrait},
    };
    use deku::prelude::*;
    use futures::{future::BoxFuture, FutureExt};
    use odin_models::{
        account_charlist::{AccountCharlist, CharacterInfo},
        character::Class,
        nickname::Nickname,
        uuid::Uuid,
    };
    use odin_networking::WritableResource;
    use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
    use std::{
        collections::HashMap,
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
        ) -> BoxFuture<Result<Option<AccountCharlist>, AccountRepositoryError>> {
            async move {
                let accounts = self.accounts.read().unwrap();
                let Some(account) = accounts.get(username) else {
                    return Ok(None);
                };

                Ok(Some(account.account_charlist.clone()))
            }
            .boxed()
        }

        fn fetch_charlist(
            &self,
            account_id: Uuid,
        ) -> BoxFuture<Result<Vec<(usize, CharacterInfo)>, AccountRepositoryError>> {
            async move {
                let accounts = self.accounts.read().unwrap();
                let Some((_, account)) = accounts
                    .iter()
                    .find(|account| account.1.account_charlist.identifier == account_id)
                else {
                    return Ok(vec![]);
                };

                Ok(account.account_charlist.charlist.clone())
            }
            .boxed()
        }

        fn update_token(
            &self,
            id: Uuid,
            new_token: Option<String>,
        ) -> BoxFuture<Result<(), AccountRepositoryError>> {
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

        fn get_token(&self, id: Uuid) -> BoxFuture<Result<Option<String>, AccountRepositoryError>> {
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

        fn create_character<'a>(
            &'a self,
            account_id: Uuid,
            slot: u32,
            name: &'a Nickname,
            class: Class,
        ) -> BoxFuture<'a, Result<Uuid, AccountRepositoryError>> {
            async move {
                let mut accounts = self.accounts.write().unwrap();
                let (_, account) = accounts
                    .iter_mut()
                    .find(|account| account.1.account_charlist.identifier == account_id)
                    .ok_or_else(|| {
                        AccountRepositoryError::FailToLoad("Could not find account".to_string())
                    })?;

                let uuid = Uuid::new_v4();
                account.account_charlist.charlist[slot as usize].1 = CharacterInfo {
                    identifier: uuid,
                    class,
                    name: name.to_string(),
                    ..Default::default()
                };

                Ok(uuid)
            }
            .boxed()
        }

        fn name_exists<'a>(
            &'a self,
            name: &'a Nickname,
        ) -> BoxFuture<'a, Result<bool, AccountRepositoryError>> {
            async move {
                let accounts = self.accounts.read().unwrap();
                Ok(accounts.iter().any(|account| {
                    account
                        .1
                        .account_charlist
                        .charlist
                        .iter()
                        .any(|character| character.1.name.as_str() == name.as_str())
                }))
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
