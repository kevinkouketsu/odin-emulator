use crate::map::EntityId;
use crate::packets::ToCreateMob;
use crate::session::{PacketSender, SessionError};
use crate::world::{Mob, World};
use odin_models::position::Position;
use odin_networking::{
    WritableResourceError,
    messages::{
        client::action::ActionRaw,
        server::{
            action::{
                ActionBroadcastData, ActionIllusionBroadcast, ActionStopBroadcast,
                ActionWalkBroadcast,
            },
            remove_mob::RemoveMob,
        },
    },
};

use crate::map::MapError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Walk,
    Illusion,
    Stop,
}

#[derive(Debug)]
pub struct Action {
    pub last_pos: Position,
    pub move_type: u32,
    pub move_speed: u32,
    pub command: [u8; 24],
    pub destiny: Position,
}

impl Action {
    pub fn handle<P: PacketSender>(
        &self,
        entity_id: EntityId,
        world: &mut World,
        sender: &P,
        action_type: ActionType,
    ) -> Result<(), ActionError> {
        if world.get_mob(entity_id).is_none() {
            return Err(ActionError::EntityNotFound);
        }

        let move_result = world.force_move_entity(entity_id, self.destiny)?;
        let data = ActionBroadcastData {
            mover_id: entity_id.id() as u16,
            last_pos: self.last_pos,
            move_type: self.move_type,
            move_speed: self.move_speed,
            route: ActionBroadcastData::route_from_bytes(self.command),
            destiny: move_result.to,
        };

        for spectator in move_result.stayed.iter().chain(move_result.entered.iter()) {
            match action_type {
                ActionType::Walk => sender.send_to(*spectator, ActionWalkBroadcast(data))?,
                ActionType::Illusion => {
                    sender.send_to(*spectator, ActionIllusionBroadcast(data))?
                }
                ActionType::Stop => sender.send_to(*spectator, ActionStopBroadcast(data))?,
            };
        }

        let mob = world.get_mob(entity_id).unwrap();
        let my_create_mob = mob.to_create_mob(move_result.to);
        for entered in &move_result.entered {
            let Some(spectator) = world.get_mob(*entered) else {
                continue;
            };

            let spectator_pos = world
                .map()
                .get_position(*entered)
                .expect("spectator from map must have a position");

            sender.send_to(*entered, my_create_mob.clone())?;
            sender.send_to(entity_id, spectator.to_create_mob(spectator_pos))?;
        }

        for exited in &move_result.exited {
            sender.send_to(
                *exited,
                RemoveMob {
                    mob_id: entity_id.id() as u16,
                    remove_type: 0,
                },
            )?;

            sender.send_to(
                entity_id,
                RemoveMob {
                    mob_id: exited.id() as u16,
                    remove_type: 0,
                },
            )?;
        }

        if let Some(Mob::Player(player)) = world.get_mob_mut(entity_id) {
            player.last_pos = move_result.to;
        }

        Ok(())
    }
}

impl TryFrom<ActionRaw> for Action {
    type Error = WritableResourceError;

