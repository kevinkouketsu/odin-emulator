use std::time::Duration;

use odin_networking::messages::server::{
    action::{ActionBroadcastData, ActionWalkBroadcast},
    remove_mob::RemoveMob,
};

use crate::map::{EntityId, MoveResult};
use crate::npc::movement::MovementAction;
use crate::npc::pathfinding::{MAX_PATH_STEPS, Pathfinder};
use crate::packets::ToCreateMob;
use crate::session::PacketSender;
use crate::world::{Mob, World};
use odin_models::{direction::Direction, position::Position};

pub const TICK_INTERVAL_MS: u64 = 500;
pub const TICK_STRIDE: usize = 6;

pub struct NpcTicker {
    tick_counter: u64,
    stride: usize,
}

impl NpcTicker {
    pub fn new() -> Self {
        Self {
            tick_counter: 0,
            stride: TICK_STRIDE,
        }
    }

    pub fn tick_counter(&self) -> u64 {
        self.tick_counter
    }

    pub fn tick_interval() -> Duration {
        Duration::from_millis(TICK_INTERVAL_MS)
    }

    pub fn tick<P: PacketSender>(
        &mut self,
        world: &mut World,
        pathfinder: &dyn Pathfinder,
        sender: &P,
    ) {
        let all_npc_ids = world.npc_ids();
        let start = (self.tick_counter as usize % self.stride).min(all_npc_ids.len());
        let ids_to_process: Vec<EntityId> = all_npc_ids
            .into_iter()
            .skip(start)
            .step_by(self.stride)
            .collect();

        for entity_id in ids_to_process {
            Self::process_npc(world, pathfinder, sender, entity_id);
        }

        self.tick_counter += 1;
    }

    fn process_npc<P: PacketSender>(
        world: &mut World,
        pathfinder: &dyn Pathfinder,
        sender: &P,
        entity_id: EntityId,
    ) {
        let Some(current_pos) = world.map().get_position(entity_id) else {
            return;
        };

        let Some(Mob::Npc(npc)) = world.get_mob_mut(entity_id) else {
            return;
        };
        let npc_name = npc.template.name.clone();
        let action = npc.movement.next_action();

        match action {
            MovementAction::Waiting => {}

            MovementAction::PickNextTarget => {
                {
                    let Some(Mob::Npc(npc)) = world.get_mob_mut(entity_id) else {
                        return;
                    };
                    npc.movement.pick_random_target(&mut rand::thread_rng());
                }
                let Some(Mob::Npc(npc)) = world.get_mob(entity_id) else {
                    return;
                };
                let target = npc.movement.current_waypoint_target();
                let speed = npc.movement.speed;

                let Some(target_pos) = target else {
                    log::debug!("[NPC {}] PickNextTarget: no target", npc_name);
                    return;
                };

                let max_steps = (speed as usize * 2).min(MAX_PATH_STEPS);
                let map = world.map();
                let is_passable = |pos: Position| map.can_step(pos, pos);
                let path = pathfinder.find_path(current_pos, target_pos, max_steps, &is_passable);

                if path.is_empty() {
                    log::debug!("[NPC {}] Already at target, advancing waypoint", npc_name);
                    let Some(Mob::Npc(npc)) = world.get_mob_mut(entity_id) else {
                        return;
                    };
                    npc.movement.advance_waypoint();
                    return;
                }

                let dest = path.iter().fold(current_pos, |pos, dir| {
                    pos.apply_direction(*dir).unwrap_or(pos)
                });

                log::debug!(
                    "[NPC {}] PickNextTarget: pos={} target={} dest={} speed={} steps={}",
                    npc_name,
                    current_pos,
                    target_pos,
                    dest,
                    speed,
                    path.len()
                );

                if let Ok(move_result) = world.force_move_entity(entity_id, dest) {
                    Self::broadcast_npc_walk(
                        world,
                        sender,
                        entity_id,
                        current_pos,
                        speed,
                        &path,
                        &move_result,
                    );
                }
            }

            MovementAction::StepForward { .. } => {}

            MovementAction::AdvanceWaypoint => {
                log::debug!("[NPC {}] AdvanceWaypoint at pos={}", npc_name, current_pos);
                let Some(Mob::Npc(npc)) = world.get_mob_mut(entity_id) else {
                    return;
                };
                npc.movement.advance_waypoint();
            }

            MovementAction::Despawn => {
                log::debug!("[NPC {}] Despawn at pos={}", npc_name, current_pos);
                if let Ok(remove_result) = world.remove_entity(entity_id) {
                    for spectator in &remove_result.spectators {
                        let _ = sender.send_to(
                            *spectator,
                            RemoveMob {
                                mob_id: entity_id.id() as u16,
                                remove_type: 0,
                            },
                        );
                    }
                }
            }
        }
    }

