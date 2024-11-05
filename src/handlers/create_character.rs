use crate::session::{SessionError, SessionTrait};
use odin_models::{
    account_charlist::CharacterInfo,
    character::Class,
    nickname::{InvalidNicknameError, Nickname},
    uuid::Uuid,
    MAX_CHARACTERS,
};
use odin_networking::{
    messages::{
        client::create_character::CreateCharacterRaw,
        server::{
            charlist::{NameAlreadyExistsError, UpdateCharlist},
            message_panel::MessagePanel,
        },
    },
    WritableResourceError,
};
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct CreateCharacter {
    name: String,
    class: Class,
    slot: u32,
}
impl CreateCharacter {
    pub async fn handle<A: AccountRepository, S: SessionTrait>(
        &self,
        session: &S,
        account_id: Uuid,
        account_repository: A,
    ) -> Result<Vec<(usize, CharacterInfo)>, CreateCharacterError> {
        match self.handle_impl(account_id, account_repository).await {
            Ok(charlist) => {
                session.send(UpdateCharlist {
                    character_info: charlist
                        .clone()
                        .into_iter()
                        .map(|(index, character)| (index, character.into()))
                        .collect(),
                })?;

                Ok(charlist)
            }
            Err(e) => {
                log::error!("Could not create character: {e:?}");

                match e {
                    CreateCharacterError::InvalidNickname(_) => {
                        session.send::<MessagePanel>("Nome inadequado".into())
                    }
                    _ => session.send(NameAlreadyExistsError),
                }?;

                Err(e)
            }
        }
    }

    async fn handle_impl<A: AccountRepository>(
        &self,
        account_id: Uuid,
        account_repository: A,
    ) -> Result<Vec<(usize, CharacterInfo)>, CreateCharacterError> {
        if self.slot as usize >= MAX_CHARACTERS {
            return Err(CreateCharacterError::InvalidSlot(self.slot as usize));
        }

        let nickname: Nickname = self.name.clone().try_into()?;
        if account_repository.name_exists(&nickname).await? {
            return Err(CreateCharacterError::NameAlreadyExists);
        }

        let _ = account_repository
            .create_character(account_id, self.slot, &nickname, self.class)
            .await?;

        let charlist = account_repository.fetch_charlist(account_id).await?;
        Ok(charlist)
    }
}
impl TryFrom<CreateCharacterRaw> for CreateCharacter {
    type Error = WritableResourceError;

    fn try_from(value: CreateCharacterRaw) -> Result<Self, Self::Error> {
        let class = Class::try_from(value.class)
            .map_err(|e| WritableResourceError::Generic(e.to_string()))?;

        Ok(Self {
            class,
            name: value.name.try_into()?,
            slot: value.slot,
        })
    }
}

#[derive(Debug, Error)]
pub enum CreateCharacterError {
    #[error(transparent)]
    AccountRepositoryError(#[from] AccountRepositoryError),

    #[error(transparent)]
    InvalidNickname(#[from] InvalidNicknameError),

    #[error("Name already exists")]
    NameAlreadyExists,

    #[error("Slot {0} is not within valid range")]
    InvalidSlot(usize),

    #[error(transparent)]
    SessionError(#[from] SessionError),
}
