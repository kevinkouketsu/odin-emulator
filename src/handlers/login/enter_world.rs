use crate::map::{EntityId, InsertResult, Map, MapError};
use odin_models::character::Character;
use odin_models::uuid::Uuid;
use odin_networking::{WritableResourceError, messages::client::enter_world::EnterWorldRaw};
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};

#[derive(Debug)]
pub struct EnterWorld {
    pub slot: u32,
    pub force: bool,
    pub secret_code: String,
}
impl EnterWorld {
    pub async fn handle<A: AccountRepository>(
        &self,
        account_id: Uuid,
        client_id: usize,
        account_repository: A,
        map: &mut Map,
    ) -> Result<Character, EnterWorldError> {
        let character = account_repository
            .fetch_character(account_id, self.slot as usize)
            .await?
            .ok_or(EnterWorldError::CharacterNotFound)?;

        let entity_id = EntityId::Player(client_id);
        let insert_result = map.force_insert(entity_id, character.last_pos)?;

        Ok(character)
    }
}
impl TryFrom<EnterWorldRaw> for EnterWorld {
    type Error = WritableResourceError;

    fn try_from(value: EnterWorldRaw) -> Result<Self, Self::Error> {
        Ok(EnterWorld {
            slot: value.slot,
            force: value.force != 0,
            secret_code: value.secret_code.try_into()?,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EnterWorldError {
    #[error(transparent)]
    Repository(#[from] AccountRepositoryError),
    #[error("Character not found")]
    CharacterNotFound,
    #[error(transparent)]
    Map(#[from] MapError),
}