    fn try_from(value: ActionRaw) -> Result<Self, Self::Error> {
        let destiny = Position {
            x: value.destiny.x,
            y: value.destiny.y,
        };
        if destiny.x == 0 || destiny.x >= 4096 || destiny.y == 0 || destiny.y >= 4096 {
            return Err(WritableResourceError::Generic(
                "Action destination out of bounds".to_string(),
            ));
        }

        Ok(Action {
            last_pos: Position {
                x: value.last_pos.x,
                y: value.last_pos.y,
            },
            move_type: value.move_type,
            move_speed: value.move_speed,
            command: value.command,
            destiny,
        })
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ActionError {
    #[error("Entity not found in world")]
    EntityNotFound,
    #[error(transparent)]
    Map(#[from] MapError),
    #[error(transparent)]
    PacketSender(#[from] SessionError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::tests::MockPacketSender;
    use crate::world::Player;
    use odin_models::{character::Character, position::Position, uuid::Uuid};
    use odin_networking::messages::ServerMessage;

    fn make_action(destiny: Position) -> Action {
        Action {
            last_pos: Position { x: 2100, y: 2100 },
            move_type: 0,
            move_speed: 3,
            command: [0; 24],
            destiny,
        }
    }

    fn add_player(world: &mut World, client_id: usize, pos: Position) -> EntityId {
        let entity_id = EntityId::Player(client_id);
        let player = Player::from_character(
            entity_id,
            Character {
                identifier: Uuid::new_v4(),
                name: format!("Player{}", client_id),
                last_pos: pos,
                ..Default::default()
            },
        );
        world.add_player(entity_id, player, pos).unwrap();
        entity_id
    }

    #[test]
    fn handle_moves_entity_on_map() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let entity_id = add_player(&mut world, 1, Position { x: 2100, y: 2100 });

        let action = make_action(Position { x: 2105, y: 2105 });
        action
            .handle(entity_id, &mut world, &sender, ActionType::Walk)
            .unwrap();

        assert_eq!(
            world.map().get_position(entity_id),
            Some(Position { x: 2105, y: 2105 })
        );
    }

    #[test]
    fn handle_broadcasts_to_stayed_spectators() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let mover = add_player(&mut world, 1, Position { x: 2100, y: 2100 });
        let spectator = add_player(&mut world, 2, Position { x: 2105, y: 2105 });

        let action = make_action(Position { x: 2102, y: 2102 });
        action
            .handle(mover, &mut world, &sender, ActionType::Walk)
            .unwrap();

        let messages = sender.messages_for(spectator);
        assert!(
            messages
                .iter()
                .any(|m| m.identifier == ServerMessage::Action),
            "stayed spectator should receive Action broadcast"
        );
    }

    #[test]
    fn handle_broadcasts_to_entered_spectators() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let mover = add_player(&mut world, 1, Position { x: 2100, y: 2100 });
        let far_spectator = add_player(&mut world, 2, Position { x: 2130, y: 2130 });

        // Move close to far_spectator
        let action = make_action(Position { x: 2120, y: 2120 });
        action
            .handle(mover, &mut world, &sender, ActionType::Walk)
            .unwrap();

        let messages = sender.messages_for(far_spectator);
        assert!(
            messages
                .iter()
                .any(|m| m.identifier == ServerMessage::Action),
            "entered spectator should receive Action broadcast"
        );
    }

    #[test]
    fn handle_exchanges_create_mob_with_entered() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let mover = add_player(&mut world, 1, Position { x: 2100, y: 2100 });
        let far_spectator = add_player(&mut world, 2, Position { x: 2130, y: 2130 });

        let action = make_action(Position { x: 2120, y: 2120 });
        action
            .handle(mover, &mut world, &sender, ActionType::Walk)
            .unwrap();

        let spectator_messages = sender.messages_for(far_spectator);
        assert!(
            spectator_messages
                .iter()
                .any(|m| m.identifier == ServerMessage::CreateMob),
            "entered spectator should receive CreateMob of mover"
        );

        let mover_messages = sender.messages_for(mover);
        assert!(
            mover_messages
                .iter()
                .any(|m| m.identifier == ServerMessage::CreateMob),
            "mover should receive CreateMob of entered spectator"
        );
    }

    #[test]
    fn handle_occupied_destination_uses_nearest_free() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let mover = add_player(&mut world, 1, Position { x: 2100, y: 2100 });
        let _blocker = add_player(&mut world, 2, Position { x: 2105, y: 2105 });

        let action = make_action(Position { x: 2105, y: 2105 });
        action
            .handle(mover, &mut world, &sender, ActionType::Walk)
            .unwrap();

        let pos = world.map().get_position(mover).unwrap();
        assert_ne!(
            pos,
            Position { x: 2105, y: 2105 },
            "should be placed nearby, not at the occupied position"
        );
        assert_ne!(
            pos,
            Position { x: 2100, y: 2100 },
            "should have moved from original position"
        );
    }

    #[test]
    fn handle_entity_not_found() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let entity_id = EntityId::Player(999);

        let action = make_action(Position { x: 2105, y: 2105 });
        let result = action.handle(entity_id, &mut world, &sender, ActionType::Walk);

        assert_eq!(result, Err(ActionError::EntityNotFound));
    }

    #[test]
    fn handle_sends_remove_mob_to_exited_spectators() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let mover = add_player(&mut world, 1, Position { x: 2100, y: 2100 });
        let nearby = add_player(&mut world, 2, Position { x: 2110, y: 2110 });

        // Move far away so nearby exits viewport
        let action = make_action(Position { x: 2500, y: 2500 });
        action
            .handle(mover, &mut world, &sender, ActionType::Walk)
            .unwrap();

        let nearby_messages = sender.messages_for(nearby);
        assert!(
            nearby_messages
                .iter()
                .any(|m| m.identifier == ServerMessage::RemoveMob),
            "exited spectator should receive RemoveMob of mover"
        );

        let mover_messages = sender.messages_for(mover);
        assert!(
            mover_messages
                .iter()
                .any(|m| m.identifier == ServerMessage::RemoveMob),
            "mover should receive RemoveMob of exited spectator"
        );
    }

    #[test]
    fn handle_updates_last_pos() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let entity_id = add_player(&mut world, 1, Position { x: 2100, y: 2100 });

        let action = make_action(Position { x: 2105, y: 2105 });
        action
            .handle(entity_id, &mut world, &sender, ActionType::Walk)
            .unwrap();

        let Some(Mob::Player(player)) = world.get_mob(entity_id) else {
            panic!("expected Player");
        };
        assert_eq!(player.last_pos, Position { x: 2105, y: 2105 });
    }
}
