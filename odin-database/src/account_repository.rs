use chrono::Local;
use entity::{
    account::Entity as AccountEntity,
    account_ban::Entity as AccountBanEntity,
    character::{Entity as CharacterEntity, Evolution, Model as Character},
    item::{Entity as ItemEntity, ItemCategory},
    start_item::Entity as StartItemEntity,
};
use odin_models::{
    EquipmentSlot, EquipmentSlots, InventorySlots,
    account::{AccessLevel, Ban, BanType},
    account_charlist::{AccountCharlist, CharacterInfo},
    character::{Character as CharacterModel, Class, GuildLevel},
    item::Item,
    nickname::Nickname,
    position::Position,
    status::Score,
};
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
use sea_orm::{
    ActiveValue, DatabaseConnection, FromQueryResult, QueryOrder, QuerySelect, SelectColumns, Set,
    TransactionTrait, prelude::*,
};
use sea_query::Func;

#[derive(Clone)]
pub struct DatabaseAccountRepository {
    connection: DatabaseConnection,
}
impl DatabaseAccountRepository {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    async fn load_character(
        &self,
        character: Character,
    ) -> Result<CharacterInfo, AccountRepositoryError> {
        let equipments: EquipmentSlots = ItemEntity::find()
            .filter(entity::item::Column::Type.eq(ItemCategory::Equip))
            .filter(entity::item::Column::CharacterId.eq(character.id))
            .all(&self.connection)
            .await
            .map_err(map_to_fail_to_load)?
            .into_iter()
            .filter_map(|item| {
                let slot = EquipmentSlot::try_from(item.slot as usize).ok()?;
                Some((
                    slot,
                    Item::from((
                        item.item_id as u16,
                        item.ef1 as u8,
                        item.efv1 as u8,
                        item.ef2 as u8,
                        item.efv2 as u8,
                        item.ef3 as u8,
                        item.efv3 as u8,
                    )),
                ))
            })
            .collect::<Vec<_>>()
            .into();

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
impl AccountRepository for DatabaseAccountRepository {
    async fn fetch_account(
        &self,
        username: &str,
    ) -> Result<Option<AccountCharlist>, AccountRepositoryError> {
        let Some(account) = AccountEntity::find()
            .filter(
                Expr::expr(Func::lower(Expr::col(entity::account::Column::Username)))
                    .eq(Expr::expr(Func::lower(Expr::value(username)))),
            )
            .one(&self.connection)
            .await
            .map_err(map_to_fail_to_load)?
        else {
            return Ok(None);
        };

        let ban = AccountBanEntity::find()
            .filter(entity::account_ban::Column::AccountId.eq(account.id))
            .filter(entity::account_ban::Column::ExpiresAt.gt(Local::now()))
            .order_by_desc(entity::account_ban::Column::ExpiresAt)
            .one(&self.connection)
            .await
            .map_err(map_to_fail_to_load)?;

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

    async fn fetch_charlist(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<(usize, CharacterInfo)>, AccountRepositoryError> {
        let characters = CharacterEntity::find()
            .filter(entity::character::Column::AccountId.eq(account_id))
            .order_by_asc(entity::character::Column::Slot)
            .all(&self.connection)
            .await
            .map_err(map_to_fail_to_load)?;

        let mut charlist = vec![];
        for character in characters {
            let slot = character.slot as usize;
            let character = self.load_character(character).await?;

            charlist.push((slot, character))
        }

        Ok(charlist)
    }

    async fn fetch_character(
        &self,
        account_id: Uuid,
        slot: usize,
    ) -> Result<Option<CharacterModel>, AccountRepositoryError> {
        let Some(character) = CharacterEntity::find()
            .filter(entity::character::Column::AccountId.eq(account_id))
            .filter(entity::character::Column::Slot.eq(slot as i32))
            .one(&self.connection)
            .await
            .map_err(map_to_fail_to_load)?
        else {
            return Ok(None);
        };

        let items = ItemEntity::find()
            .filter(entity::item::Column::CharacterId.eq(character.id))
            .order_by_asc(entity::item::Column::Type)
            .all(&self.connection)
            .await
            .map_err(map_to_fail_to_load)?;

        let mut equipments = EquipmentSlots::default();
        let mut inventory = InventorySlots::default();

        for item in items {
            match item.r#type {
                ItemCategory::Equip => {
                    let slot: EquipmentSlot = (item.slot as usize)
                        .try_into()
                        .map_err(AccountRepositoryError::CharacterNotValid)?;
                    equipments.set(slot, item.into());
                }
                ItemCategory::Inventory => {
                    inventory.set(item.slot as usize, item.into());
                }
            }
        }

        Ok(Some(CharacterModel {
            identifier: character.id,
            name: character.name,
            slot: character.slot,
            score: Score {
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
            evolution: character.evolution.into(),
            merchant: character.merchant,
            guild: character.guild_id,
            guild_level: character.guild_level.and_then(GuildLevel::new),
            class: character.class.into(),
            affect_info: character.affect_info,
            quest_info: character.quest_info,
            coin: character.coin,
            experience: character.experience,
            last_pos: Position::try_from(character.last_pos.as_str()).unwrap_or_default(),
            inventory,
            equipments,
        }))
    }

    async fn update_token(
        &self,
        id: Uuid,
        new_token: Option<String>,
    ) -> Result<(), AccountRepositoryError> {
        let account = entity::account::ActiveModel {
            id: ActiveValue::Set(id),
            token: Set(new_token),
            ..Default::default()
        };

        account
            .update(&self.connection)
            .await
            .map_err(map_to_fail_to_load)?;

        Ok(())
    }
    async fn get_token(&self, id: Uuid) -> Result<Option<String>, AccountRepositoryError> {
        Ok(AccountEntity::find()
            .filter(entity::account::Column::Id.eq(id))
            .select_column(entity::account::Column::Token)
            .one(&self.connection)
            .await
            .map_err(map_to_fail_to_load)?
            .and_then(|x| x.token))
    }

    async fn create_character(
        &self,
        account_id: Uuid,
        slot: u32,
        name: &Nickname,
        class: Class,
    ) -> Result<Uuid, AccountRepositoryError> {
        let Some(base_character) = CharacterEntity::find()
            .filter(entity::character::Column::AccountId.is_null())
            .filter(entity::character::Column::Class.eq::<entity::character::Class>(class.into()))
            .select_only()
            .into_partial_model::<BaseCharacterPartialModel>()
            .one(&self.connection)
            .await
            .map_err(map_to_fail_to_load)?
        else {
            tracing::error!(?class, "Could not fetch the base character",);

            return Err(AccountRepositoryError::Generic(
                "Could not find to fetch base character".to_string(),
            ));
        };

        let name = name.to_string();
        self.connection
            .transaction(|transaction| {
                Box::pin(async move {
                    let uuid = Uuid::new_v4();
                    CharacterEntity::insert(entity::character::ActiveModel {
                        id: Set(uuid),
                        account_id: Set(Some(account_id)),
                        slot: Set(slot as i32),
                        name: Set(name),
                        class: Set(class.into()),
                        coin: Set(base_character.coin),
                        experience: Set(base_character.experience),
                        evolution: Set(Evolution::Mortal),
                        last_pos: Set(base_character.last_pos),
                        level: Set(base_character.level),
                        strength: Set(base_character.strength),
                        intelligence: Set(base_character.intelligence),
                        dexterity: Set(base_character.dexterity),
                        constitution: Set(base_character.constitution),
                        special0: Set(base_character.special0),
                        special1: Set(base_character.special1),
                        special2: Set(base_character.special2),
                        special3: Set(base_character.special3),
                        current_hp: Set(base_character.current_hp),
                        current_mp: Set(base_character.current_mp),
                        ..Default::default()
                    })
                    .exec_without_returning(transaction)
                    .await?;

                    let start_items = StartItemEntity::find()
                        .filter(
                            entity::start_item::Column::Class
                                .eq::<entity::character::Class>(class.into()),
                        )
                        .all(transaction)
                        .await?;

                    entity::item::Entity::insert_many(
                        start_items
                            .into_iter()
                            .map(|item| entity::item::ActiveModel {
                                id: Set(Uuid::new_v4()),
                                r#type: Set(item.r#type),
                                item_id: Set(item.item_id),
                                ef1: Set(item.ef1),
                                efv1: Set(item.efv1),
                                ef2: Set(item.ef2),
                                efv2: Set(item.efv2),
                                ef3: Set(item.ef3),
                                efv3: Set(item.efv3),
                                ef4: Set(item.ef4),
                                efv4: Set(item.efv4),
                                ef5: Set(item.ef5),
                                efv5: Set(item.efv5),
                                slot: Set(item.slot),
                                character_id: Set(uuid),
                            })
                            .collect::<Vec<_>>(),
                    )
                    .exec_without_returning(transaction)
                    .await?;

                    Result::<Uuid, DbErr>::Ok(uuid)
                })
            })
            .await
            .map_err(|err| match err {
                sea_orm::TransactionError::Connection(db_err) => map_to_generic(db_err),
                sea_orm::TransactionError::Transaction(db_err) => map_to_generic(db_err),
            })
    }

    async fn name_exists(&self, name: &Nickname) -> Result<bool, AccountRepositoryError> {
        let total = CharacterEntity::find()
            .select_only()
            .column(entity::character::Column::Id)
            .filter(
                Expr::expr(Func::lower(Expr::col(entity::character::Column::Name)))
                    .eq(Expr::expr(Func::lower(Expr::value(name.to_string())))),
            )
            .count(&self.connection)
            .await
            .map_err(map_to_generic)?;

        Ok(total > 0)
    }

    async fn delete_character(
        &self,
        account_id: Uuid,
        slot: usize,
    ) -> Result<(), AccountRepositoryError> {
        let character = CharacterEntity::find()
            .filter(entity::character::Column::AccountId.eq(account_id))
            .filter(entity::character::Column::Slot.eq(slot as i32))
            .one(&self.connection)
            .await
            .map_err(map_to_fail_to_load)?
            .ok_or(AccountRepositoryError::EntityNotFound)?;

        let r = character
            .delete(&self.connection)
            .await
            .map_err(map_to_generic)?;

        match r.rows_affected {
            0 => Err(AccountRepositoryError::Generic(
                "It was not possible to delete the character even if it was found".to_string(),
            )),
            1 => Ok(()),
            n => Err(AccountRepositoryError::Generic(format!(
                "{n} were characters deleted, this should not happen"
            ))),
        }
    }

    async fn check_password(
        &self,
        account_id: Uuid,
        password: &str,
    ) -> Result<bool, AccountRepositoryError> {
        let r: String = AccountEntity::find()
            .select_only()
            .column(entity::account::Column::Password)
            .filter(entity::account::Column::Id.eq(account_id))
            .into_tuple()
            .one(&self.connection)
            .await
            .map_err(map_to_fail_to_load)?
            .ok_or(AccountRepositoryError::EntityNotFound)?;

        Ok(r == password)
    }
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "CharacterEntity")]
struct BaseCharacterPartialModel {
    coin: i32,
    experience: i64,
    last_pos: String,
    level: i32,
    strength: i32,
    intelligence: i32,
    dexterity: i32,
    constitution: i32,
    special0: i32,
    special1: i32,
    special2: i32,
    special3: i32,
    current_hp: i32,
    current_mp: i32,
}

fn map_to_generic(err: DbErr) -> AccountRepositoryError {
    AccountRepositoryError::Generic(err.to_string())
}

fn map_to_fail_to_load(err: DbErr) -> AccountRepositoryError {
    AccountRepositoryError::Generic(err.to_string())
}
