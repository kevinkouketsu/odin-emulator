use crate::{
    services::{equipments::Equipments, inventory::Inventory},
    session::{SessionError, SessionTrait},
};
use odin_models::{
    account_charlist::CharacterInfo, character::Evolution, uuid::Uuid, EquipmentSlot,
};
use odin_networking::{
    messages::{
        client::delete_character::DeleteCharacterRaw,
        server::{charlist::UpdateCharlist, message_panel::MessagePanel},
    },
    WritableResourceError,
};
use odin_repositories::account_repository::{AccountRepository, AccountRepositoryError};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct DeleteCharacter {
    slot: usize,
    password: String,
}
impl DeleteCharacter {
    pub async fn handle<S: SessionTrait, A: AccountRepository>(
        &self,
        account_id: Uuid,
        session: &S,
        account_repository: A,
    ) -> Result<Vec<(usize, CharacterInfo)>, DeleteCharacterError> {
        match self.handle_impl(account_id, account_repository).await {
            Ok(new_charlist) => {
                session.send(UpdateCharlist::<true> {
                    character_info: new_charlist
                        .clone()
                        .into_iter()
                        .map(|(index, character)| (index, character.into()))
                        .collect(),
                })?;

                Ok(new_charlist)
            }
            Err(e) => {
                match e {
                    DeleteCharacterError::EquippedItems => session.send::<MessagePanel>(
                        "Desequipe todos os itens do personagem antes de deletar".into(),
                    )?,
                    DeleteCharacterError::InventoryItems => session.send::<MessagePanel>(
                        "Limpe seu inventário antes de deletar o personagem".into(),
                    )?,
                    DeleteCharacterError::IncorrectPassword => {
                        session.send::<MessagePanel>("Senha incorreta".into())?
                    }
                    DeleteCharacterError::Evolution(_) => session
                        .send::<MessagePanel>("Só é possível deletar personagens mortais".into())?,
                    DeleteCharacterError::Coin => session
                        .send::<MessagePanel>("Remova o gold do inventário para deletar".into())?,
                    _ => session.send::<MessagePanel>("Falha ao deletar".into())?,
                };

                Err(e)
            }
        }
    }

    pub async fn handle_impl<A: AccountRepository>(
        &self,
        account_id: Uuid,
        account_repository: A,
    ) -> Result<Vec<(usize, CharacterInfo)>, DeleteCharacterError> {
        if !account_repository
            .check_password(account_id, &self.password)
            .await?
        {
            return Err(DeleteCharacterError::IncorrectPassword);
        }

        let character = account_repository
            .fetch_character(account_id, self.slot)
            .await?
            .ok_or(DeleteCharacterError::CharacterNotFound)?;

        if character.coin != 0 {
            return Err(DeleteCharacterError::Coin);
        }

        if character.evolution > Evolution::Mortal {
            return Err(DeleteCharacterError::Evolution(character.evolution));
        }

        let equipments = Equipments::from(character.equipments);
        if equipments
            .iter()
            .any(|(slot, _)| slot != EquipmentSlot::Face && slot != EquipmentSlot::Mantle)
        {
            return Err(DeleteCharacterError::EquippedItems);
        }

        let inventory = Inventory::from(character.inventory);
        if inventory.iter().count() != 0 {
            return Err(DeleteCharacterError::InventoryItems);
        }

        account_repository
            .delete_character(account_id, self.slot)
            .await?;

        Ok(account_repository.fetch_charlist(account_id).await?)
    }
}
impl TryFrom<DeleteCharacterRaw> for DeleteCharacter {
    type Error = WritableResourceError;

    fn try_from(value: DeleteCharacterRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            slot: value.slot as usize,
            password: value.password.try_into()?,
        })
    }
}

#[derive(Debug, Error)]
pub enum DeleteCharacterError {
    #[error(transparent)]
    AccountRepositoryError(AccountRepositoryError),

    #[error("The character has not been found")]
    CharacterNotFound,

    #[error("There are equipped items")]
    EquippedItems,

    #[error("There are items in the inventory")]
    InventoryItems,

