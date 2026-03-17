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
        let entity_id = EntityId::Player(client_id);
        let player = Player::from_character(entity_id, character);
        let insert_result = world.add_player(entity_id, player, position)?;
        world.recalculate_score(entity_id);

        let mob = world.get_mob(entity_id).unwrap();
        let Mob::Player(player) = mob;
        let position = insert_result.position;

        sender.send_to(client_id, player.to_character_login(position))?;

        let my_create_mob = mob.to_create_mob(position);
        sender.send_to(client_id, my_create_mob.clone())?;

        for spectator_entity in insert_result.spectators {
            let Some(spectator) = world.get_mob(spectator_entity) else {
                continue;
            };

            let spectator_pos = world
                .map()
                .get_position(spectator_entity)
                .expect("spectator from map must have a position");

            sender.send_to(client_id, spectator.to_create_mob(spectator_pos))?;
            sender.send_to(spectator_entity.id(), my_create_mob.clone())?;
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
        let mut world = World::default();
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
        let mut world = World::default();
        let client_id = EntityId::Player(1);
        let account_id =
            setup_account_with_character(&repository, Position { x: 2100, y: 2100 }).await;

        enter_world(0)
            .handle(
                account_id,
                client_id.id(),
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
        let mut world = World::default();

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
    async fn enter_world_spectator_receives_entering_player() {
        let repository = TestAccountRepository::new().await;
        let sender = MockPacketSender::default();
        let mut world = World::default();

        let spectator_id = EntityId::Player(10);
        let spectator_char = Player::from_character(
            spectator_id,
            Character {
                identifier: Uuid::new_v4(),
                name: "Spectator".to_string(),
                last_pos: Position { x: 2105, y: 2105 },
                ..Default::default()
            },
        );
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
        assert_eq!(spectator_messages.len(), 1);
        assert_eq!(
            spectator_messages[0].identifier,
            ServerMessage::CreateMob,
            "spectator should receive CreateMob of the entering player"
        );
    }

    #[tokio::test]
    async fn enter_world_player_receives_each_spectator() {
        let repository = TestAccountRepository::new().await;
        let sender = MockPacketSender::default();
        let mut world = World::default();

        for i in 0..3 {
            let spectator = Player::from_character(
                EntityId::Player(10 + i),
                Character {
                    identifier: Uuid::new_v4(),
                    name: format!("Spectator{}", i),
                    last_pos: Position {
                        x: 2100 + i as u16,
                        y: 2101,
                    },
                    ..Default::default()
                },
            );
            world
                .add_player(
                    EntityId::Player(10 + i),
                    spectator,
                    Position {
                        x: 2100 + i as u16,
                        y: 2101,
                    },
                )
                .unwrap();
        }

        let entity_id = EntityId::Player(1);
        let account_id =
            setup_account_with_character(&repository, Position { x: 2100, y: 2100 }).await;

        enter_world(0)
            .handle(
                account_id,
                entity_id.id(),
                repository.account_repository(),
                &sender,
                &mut world,
            )
            .await
            .unwrap();

        let messages = sender.messages_for(entity_id);
        // CharacterLogin + own CreateMob + 3 spectator CreateMobs
        assert_eq!(messages.len(), 5);
        assert_eq!(messages[0].identifier, ServerMessage::CharacterLogin);
        assert_eq!(messages[1].identifier, ServerMessage::CreateMob);
        assert_eq!(messages[2].identifier, ServerMessage::CreateMob);
        assert_eq!(messages[3].identifier, ServerMessage::CreateMob);
        assert_eq!(messages[4].identifier, ServerMessage::CreateMob);

        for spectator_offset in 0..3 {
            let spectator_messages = sender.messages_for(EntityId::Player(10 + spectator_offset));
            assert_eq!(
                spectator_messages.len(),
                1,
                "each spectator should receive exactly one CreateMob"
            );
            assert_eq!(spectator_messages[0].identifier, ServerMessage::CreateMob);
        }
    }

    #[tokio::test]
    async fn enter_world_at_occupied_position_finds_nearby() {
        let repository = TestAccountRepository::new().await;
        let sender = MockPacketSender::default();
        let mut world = World::default();

        let blocker_char = Player::from_character(
            EntityId::Player(10),
            Character {
                identifier: Uuid::new_v4(),
                name: "Blocker".to_string(),
                last_pos: Position { x: 2100, y: 2100 },
                ..Default::default()
            },
        );
        world
            .add_player(
                EntityId::Player(10),
                blocker_char,
                Position { x: 2100, y: 2100 },
            )
            .unwrap();

        let entity_id = EntityId::Player(1);
        let account_id =
            setup_account_with_character(&repository, Position { x: 2100, y: 2100 }).await;

        enter_world(0)
            .handle(
                account_id,
                entity_id.id(),
                repository.account_repository(),
                &sender,
                &mut world,
            )
            .await
            .unwrap();

        let pos = world.map().get_position(entity_id).unwrap();
        assert_ne!(
            pos,
            Position { x: 2100, y: 2100 },
            "should be placed nearby, not at the occupied position"
        );
    }
}
