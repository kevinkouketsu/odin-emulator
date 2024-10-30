use chrono::Local;
use entity::{
    account::Entity as AccountEntity,
    account_ban::Entity as AccountBanEntity,
    character::{Entity as CharacterEntity, Model as Character},
    item::{Entity as ItemEntity, ItemCategory},
};
use futures::FutureExt;
use odin_models::{
    account::{AccessLevel, Ban, BanType},
    account_charlist::{AccountCharlist, CharacterInfo},
    character::Class,
    item::Item,
    position::Position,
    status::Score,
};
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
use sea_orm::{prelude::*, DatabaseConnection, QueryOrder};
use std::{future::Future, pin::Pin};

#[derive(Clone)]
pub struct PostgreSqlAccountRepository {
    connection: DatabaseConnection,
}
impl PostgreSqlAccountRepository {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    async fn load_character(
        &self,
        character: Character,
    ) -> Result<CharacterInfo, AccountRepositoryError> {
        let equipments = ItemEntity::find()
            .filter(entity::item::Column::Type.eq(ItemCategory::Equip))
            .filter(entity::item::Column::CharacterId.eq(character.id))
            .all(&self.connection)
            .await
            .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?
            .into_iter()
            .map(|item| {
                (
                    item.slot as usize,
                    Item::from((
                        item.item_id as u16,
                        item.ef1 as u8,
                        item.efv1 as u8,
                        item.ef2 as u8,
                        item.efv2 as u8,
                        item.ef3 as u8,
                        item.efv3 as u8,
                    )),
                )
            })
            .collect();

        Ok(CharacterInfo {
            name: character.name,
            status: Score {
                level: character.level as u16,
                reserved: character.reserved as i8,
                hp: character.current_hp as u32,
                mp: character.current_mp as u32,
                strength: character.strength as u16,
                intelligence: character.intelligence as u16,
                dexterity: character.dexterity as u16,
                constitution: character.constitution as u16,
                specials: [
                    character.special0 as u16,
                    character.special1 as u16,
                    character.special2 as u16,
                    character.special3 as u16,
                ],
                ..Default::default()
            },
            guild: character.guild_id.map(|x| x as u16),
            class: Class::new(character.class).ok_or_else(|| {
                AccountRepositoryError::Generic("Could not get class".to_string())
            })?,
            coin: character.coin as u32,
            experience: character.experience,
            position: Position::try_from(character.last_pos.as_str())
                .map_err(|err| AccountRepositoryError::Generic(err.to_string()))?,
            equipments,
        })
    }
}
impl AccountRepository for PostgreSqlAccountRepository {
    fn fetch_account<'a>(
        &'a self,
        username: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<AccountCharlist>, AccountRepositoryError>> + 'a>>
    {
        async move {
            let Some(account) = AccountEntity::find()
                .filter(entity::account::Column::Username.eq(username))
                .one(&self.connection)
                .await
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?
            else {
                return Ok(None);
            };

            let ban = AccountBanEntity::find()
                .filter(entity::account_ban::Column::AccountId.eq(account.id))
                .filter(entity::account_ban::Column::ExpiresAt.gt(Local::now()))
                .order_by_desc(entity::account_ban::Column::ExpiresAt)
                .one(&self.connection)
                .await
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?;

            let characters = CharacterEntity::find()
                .filter(entity::character::Column::AccountId.eq(account.id))
                .order_by_asc(entity::character::Column::Slot)
                .all(&self.connection)
                .await
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?;

            let access = match account.access {
                1..=99 => Some(AccessLevel::GameMaster(account.access as u32)),
                100 => Some(AccessLevel::Administrator),
                _ => None,
            };

            let mut charlist = vec![];
            for character in characters {
                let slot = character.slot as usize;
                let character = self.load_character(character).await?;

                charlist.push((slot, character))
            }

            Ok(Some(AccountCharlist {
                username: account.username,
                password: account.password,
                ban: ban.map(|ban| Ban {
                    expiration: ban.expires_at,
                    r#type: match ban.r#type {
                        entity::account_ban::BanType::Analysis => BanType::Analysis,
                        entity::account_ban::BanType::Blocked => BanType::Blocked,
                    },
                }),
                access,
                storage: Default::default(),
                token: account.token,
                charlist,
            }))
        }
        .boxed()
    }
}
