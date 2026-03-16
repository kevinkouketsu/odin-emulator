use crate::map::EntityId;
use crate::packets::ToCharacterLogin;
use crate::session::{PacketSender, SessionError};
use crate::world::{Mob, Player, World};
use crate::{map::MapError, packets::ToCreateMob};
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
    pub async fn handle<A: AccountRepository, P: PacketSender>(
        &self,
        account_id: Uuid,
        client_id: usize,
        account_repository: A,
        sender: &P,
        world: &mut World,
    ) -> Result<(), EnterWorldError> {
        let character = account_repository
            .fetch_character(account_id, self.slot as usize)
            .await?
            .ok_or(EnterWorldError::CharacterNotFound)?;

        let position = character.last_pos;
        let player = Player::default();

        let insert_result = world.add_player(client_id, player, position)?;

        let entity_id = EntityId::Player(client_id);
        let mut mob = world.get_mob_mut(entity_id).unwrap();
        let Mob::Player(player) = &mut mob;
        let position = insert_result.position;

        sender.send_to(
            client_id,
            player.to_character_login(position, client_id as u16),
        )?;

        sender.send_to(client_id, mob.to_create_mob(entity_id, position))?;

        for spectator in insert_result.spectators {
            let Some(mob) = world.get_mob(spectator) else {
                continue;
            };

            let spectator_pos = world
                .map()
                .get_position(spectator)
                .expect("spectator from map must have a position");

            sender.send_to(spectator.id(), mob.to_create_mob(spectator, spectator_pos))?;
        }

        Ok(())
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

    #[error(transparent)]
    PacketSender(#[from] SessionError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::tests::{MockPacketSender, TestAccountRepository};
    use odin_models::{
        account_charlist::AccountCharlist, character::Character, position::Position, uuid::Uuid,
    };
    use odin_networking::messages::ServerMessage;

    fn enter_world(slot: u32) -> EnterWorld {
        EnterWorld {
            slot,
            force: false,
            secret_code: String::new(),
        }
    }

    async fn setup_account_with_character(
        repository: &TestAccountRepository,
        position: Position,
    ) -> Uuid {
        let account_id = Uuid::new_v4();
        repository
            .add_account(
                AccountCharlist {
                    identifier: account_id,
                    username: "player".to_string(),
                    password: "pass".to_string(),
                    ..Default::default()
                },
                None,
            )
            .await;

        repository
            .add_character(
                account_id,
                Character {
                    identifier: Uuid::new_v4(),
                    name: "TestChar".to_string(),
                    last_pos: position,
                    ..Default::default()
                },
            )
            .await;

        account_id
    }

    #[tokio::test]
    async fn enter_world_places_player_on_map() {
        let repository = TestAccountRepository::new().await;
        let sender = MockPacketSender::default();
        let mut world = World::new();
        let client_id = 1;
        let position = Position { x: 2100, y: 2100 };
        let account_id = setup_account_with_character(&repository, position).await;

        enter_world(0)
            .handle(
                account_id,
                client_id,
                repository.account_repository(),
                &sender,
                &mut world,
            )
            .await
            .unwrap();

        assert!(world.get_mob(EntityId::Player(client_id)).is_some());
        assert_eq!(
            world.map().get_position(EntityId::Player(client_id)),
            Some(position)
        );
    }

    #[tokio::test]
    async fn enter_world_sends_character_login_and_create_mob_to_player() {
        let repository = TestAccountRepository::new().await;
        let sender = MockPacketSender::default();
        let mut world = World::new();
        let client_id = 1;
        let account_id =
            setup_account_with_character(&repository, Position { x: 2100, y: 2100 }).await;

        enter_world(0)
            .handle(
                account_id,
                client_id,
                repository.account_repository(),
                &sender,
                &mut world,
            )
            .await
            .unwrap();

        let messages = sender.messages_for(client_id);
        assert_eq!(
            messages.len(),
            2,
            "should receive CharacterLogin + CreateMob"
        );
        assert_eq!(messages[0].identifier, ServerMessage::CharacterLogin);
        assert_eq!(messages[1].identifier, ServerMessage::CreateMob);
    }

    #[tokio::test]
    async fn enter_world_character_not_found() {
        let repository = TestAccountRepository::new().await;
        let sender = MockPacketSender::default();
        let mut world = World::new();

        let account_id = Uuid::new_v4();
        repository
            .add_account(
                AccountCharlist {
                    identifier: account_id,
                    username: "player".to_string(),
                    password: "pass".to_string(),
                    ..Default::default()
                },
                None,
            )
            .await;

        let result = enter_world(0)
            .handle(
                account_id,
                1,
                repository.account_repository(),
                &sender,
                &mut world,
            )
            .await;

        assert!(matches!(result, Err(EnterWorldError::CharacterNotFound)));
    }

    #[tokio::test]
    async fn enter_world_sends_create_mob_to_spectators() {
        let repository = TestAccountRepository::new().await;
        let sender = MockPacketSender::default();
        let mut world = World::new();

        let spectator_id = 10;
        let spectator_char = Character {
            identifier: Uuid::new_v4(),
            name: "Spectator".to_string(),
            last_pos: Position { x: 2105, y: 2105 },
            ..Default::default()
        };
        world
            .add_player(spectator_id, spectator_char, Position { x: 2105, y: 2105 })
            .unwrap();

        let client_id = 1;
        let account_id =
            setup_account_with_character(&repository, Position { x: 2100, y: 2100 }).await;

        enter_world(0)
            .handle(
                account_id,
                client_id,
                repository.account_repository(),
                &sender,
                &mut world,
            )
            .await
            .unwrap();

        let spectator_messages = sender.messages_for(spectator_id);
        assert_eq!(
            spectator_messages.len(),
            1,
            "spectator should receive CreateMob for the entering player"
        );
    }

    #[tokio::test]
    async fn enter_world_at_occupied_position_finds_nearby() {
        let repository = TestAccountRepository::new().await;
        let sender = MockPacketSender::default();
        let mut world = World::new();

        let blocker_char = Character {
            identifier: Uuid::new_v4(),
            name: "Blocker".to_string(),
            last_pos: Position { x: 2100, y: 2100 },
            ..Default::default()
        };
        world
            .add_player(10, blocker_char, Position { x: 2100, y: 2100 })
            .unwrap();

        let client_id = 1;
        let account_id =
            setup_account_with_character(&repository, Position { x: 2100, y: 2100 }).await;

        enter_world(0)
            .handle(
                account_id,
                client_id,
                repository.account_repository(),
                &sender,
                &mut world,
            )
            .await
            .unwrap();

        let pos = world
            .map()
            .get_position(EntityId::Player(client_id))
            .unwrap();
        assert_ne!(
            pos,
            Position { x: 2100, y: 2100 },
            "should be placed nearby, not at the occupied position"
        );
    }
}
