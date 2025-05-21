pub mod login;

#[cfg(test)]
pub mod tests {
    use crate::{
        configuration::{CliVer, Configuration, ServerState},
        session::{SessionError, SessionTrait},
    };
    use deku::prelude::*;
    use odin_database::{
        entity::item::ItemCategory,
        sea_orm::{prelude::*, ActiveValue::NotSet, Database, Set},
        DatabaseService,
    };
    use odin_models::{
        account_charlist::AccountCharlist, character::Character, item::Item, uuid::Uuid,
    };
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
        }

        pub async fn add_character(&self, account_id: Uuid, character: Character) {
            odin_database::entity::character::Entity::insert(
                odin_database::entity::character::ActiveModel {
                    id: Set(character.identifier),
                    account_id: Set(Some(account_id)),
                    slot: Set(character.slot),
                    name: Set(character.name.clone()),
                    class: Set(character.class.into()),
                    coin: Set(character.coin),
                    last_pos: Set(format!("({})", character.last_pos)),
                    evolution: Set(character.evolution.into()),
                    ..Default::default()
                },
            )
            .exec_without_returning(&self.get_connection())
            .await
            .unwrap();

            let mut items = Self::get_items(
                character.identifier,
                &character.equipments,
                ItemCategory::Equip,
            );
            items.extend(Self::get_items(
                character.identifier,
                &character.inventory,
                ItemCategory::Inventory,
            ));

            if !items.is_empty() {
                odin_database::entity::item::Entity::insert_many(items)
                    .exec_without_returning(&self.get_connection())
                    .await
                    .unwrap();
            }
        }

        fn get_items<T: Into<usize> + Copy>(
            character_id: Uuid,
            items: &[(T, Item)],
            category: ItemCategory,
        ) -> Vec<odin_database::entity::item::ActiveModel> {
            items
                .iter()
                .map(
                    |(index, equipment)| odin_database::entity::item::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        r#type: Set(category),
                        slot: Set((*index).into() as i16),
                        item_id: Set(equipment.id as i16),
                        ef1: Set(equipment.effects[0].index as i16),
                        efv1: Set(equipment.effects[0].value as i16),
                        ef2: Set(equipment.effects[1].index as i16),
                        efv2: Set(equipment.effects[1].value as i16),
                        ef3: Set(equipment.effects[2].index as i16),
                        efv3: Set(equipment.effects[2].value as i16),
                        character_id: Set(character_id),
                        ..Default::default()
                    },
                )
                .collect::<Vec<_>>()
        }

        pub async fn add_account_with_characters(
            &self,
            account_charlist: AccountCharlist,
            characters: Vec<Character>,
        ) -> Uuid {
            let account_id = account_charlist.identifier;
            self.add_account(account_charlist, None).await;

            for character in characters {
                self.add_character(account_id, character).await;
            }

            account_id
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
