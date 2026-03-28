use crate::map::EntityId;
use crate::session::{PacketSender, SessionError};
use crate::world::{Mob, Player, World};
use odin_models::MAX_AFFECT;
use odin_networking::messages::server::update_score::UpdateScore;

pub trait ToUpdateScore {
    fn to_update_score(&self) -> UpdateScore;
}

impl ToUpdateScore for Player {
    fn to_update_score(&self) -> UpdateScore {
        let guild_level = self.guild_level.map(|g| g.as_raw()).unwrap_or(0);

        UpdateScore {
            mob_id: self.entity_id.id() as u16,
            score: self.computed.score,
            critical: self.computed.critical.raw(),
            save_mana: self.computed.save_mana as i8,
            affect: [0u8; MAX_AFFECT], // TODO: affects system
            guild: self.guild.unwrap_or(0) as u16,
            guild_level,
            resist: [
                self.computed.resist[0] as i8,
                self.computed.resist[1] as i8,
                self.computed.resist[2] as i8,
                self.computed.resist[3] as i8,
            ],
            req_hp: self.computed.score.hp as i32,
            req_mp: self.computed.score.mp as i32,
            magic: self.computed.magic,
            rsv: 0,
            learned_skill: 0, // TODO: learned skills
        }
    }
}

pub trait BroadcastUpdateScore {
    fn broadcast_update_score<P: PacketSender>(
        &self,
        entity_id: EntityId,
        sender: &P,
    ) -> Result<(), SessionError>;
}

impl BroadcastUpdateScore for World {
    fn broadcast_update_score<P: PacketSender>(
        &self,
        entity_id: EntityId,
        sender: &P,
    ) -> Result<(), SessionError> {
        let Some(Mob::Player(player)) = self.get_mob(entity_id) else {
            return Ok(());
        };

        let Some(position) = self.map().get_position(entity_id) else {
            return Ok(());
        };

        let spectators = self.map().get_spectators(position, entity_id);
        sender.send_to(entity_id, player.to_update_score())?;

        for spectator in &spectators {
            sender.send_to(*spectator, player.to_update_score())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::tests::MockPacketSender;
    use odin_models::{character::Character, position::Position, uuid::Uuid};
    use odin_networking::messages::ServerMessage;

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
    fn broadcast_update_score_sends_to_self() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let entity_id = add_player(&mut world, 1, Position { x: 2100, y: 2100 });

        world.broadcast_update_score(entity_id, &sender).unwrap();

        let messages = sender.messages_for(entity_id);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].identifier, ServerMessage::UpdateScore);
    }

    #[test]
    fn broadcast_update_score_sends_to_spectators() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let entity_id = add_player(&mut world, 1, Position { x: 2100, y: 2100 });
        let spectator = add_player(&mut world, 2, Position { x: 2105, y: 2105 });

        world.broadcast_update_score(entity_id, &sender).unwrap();

        let spectator_messages = sender.messages_for(spectator);
        assert_eq!(spectator_messages.len(), 1);
        assert_eq!(spectator_messages[0].identifier, ServerMessage::UpdateScore);
    }

    #[test]
    fn broadcast_update_score_skips_far_players() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let entity_id = add_player(&mut world, 1, Position { x: 2100, y: 2100 });
        let far_player = add_player(&mut world, 2, Position { x: 2500, y: 2500 });

        world.broadcast_update_score(entity_id, &sender).unwrap();

        let far_messages = sender.messages_for(far_player);
        assert!(far_messages.is_empty());
    }

    #[test]
    fn broadcast_update_score_noop_for_missing_entity() {
        let world = World::default();
        let sender = MockPacketSender::default();

        let result = world.broadcast_update_score(EntityId::Player(999), &sender);

        assert!(result.is_ok());
    }
}