    #[error("There is gold in the inventory")]
    Coin,

    #[error("The password is incorrect")]
    IncorrectPassword,

    #[error("Can't delete character above mortal")]
    Evolution(Evolution),

    #[error(transparent)]
    SessionError(#[from] SessionError),
}
impl From<AccountRepositoryError> for DeleteCharacterError {
    fn from(value: AccountRepositoryError) -> Self {
        match value {
            AccountRepositoryError::FailToLoad(_) => {
                DeleteCharacterError::AccountRepositoryError(value)
            }
            AccountRepositoryError::Generic(_) => {
                DeleteCharacterError::AccountRepositoryError(value)
            }
            AccountRepositoryError::EntityNotFound
            | AccountRepositoryError::CharacterNotValid(_) => {
                DeleteCharacterError::CharacterNotFound
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::tests::TestAccountRepository;
    use odin_models::{
        account_charlist::AccountCharlist, character::Character, item::Item, uuid::Uuid,
    };
    use rstest::rstest;

    #[tokio::test]
    async fn it_should_delete_the_character() {
        let repository = TestAccountRepository::new().await;
        let character = Character {
            last_pos: (2100, 2100).into(),
            name: "charlist".to_string(),
            ..Default::default()
        };

        let account_id = Uuid::new_v4();
        repository
            .add_account(
                AccountCharlist {
                    identifier: account_id,
                    password: "admin".to_string(),
                    ..Default::default()
                },
                None,
            )
            .await;

        repository.add_character(account_id, character).await;

        let charlist = DeleteCharacter {
            password: "admin".to_string(),
            slot: 0,
        }
        .handle_impl(account_id, repository.account_repository())
        .await
        .unwrap();

        assert!(charlist.is_empty());
        assert!(repository
            .account_repository()
            .fetch_charlist(account_id)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn deleting_a_character_that_does_not_exist_returns_an_error() {
        let repository = TestAccountRepository::new().await;
        let account_id = Uuid::new_v4();
        repository
            .add_account(
                AccountCharlist {
                    identifier: account_id,
                    password: "admin".to_string(),
                    charlist: vec![],
                    ..Default::default()
                },
                None,
            )
            .await;

        let r = DeleteCharacter {
            password: "admin".to_string(),
            slot: 0,
        }
        .handle_impl(account_id, repository.account_repository())
        .await;

        match r {
            Ok(_) => panic!("Expected DeleteCharacterError::CharacterNotFound, got Ok"),
            Err(DeleteCharacterError::CharacterNotFound) => {}
            Err(e) => panic!("Expected DeleteCharacterError::CharacterNotFound, got {e:?}"),
        }
    }

    #[tokio::test]
    async fn cant_delete_character_with_equiped_items() {
        let repository = TestAccountRepository::new().await;
        let account_id = repository
            .add_account_with_characters(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: "admin".to_string(),
                    password: "admin".to_string(),
                    ..Default::default()
                },
                vec![Character {
                    name: "charlist".to_string(),
                    equipments: vec![(EquipmentSlot::Helmet, 3500.into())],
                    ..Default::default()
                }],
            )
            .await;

        let r = DeleteCharacter {
            password: "admin".to_string(),
            slot: 0,
        }
        .handle_impl(account_id, repository.account_repository())
        .await;

        match r {
            Ok(_) => panic!("Expected DeleteCharacterError::EquippedItems, got Ok"),
            Err(DeleteCharacterError::EquippedItems) => {}
            Err(e) => panic!("Expected DeleteCharacterError::EquippedItems, got {e:?}"),
        }
    }

    #[rstest]
    #[case((EquipmentSlot::Face, 11.into()))]
    #[case((EquipmentSlot::Mantle, 737.into()))]
    #[tokio::test]
    async fn it_ignores_mantle_and_face_equipments_to_delete_character(
        #[case] item: (EquipmentSlot, Item),
    ) {
        let repository = TestAccountRepository::new().await;
        let account_id = repository
            .add_account_with_characters(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    password: "admin".to_string(),
                    ..Default::default()
                },
                vec![Character {
                    name: "charlist".to_string(),
                    equipments: vec![item],
                    ..Default::default()
                }],
            )
            .await;

        let charlist = DeleteCharacter {
            password: "admin".to_string(),
            slot: 0,
        }
        .handle_impl(account_id, repository.account_repository())
        .await
        .unwrap();

        assert!(charlist.is_empty());
        assert!(repository
            .account_repository()
            .fetch_charlist(account_id)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn cant_delete_character_with_items_in_the_inventory() {
        let repository = TestAccountRepository::new().await;
        let account_id = repository
            .add_account_with_characters(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: "admin".to_string(),
                    password: "admin".to_string(),
                    ..Default::default()
                },
                vec![Character {
                    name: "charlist".to_string(),
                    inventory: vec![(0, 11.into())],
                    ..Default::default()
                }],
            )
            .await;

        let r = DeleteCharacter {
            password: "admin".to_string(),
            slot: 0,
        }
        .handle_impl(account_id, repository.account_repository())
        .await;

        match r {
            Ok(_) => panic!("Expected DeleteCharacterError::InventoryItems, got Ok"),
            Err(DeleteCharacterError::InventoryItems) => {}
            Err(e) => panic!("Expected DeleteCharacterError::InventoryItems, got {e:?}"),
        }
    }

    #[tokio::test]
    async fn cant_delete_character_with_gold() {
        let repository = TestAccountRepository::new().await;
        let account_id = repository
            .add_account_with_characters(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: "admin".to_string(),
                    password: "admin".to_string(),
                    ..Default::default()
                },
                vec![Character {
                    name: "charlist".to_string(),
                    coin: 1000,
                    ..Default::default()
                }],
            )
            .await;

        let r = DeleteCharacter {
            password: "admin".to_string(),
            slot: 0,
        }
        .handle_impl(account_id, repository.account_repository())
        .await;

        match r {
            Ok(_) => panic!("Expected DeleteCharacterError::Coin, got Ok"),
            Err(DeleteCharacterError::Coin) => {}
            Err(e) => panic!("Expected DeleteCharacterError::Coin, got {e:?}"),
        }
    }

    #[rstest]
    #[case(Evolution::Arch)]
    #[case(Evolution::Celestial)]
    #[case(Evolution::SubCelestial)]
    #[tokio::test]
    async fn cant_delete_character_with_evolution_above_mortal(#[case] evolution: Evolution) {
        let repository = TestAccountRepository::new().await;
        let account_id = repository
            .add_account_with_characters(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: "admin".to_string(),
                    password: "admin".to_string(),
                    ..Default::default()
                },
                vec![Character {
                    name: "charlist".to_string(),
                    evolution,
                    ..Default::default()
                }],
            )
            .await;

        let r = DeleteCharacter {
            password: "admin".to_string(),
            slot: 0,
        }
        .handle_impl(account_id, repository.account_repository())
        .await;

        match r {
            Ok(_) => panic!("Expected DeleteCharacterError::Evolution, got Ok"),
            Err(DeleteCharacterError::Evolution(_)) => {}
            Err(e) => panic!("Expected DeleteCharacterError::Evolution, got {e:?}"),
        }
    }

    #[tokio::test]
    async fn it_checks_the_password() {
        let repository = TestAccountRepository::new().await;
        let account_id = repository
            .add_account_with_characters(
                AccountCharlist {
                    identifier: Uuid::new_v4(),
                    username: "admin".to_string(),
                    password: "admin".to_string(),
                    ..Default::default()
                },
                vec![Character {
                    name: "charlist".to_string(),
                    ..Default::default()
                }],
            )
            .await;

        let r = DeleteCharacter {
            password: "admin2".to_string(),
            slot: 0,
        }
        .handle_impl(account_id, repository.account_repository())
        .await;

        match r {
            Ok(_) => panic!("Expected DeleteCharacterError::IncorrectPassword, got Ok"),
            Err(DeleteCharacterError::IncorrectPassword) => {}
            Err(e) => panic!("Expected DeleteCharacterError::IncorrectPassword, got {e:?}"),
        }
    }
}
