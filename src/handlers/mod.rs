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
    use odin_database::{
        sea_orm::{prelude::*, ActiveValue::NotSet, Database, Set},
        DatabaseService,
    };
    use odin_models::{account_charlist::AccountCharlist, uuid::Uuid};
    use odin_networking::WritableResource;
    use std::{ops::Deref, sync::RwLock};

    #[derive(Clone)]
    pub struct TestAccountRepository {
        pub connection: DatabaseService,
    }
    impl TestAccountRepository {
        pub async fn new() -> Self {
            let connection =
                DatabaseService::from_database(Database::connect("sqlite::memory:").await.unwrap());
            connection.fresh().await.unwrap();

            Self { connection }
        }

        pub async fn add_account(&self, account: AccountCharlist, token: Option<String>) {
            let account_identifier = if account.identifier != Uuid::default() {
                Set(account.identifier)
            } else {
                NotSet
            };
            let account_model = odin_database::entity::account::ActiveModel {
                id: account_identifier,
                username: Set(account.username),
                password: Set(account.password),
                access: Set(account.access.map(|x| x.get_level() as i32).unwrap_or(0)),
                storage_coin: Set(account.storage.coin as i64),
                token: Set(token),
                ..Default::default()
            }
            .insert(&self.get_connection())
            .await
            .unwrap();

            if let Some(ban) = account.ban {
                odin_database::entity::account_ban::Entity::insert(
                    odin_database::entity::account_ban::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        account_id: Set(account_model.id),
                        account_banned_by: Set(account_model.id),
                        expires_at: Set(ban.expiration),
                        reason: Set("".to_string()),
                        r#type: Set(match ban.r#type {
                            odin_models::account::BanType::Analysis => {
                                odin_database::entity::account_ban::BanType::Analysis
                            }
                            odin_models::account::BanType::Blocked => {
                                odin_database::entity::account_ban::BanType::Blocked
                            }
                        }),
                        ..Default::default()
                    },
                )
                .exec_without_returning(&self.get_connection())
                .await
                .unwrap();
            }

            if account.charlist.is_empty() {
                return;
            }

            odin_database::entity::character::Entity::insert_many(
                account
                    .charlist
                    .into_iter()
                    .map(
                        |(i, character)| odin_database::entity::character::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            account_id: Set(Some(account_model.id)),
                            slot: Set(i as i32),
                            name: Set(character.name),
                            class: Set(character.class.into()),
                            last_pos: Set(format!("({})", character.position)),
                            ..Default::default()
                        },
                    )
                    .collect::<Vec<_>>(),
            )
            .exec_without_returning(&self.get_connection())
            .await
            .unwrap();
        }
    }
    impl Deref for TestAccountRepository {
        type Target = DatabaseService;

        fn deref(&self) -> &Self::Target {
            &self.connection
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