    fn broadcast_npc_walk<P: PacketSender>(
        world: &World,
        sender: &P,
        entity_id: EntityId,
        old_pos: Position,
        speed: u8,
        route: &[Direction],
        move_result: &MoveResult,
    ) {
        let data = ActionBroadcastData {
            mover_id: entity_id.id() as u16,
            last_pos: old_pos,
            move_type: 0,
            move_speed: speed as u32,
            route: ActionBroadcastData::route_from_directions(route),
            destiny: move_result.to,
        };

        // Entered: CreateMob at start position first, then Action so client sees the walk
        let create_mob = world.get_mob(entity_id).unwrap().to_create_mob(old_pos);
        for entered in &move_result.entered {
            let _ = sender.send_to(*entered, create_mob.clone());
            let _ = sender.send_to(*entered, ActionWalkBroadcast(data));
        }

        // Stayed: already know this mob, just send Action
        for spectator in &move_result.stayed {
            let _ = sender.send_to(*spectator, ActionWalkBroadcast(data));
        }

        for exited in &move_result.exited {
            let _ = sender.send_to(
                *exited,
                RemoveMob {
                    mob_id: entity_id.id() as u16,
                    remove_type: 0,
                },
            );
        }
    }
}

impl Default for NpcTicker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::tests::MockPacketSender;
    use crate::npc::Npc;
    use crate::npc::movement::{MovementBehavior, MovementState, Waypoint};
    use crate::npc::pathfinding::GreedyPathfinder;
    use crate::world::Player;
    use odin_models::character::Character;
    use odin_models::direction::Direction;
    use odin_models::npc_mob::NpcMob;
    use odin_models::position::Position;
    use odin_networking::messages::ServerMessage;

    fn pos(x: u16, y: u16) -> Position {
        Position { x, y }
    }

    fn make_npc(id: usize, movement: MovementState) -> (EntityId, Npc) {
        let entity_id = EntityId::Mob(id);
        let template = NpcMob {
            name: format!("Npc{id}"),
            ..Default::default()
        };
        (entity_id, Npc::new(entity_id, template, movement))
    }

    fn stationary_movement() -> MovementState {
        MovementState::new(MovementBehavior::Stationary, 1)
    }

    fn patrol_to(target: Position) -> MovementState {
        MovementState::new(
            MovementBehavior::Patrol {
                waypoints: vec![Waypoint {
                    position: target,
                    range: 0,
                    wait_ticks: 0,
                }],
                current_index: 0,
            },
            3,
        )
    }

    fn idle_movement(ticks: u32) -> MovementState {
        let mut state = MovementState::new(MovementBehavior::Stationary, 3);
        state.set_idle(ticks);
        state
    }

    fn add_player(world: &mut World, client_id: usize, position: Position) -> EntityId {
        let entity_id = EntityId::Player(client_id);
        let player = Player::from_character(
            entity_id,
            Character {
                name: format!("Player{client_id}"),
                ..Default::default()
            },
        );
        world.add_player(entity_id, player, position).unwrap();
        entity_id
    }

    #[test]
    fn tick_counter_increments() {
        let mut ticker = NpcTicker::new();
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        ticker.tick(&mut world, &pathfinder, &sender);
        ticker.tick(&mut world, &pathfinder, &sender);
        ticker.tick(&mut world, &pathfinder, &sender);

        assert_eq!(ticker.tick_counter(), 3);
    }

    #[test]
    fn tick_processes_stride_subset() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let mut ids = Vec::new();
        for i in 0..12 {
            let target = pos(2100, 2050);
            let (id, npc) = make_npc(1000 + i, patrol_to(target));
            let y = 2100 + (i as u16) * 2;
            world.add_npc(id, npc, pos(2100, y)).unwrap();
            ids.push(id);
        }

        let mut ticker = NpcTicker::new();
        ticker.tick(&mut world, &pathfinder, &sender);

        let npc_ids = world.npc_ids();
        let mut moved = 0;
        let mut stayed = 0;
        for (i, id) in ids.iter().enumerate() {
            if !npc_ids.contains(id) {
                continue;
            }
            let original_y = 2100 + (i as u16) * 2;
            let current = world.map().get_position(*id).unwrap();
            if current.y != original_y {
                moved += 1;
            } else {
                stayed += 1;
            }
        }

        assert_eq!(moved, 2);
        assert_eq!(stayed, 10);
    }

    #[test]
    fn tick_idle_npc_does_not_move() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let (id, npc) = make_npc(1000, idle_movement(10));
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();
        ticker.tick(&mut world, &pathfinder, &sender);

        assert_eq!(world.map().get_position(id), Some(pos(2100, 2100)));
    }

    #[test]
    fn tick_stationary_npc_never_moves() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let (id, npc) = make_npc(1000, stationary_movement());
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();
        for _ in 0..100 {
            ticker.tick(&mut world, &pathfinder, &sender);
        }

        assert_eq!(world.map().get_position(id), Some(pos(2100, 2100)));
    }

    #[test]
    fn tick_moves_npc_multiple_steps() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let (id, npc) = make_npc(1000, patrol_to(pos(2100, 2090)));
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();
        ticker.tick(&mut world, &pathfinder, &sender);

        let current = world.map().get_position(id).unwrap();
        assert!(
            current.y < 2100,
            "NPC should have moved north, got {:?}",
            current
        );
        let steps_taken = 2100 - current.y;
        assert!(
            steps_taken > 1,
            "NPC should take multiple steps per tick, took {}",
            steps_taken
        );
    }

    #[test]
    fn tick_broadcasts_walk_to_nearby_players() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let player_id = add_player(&mut world, 1, pos(2105, 2100));
        let (npc_id, npc) = make_npc(1000, patrol_to(pos(2100, 2095)));
        world.add_npc(npc_id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();
        ticker.tick(&mut world, &pathfinder, &sender);

        let messages = sender.messages_for(player_id);
        assert!(
            messages
                .iter()
                .any(|m| m.identifier == ServerMessage::Action),
            "nearby player should receive Action broadcast for NPC movement"
        );
    }

    #[test]
    fn tick_sends_remove_mob_on_exit_vision() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        // Player at edge of vision range from NPC
        let player_id = add_player(&mut world, 1, pos(2100, 2080));
        // NPC walks south, away from player
        let (npc_id, npc) = make_npc(1000, patrol_to(pos(2100, 2120)));
        world.add_npc(npc_id, npc, pos(2100, 2096)).unwrap();

        let mut ticker = NpcTicker::new();
        // Tick enough times for NPC to walk out of range
        for _ in 0..30 {
            ticker.tick(&mut world, &pathfinder, &sender);
        }

        let messages = sender.messages_for(player_id);
        assert!(
            messages
                .iter()
                .any(|m| m.identifier == ServerMessage::RemoveMob),
            "player should receive RemoveMob when NPC exits vision range"
        );
    }

    #[test]
    fn tick_patrol_disappear_removes_npc() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let waypoints = vec![Waypoint {
            position: pos(2100, 2100),
            range: 0,
            wait_ticks: 0,
        }];
        let mut movement = MovementState::new(
            MovementBehavior::PatrolDisappear {
                waypoints,
                current_index: 0,
            },
            3,
        );
        movement.set_arrived();

        let (id, npc) = make_npc(1000, movement);
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        assert!(world.entity_exists(id));

        let mut ticker = NpcTicker::new();
        ticker.tick(&mut world, &pathfinder, &sender);

        assert!(!world.entity_exists(id));
    }

    #[test]
    fn entered_player_receives_create_mob_at_start_pos_before_action() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        // NPC starts far from player, will walk into player's viewport
        let player_id = add_player(&mut world, 1, pos(2100, 2100));
        let (npc_id, npc) = make_npc(1000, patrol_to(pos(2100, 2100)));
        world.add_npc(npc_id, npc, pos(2100, 2120)).unwrap();

        let mut ticker = NpcTicker::new();
        for _ in 0..30 {
            ticker.tick(&mut world, &pathfinder, &sender);
        }

        let messages = sender.messages_for(player_id);
        let create_idx = messages
            .iter()
            .position(|m| m.identifier == ServerMessage::CreateMob);
        let action_idx = messages
            .iter()
            .position(|m| m.identifier == ServerMessage::Action);

        assert!(
            create_idx.is_some(),
            "player should receive CreateMob when NPC enters viewport"
        );
        assert!(
            action_idx.is_some(),
            "player should receive Action when NPC enters viewport"
        );
        assert!(
            create_idx.unwrap() < action_idx.unwrap(),
            "CreateMob (idx={}) must arrive before Action (idx={}) so client knows the mob before animating it",
            create_idx.unwrap(),
            action_idx.unwrap()
        );
    }

    #[test]
    fn full_lifecycle_patrol() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let waypoints = vec![
            Waypoint {
                position: pos(2100, 2100),
                range: 0,
                wait_ticks: 0,
            },
            Waypoint {
                position: pos(2103, 2100),
                range: 0,
                wait_ticks: 0,
            },
        ];
        let movement = MovementState::new(
            MovementBehavior::Patrol {
                waypoints,
                current_index: 0,
            },
            3,
        );
        let (id, npc) = make_npc(1000, movement);
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();

        // With stride 6 and 1 NPC, the NPC is processed every 6th tick.
        // Each processing: PickNextTarget → Walking → StepForward(x3) → AdvanceWaypoint
        for _ in 0..60 {
            ticker.tick(&mut world, &pathfinder, &sender);
        }

        let current = world.map().get_position(id).unwrap();
        assert!(
            current.chebyshev_distance(pos(2103, 2100)) <= 2
                || current.chebyshev_distance(pos(2100, 2100)) <= 2,
            "NPC should be near a waypoint, got {:?}",
            current
        );
        assert!(world.entity_exists(id));
    }

    #[test]
    fn full_lifecycle_patrol_disappear() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let waypoints = vec![Waypoint {
            position: pos(2102, 2100),
            range: 0,
            wait_ticks: 0,
        }];
        let movement = MovementState::new(
            MovementBehavior::PatrolDisappear {
                waypoints,
                current_index: 0,
            },
            3,
        );
        let (id, npc) = make_npc(1000, movement);
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();

        // With stride 6, need many ticks: PickNextTarget + Walk + Arrive + Despawn
        for _ in 0..60 {
            ticker.tick(&mut world, &pathfinder, &sender);
            if !world.entity_exists(id) {
                break;
            }
        }

        assert!(
            !world.entity_exists(id),
            "PatrolDisappear NPC should despawn after reaching final waypoint"
        );
    }

    #[test]
    fn npc_movement_broadcasts_to_players() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let player_id = add_player(&mut world, 1, pos(2105, 2100));

        let (npc_id, npc) = make_npc(1000, patrol_to(pos(2103, 2100)));
        world.add_npc(npc_id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();
        ticker.tick(&mut world, &pathfinder, &sender);

        let messages = sender.messages_for(player_id);
        let action_count = messages
            .iter()
            .filter(|m| m.identifier == ServerMessage::Action)
            .count();
        assert!(
            action_count >= 1,
            "player should receive at least 1 Action broadcast, got {}",
            action_count
        );
    }
}
