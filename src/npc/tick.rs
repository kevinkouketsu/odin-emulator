use std::time::Duration;

use odin_networking::messages::server::{
    action::{ActionBroadcastData, ActionWalkBroadcast},
    remove_mob::RemoveMob,
};

use crate::map::{EntityId, MoveResult};
use crate::npc::movement::{MovementBehavior, TickAction};
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
    ) -> Vec<usize> {
        let all_npc_ids = world.npc_ids();
        let start = (self.tick_counter as usize % self.stride).min(all_npc_ids.len());
        let ids_to_process = all_npc_ids.into_iter().skip(start).step_by(self.stride);

        let mut despawned = Vec::new();
        for entity_id in ids_to_process {
            if let Some(id) = Self::process_npc(world, pathfinder, sender, entity_id) {
                despawned.push(id);
            }
        }

        self.tick_counter += 1;
        despawned
    }

    fn process_npc<P: PacketSender>(
        world: &mut World,
        pathfinder: &dyn Pathfinder,
        sender: &P,
        entity_id: EntityId,
    ) -> Option<usize> {
        let current_pos = world.map().get_position(entity_id)?;

        // Pick random target if needed (must happen before computing at_target)
        {
            let Some(Mob::Npc(npc)) = world.get_mob_mut(entity_id) else {
                return None;
            };
            if matches!(
                npc.movement.behavior,
                MovementBehavior::Random {
                    current_target: None,
                    ..
                }
            ) {
                npc.movement.pick_random_target(&mut rand::thread_rng());
            }
        }

        // Read state for tick decision
        let (npc_name, target, speed, phase_before) = {
            let Some(Mob::Npc(npc)) = world.get_mob(entity_id) else {
                return None;
            };
            let name = npc.template.name.clone();
            let target = npc.movement.current_waypoint_target();
            let speed = npc.movement.speed;
            let phase = format!("{:?}", npc.movement.phase);
            (name, target, speed, phase)
        };

        // At target if: exactly there, OR adjacent and target cell is occupied
        let at_target = target.is_none_or(|t| {
            current_pos == t
                || (current_pos.chebyshev_distance(t) <= 1
                    && world.map().is_occupied_by_other(t, entity_id))
        });

        // Get tick action from movement state machine
        let action = {
            let Some(Mob::Npc(npc)) = world.get_mob_mut(entity_id) else {
                return None;
            };
            npc.movement.tick(at_target)
        };

        match action {
            TickAction::Wait => {
                log::trace!(
                    "[NPC {:?} {}] Wait (phase={}, pos={}, target={:?})",
                    entity_id,
                    npc_name,
                    phase_before,
                    current_pos,
                    target,
                );
                None
            }

            TickAction::Despawn => {
                log::trace!(
                    "[NPC {:?} {}] Despawn at pos={}",
                    entity_id,
                    npc_name,
                    current_pos,
                );
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
                    return Some(entity_id.id());
                }
                None
            }

            TickAction::Move => {
                let target_pos = target?;
                let max_steps = (speed as usize).min(MAX_PATH_STEPS);
                let map = world.map();
                let is_passable = |pos: Position| map.is_terrain_passable(pos);
                let path = pathfinder.find_path(current_pos, target_pos, max_steps, &is_passable);

                if path.is_empty() {
                    log::trace!(
                        "[NPC {:?} {}] Move: empty path from {} to {} (already there?)",
                        entity_id,
                        npc_name,
                        current_pos,
                        target_pos,
                    );
                    return None;
                }

                let intended_dest = path.iter().fold(current_pos, |pos, dir| {
                    pos.apply_direction(*dir).unwrap_or(pos)
                });

                // Check occupancy and re-route if needed
                let (final_path, final_dest) =
                    if world.map().is_occupied_by_other(intended_dest, entity_id) {
                        log::trace!(
                            "[NPC {:?} {}] dest {} occupied, re-routing",
                            entity_id,
                            npc_name,
                            intended_dest,
                        );
                        match world.map().find_nearest_free(intended_dest) {
                            Some(free_pos) => {
                                let map = world.map();
                                let is_passable = |pos: Position| map.is_terrain_passable(pos);
                                let new_path = pathfinder.find_path(
                                    current_pos,
                                    free_pos,
                                    max_steps,
                                    &is_passable,
                                );
                                if new_path.is_empty() {
                                    return None;
                                }
                                let new_dest = new_path.iter().fold(current_pos, |pos, dir| {
                                    pos.apply_direction(*dir).unwrap_or(pos)
                                });
                                (new_path, new_dest)
                            }
                            None => return None,
                        }
                    } else {
                        (path, intended_dest)
                    };

                let route_bytes: Vec<u8> = final_path.iter().map(|d| d.to_route_byte()).collect();

                log::trace!(
                    "[NPC {:?} {}] Move: pos={} target={} dest={} speed={} steps={} route={:?}",
                    entity_id,
                    npc_name,
                    current_pos,
                    target_pos,
                    final_dest,
                    speed,
                    final_path.len(),
                    route_bytes,
                );

                // Strict move (no deflection)
                if let Ok(move_result) = world.move_entity(entity_id, final_dest) {
                    Self::broadcast_npc_walk(
                        world,
                        sender,
                        entity_id,
                        current_pos,
                        speed,
                        &final_path,
                        &move_result,
                    );
                }
                None
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

        let create_mob = world.get_mob(entity_id).unwrap().to_create_mob(old_pos);
        for entered in &move_result.entered {
            let _ = sender.send_to(*entered, create_mob.clone());
            let _ = sender.send_to(*entered, ActionWalkBroadcast(data));
        }

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
            MovementBehavior::Loop {
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
    fn tick_npc_at_target_waits() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let movement = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![
                    Waypoint {
                        position: pos(2100, 2100),
                        range: 0,
                        wait_ticks: 10,
                    },
                    Waypoint {
                        position: pos(2110, 2100),
                        range: 0,
                        wait_ticks: 0,
                    },
                ],
                current_index: 0,
            },
            3,
        );
        let (id, npc) = make_npc(1000, movement);
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();
        // NPC starts at waypoint 0 with wait=10, should not move during waiting
        for _ in 0..6 {
            ticker.tick(&mut world, &pathfinder, &sender);
        }

        assert_eq!(world.map().get_position(id), Some(pos(2100, 2100)));
    }

    #[test]
    fn tick_npc_moves_toward_target() {
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
    }

    #[test]
    fn tick_npc_moves_multiple_steps() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let (id, npc) = make_npc(1000, patrol_to(pos(2100, 2090)));
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();
        ticker.tick(&mut world, &pathfinder, &sender);

        let current = world.map().get_position(id).unwrap();
        let steps_taken = 2100 - current.y;
        assert!(
            steps_taken > 1,
            "NPC should take multiple steps per tick, took {}",
            steps_taken
        );
    }

    #[test]
    fn tick_dest_occupied_reroutes() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        // Target position occupied by another NPC
        let (blocker_id, blocker) = make_npc(999, stationary_movement());
        world.add_npc(blocker_id, blocker, pos(2103, 2100)).unwrap();

        let movement = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![Waypoint {
                    position: pos(2103, 2100),
                    range: 0,
                    wait_ticks: 0,
                }],
                current_index: 0,
            },
            3,
        );
        let (id, npc) = make_npc(1000, movement);
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();
        // Tick enough times to ensure both NPCs are processed (stride=6, 2 NPCs)
        for _ in 0..6 {
            ticker.tick(&mut world, &pathfinder, &sender);
        }

        let npc_pos = world.map().get_position(id).unwrap();
        let blocker_pos = world.map().get_position(blocker_id).unwrap();

        // NPC should have moved but NOT to the blocker's position
        assert_ne!(npc_pos, pos(2100, 2100), "NPC should have moved");
        assert_ne!(npc_pos, blocker_pos, "NPC should not overlap blocker");
    }

    #[test]
    fn tick_dest_occupied_skips_if_stuck() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let start = pos(2100, 2100);
        let target = pos(2101, 2100);

        let (id, npc) = make_npc(
            1000,
            MovementState::new(
                MovementBehavior::Loop {
                    waypoints: vec![Waypoint {
                        position: target,
                        range: 0,
                        wait_ticks: 0,
                    }],
                    current_index: 0,
                },
                3,
            ),
        );

        // Fill all cells around target so there's no free cell
        let mut id_counter = 2000;
        for dy in -5i32..=5 {
            for dx in -5i32..=5 {
                let p = pos((2101 + dx) as u16, (2100 + dy) as u16);
                if p == start {
                    continue;
                }
                let (bid, bnpc) = make_npc(id_counter, stationary_movement());
                let _ = world.add_npc(bid, bnpc, p);
                id_counter += 1;
            }
        }
        world.add_npc(id, npc, start).unwrap();

        let mut ticker = NpcTicker::new();
        ticker.tick(&mut world, &pathfinder, &sender);

        // NPC should stay at start since no free cell
        assert_eq!(world.map().get_position(id), Some(start));
    }

    #[test]
    fn tick_walk_and_despawn_removes_npc() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let movement = MovementState::new(
            MovementBehavior::WalkAndDespawn {
                waypoints: vec![Waypoint {
                    position: pos(2102, 2100),
                    range: 0,
                    wait_ticks: 0,
                }],
                current_index: 0,
            },
            3,
        );
        let (id, npc) = make_npc(1000, movement);
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();
        for _ in 0..60 {
            ticker.tick(&mut world, &pathfinder, &sender);
            if !world.entity_exists(id) {
                break;
            }
        }

        assert!(
            !world.entity_exists(id),
            "WalkAndDespawn NPC should despawn after reaching final waypoint"
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

        let player_id = add_player(&mut world, 1, pos(2100, 2080));
        let (npc_id, npc) = make_npc(1000, patrol_to(pos(2100, 2120)));
        world.add_npc(npc_id, npc, pos(2100, 2096)).unwrap();

        let mut ticker = NpcTicker::new();
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
    fn tick_entered_player_receives_create_mob_before_action() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

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

        assert!(create_idx.is_some(), "player should receive CreateMob");
        assert!(action_idx.is_some(), "player should receive Action");
        assert!(
            create_idx.unwrap() < action_idx.unwrap(),
            "CreateMob must arrive before Action"
        );
    }

    #[test]
    fn tick_full_loop_lifecycle() {
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
            MovementBehavior::Loop {
                waypoints,
                current_index: 0,
            },
            3,
        );
        let (id, npc) = make_npc(1000, movement);
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();
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
    fn tick_full_walk_and_despawn_lifecycle() {
        let mut world = World::default();
        let pathfinder = GreedyPathfinder;
        let sender = MockPacketSender::default();

        let waypoints = vec![Waypoint {
            position: pos(2102, 2100),
            range: 0,
            wait_ticks: 0,
        }];
        let movement = MovementState::new(
            MovementBehavior::WalkAndDespawn {
                waypoints,
                current_index: 0,
            },
            3,
        );
        let (id, npc) = make_npc(1000, movement);
        world.add_npc(id, npc, pos(2100, 2100)).unwrap();

        let mut ticker = NpcTicker::new();
        for _ in 0..60 {
            ticker.tick(&mut world, &pathfinder, &sender);
            if !world.entity_exists(id) {
                break;
            }
        }

        assert!(
            !world.entity_exists(id),
            "WalkAndDespawn NPC should despawn after reaching final waypoint"
        );
    }

    #[test]
    fn tick_stride_processes_subset() {
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
    fn tick_npc_movement_broadcasts_to_players() {
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
