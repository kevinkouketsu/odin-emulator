use chrono::Local;
use entity::{
    account::Entity as AccountEntity,
    account_ban::Entity as AccountBanEntity,
    character::{Entity as CharacterEntity, Model as Character},
    item::{Entity as ItemEntity, ItemCategory},
};
use futures::{future::BoxFuture, FutureExt};
use odin_models::{
    account::{AccessLevel, Ban, BanType},
    account_charlist::{AccountCharlist, CharacterInfo},
    character::Class,
    item::Item,
    nickname::Nickname,
    position::Position,
    status::Score,
};
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
use sea_orm::{
    prelude::*, ActiveValue, DatabaseConnection, QueryOrder, QuerySelect, SelectColumns, Set,
};
use sea_query::{Func, Query};
use std::str::FromStr;

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
            identifier: character.id,
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
            class: Class::try_from(character.class as i32)
                .map_err(|x| AccountRepositoryError::Generic(x.to_string()))?,
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
    ) -> BoxFuture<'a, Result<Option<AccountCharlist>, AccountRepositoryError>> {
        async move {
            let Some(account) = AccountEntity::find()
                .filter(
                    Expr::expr(Func::lower(Expr::col(entity::account::Column::Username)))
                        .eq(Expr::expr(Func::lower(Expr::value(username)))),
                )
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

            let access = match account.access {
                1..=99 => Some(AccessLevel::GameMaster(account.access as u32)),
                100 => Some(AccessLevel::Administrator),
                _ => None,
            };

            let charlist = self.fetch_charlist(account.id).await?;
            Ok(Some(AccountCharlist {
                identifier: account.id,
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
                charlist,
            }))
        }
        .boxed()
    }

    fn fetch_charlist(
        &self,
        account_id: Uuid,
    ) -> BoxFuture<Result<Vec<(usize, CharacterInfo)>, AccountRepositoryError>> {
        async move {
            let characters = CharacterEntity::find()
                .filter(entity::character::Column::AccountId.eq(account_id))
                .order_by_asc(entity::character::Column::Slot)
                .all(&self.connection)
                .await
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?;

            let mut charlist = vec![];
            for character in characters {
                let slot = character.slot as usize;
                let character = self.load_character(character).await?;

                charlist.push((slot, character))
            }

            Ok(charlist)
        }
        .boxed()
    }

    fn update_token(
        &self,
        id: Uuid,
        new_token: Option<String>,
    ) -> BoxFuture<Result<(), AccountRepositoryError>> {
        async move {
            let account = entity::account::ActiveModel {
                id: ActiveValue::Unchanged(id),
                token: Set(new_token.map(Into::into)),
                ..Default::default()
            };

            AccountEntity::update(account)
                .exec(&self.connection)
                .await
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?;

            Ok(())
        }
        .boxed()
    }

    fn get_token(&self, id: Uuid) -> BoxFuture<Result<Option<String>, AccountRepositoryError>> {
        async move {
            Ok(AccountEntity::find()
                .filter(entity::account::Column::Id.eq(id))
                .select_column(entity::account::Column::Token)
                .one(&self.connection)
                .await
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?
                .and_then(|x| x.token))
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
            let query = Query::select()
                .expr(
                    Func::cust(CreateCharacter)
                        .arg(account_id.to_string())
                        .arg(name.to_string())
                        .arg(slot as i32)
                        .arg(i32::from(class)),
                )
                .to_owned();

            let r = self
                .connection
                .query_one(self.connection.get_database_backend().build(&query))
                .await
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?
                .ok_or_else(|| {
                    AccountRepositoryError::Generic("Fail to create character".to_string())
                })?;

            let r: String = r
                .try_get_by_index(0)
                .map_err(|e| AccountRepositoryError::Generic(e.to_string()))?;

            Ok(Uuid::from_str(&r).unwrap())
        }
        .boxed()
    }

    fn name_exists<'a>(
        &'a self,
        name: &'a Nickname,
    ) -> BoxFuture<'a, Result<bool, AccountRepositoryError>> {
        async move {
            Ok(CharacterEntity::find()
                .select_only()
                .column_as(entity::character::Column::Id.count(), "count")
                .filter(
                    Expr::expr(Func::lower(Expr::col(entity::character::Column::Name)))
                        .eq(Expr::expr(Func::lower(Expr::value(name.to_string())))),
                )
                .count(&self.connection)
                .await
                .map_err(|e| AccountRepositoryError::FailToLoad(e.to_string()))?
                > 0)
        }
        .boxed()
    }
}

#[derive(Iden)]
#[iden = "CreateCharacter"]
struct CreateCharacter;
