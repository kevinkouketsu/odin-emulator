use crate::map::EntityId;
use crate::npc::Npc;
use crate::npc::mob_id_allocator::MobIdAllocator;
use crate::npc::movement::MovementState;
use crate::npc::spawn_group::{SpawnGroup, SpawnGroupConfig};
use crate::packets::ToCreateMob;
use crate::session::PacketSender;
use crate::world::World;
use rand::Rng;

pub struct SpawnManager {
    groups: Vec<SpawnGroup>,
    mob_id_allocator: MobIdAllocator,
}

impl SpawnManager {
    pub fn new(configs: Vec<SpawnGroupConfig>) -> Self {
        let groups = configs.into_iter().map(SpawnGroup::new).collect();
        Self {
            groups,
            mob_id_allocator: MobIdAllocator::new(),
        }
    }

    pub fn initial_spawn<P: PacketSender>(&mut self, world: &mut World, sender: &P) {
        for group_index in 0..self.groups.len() {
            while (self.groups[group_index].active_npcs.len() as u32)
                < self.groups[group_index].config.max_alive
            {
                self.spawn_group(group_index, world, sender);
            }
        }
    }

    pub fn tick<P: PacketSender>(&mut self, world: &mut World, sender: &P) {
        for group in &mut self.groups {
            group.active_npcs.retain(|id| world.entity_exists(*id));
            group.tick_respawn();
        }

        let indices: Vec<usize> = self
            .groups
            .iter()
            .enumerate()
            .filter_map(|(i, g)| if g.needs_spawn() { Some(i) } else { None })
            .collect();

        for group_index in indices {
            self.spawn_group(group_index, world, sender);
        }
    }

    pub fn reload(&mut self, configs: Vec<SpawnGroupConfig>) {
        self.groups = configs.into_iter().map(SpawnGroup::new).collect();
    }

    pub fn release_mob_id(&mut self, id: usize) {
        let _ = self.mob_id_allocator.release(id);
    }

    fn spawn_group<P: PacketSender>(&mut self, group_index: usize, world: &mut World, sender: &P) {
        let config = &self.groups[group_index].config;
        let max_alive = config.max_alive as usize;
        if self.groups[group_index].active_npcs.len() >= max_alive {
            return;
        }

        let mut rng = rand::thread_rng();
        let route_type = config.route_type.clone();
        let leader_template = config.leader_template.clone();
        let follower_template = config.follower_template.clone();
        let min_group = config.min_group;
        let max_group = config.max_group;
        let speed = leader_template.score.attack_run as u8;
        let respawn_ticks = config.respawn_ticks;

        let leader_waypoints = self.groups[group_index].resolve_waypoints(&mut rng);

        let Some(mob_id) = self.mob_id_allocator.allocate() else {
            return;
        };
        let entity_id = EntityId::Mob(mob_id);
        let behavior = route_type.to_behavior(leader_waypoints.clone());
        let movement = MovementState::new(behavior, speed);
        let mut npc = Npc::new(entity_id, leader_template.clone(), movement);
        npc.group_id = Some(group_index);
        npc.is_leader = true;

        let spawn_pos = leader_waypoints
            .first()
            .map_or(Default::default(), |w| w.position);

        match world.add_npc(entity_id, npc, spawn_pos) {
            Ok(result) => {
                self.groups[group_index].active_npcs.push(entity_id);
                let mob = world.get_mob(entity_id).unwrap();
                let create_mob = mob.to_create_mob(result.position);
                for spectator in &result.spectators {
                    let _ = sender.send_to(*spectator, create_mob.clone());
                }
            }
            Err(_) => {
                let _ = self.mob_id_allocator.release(mob_id);
                self.groups[group_index].respawn_countdown = respawn_ticks;
                return;
            }
        }

        if let Some(ref follower_tmpl) = follower_template {
            let follower_count = if min_group == max_group {
                min_group as usize
            } else if min_group > max_group {
                panic!(
                    "Invalid spawn group config: min_group {} is less than max_group {}. Mob: {}",
                    min_group, max_group, leader_template.name
                );
            } else {
                rng.gen_range(min_group..=max_group) as usize
            };
            let remaining = max_alive - self.groups[group_index].active_npcs.len();
            let follower_count = follower_count.min(remaining);
            let follower_speed = follower_tmpl.score.attack_run as u8;

            for fi in 0..follower_count {
                let follower_waypoints =
                    self.groups[group_index].resolve_follower_waypoints(&leader_waypoints, fi);

                let Some(f_mob_id) = self.mob_id_allocator.allocate() else {
                    break;
                };
                let f_entity_id = EntityId::Mob(f_mob_id);
                let f_behavior = route_type.to_behavior(follower_waypoints.clone());
                let f_movement = MovementState::new(f_behavior, follower_speed);
                let mut f_npc = Npc::new(f_entity_id, follower_tmpl.clone(), f_movement);
                f_npc.group_id = Some(group_index);
                f_npc.is_leader = false;
                f_npc.leader = Some(entity_id);

                let f_spawn_pos = follower_waypoints
                    .first()
                    .map_or(Default::default(), |w| w.position);

                match world.add_npc(f_entity_id, f_npc, f_spawn_pos) {
                    Ok(result) => {
                        self.groups[group_index].active_npcs.push(f_entity_id);
                        let mob = world.get_mob(f_entity_id).unwrap();
                        let create_mob = mob.to_create_mob(result.position);
                        for spectator in &result.spectators {
                            let _ = sender.send_to(*spectator, create_mob.clone());
                        }
                    }
                    Err(_) => {
                        let _ = self.mob_id_allocator.release(f_mob_id);
                    }
                }
            }
        }

        self.groups[group_index].respawn_countdown = respawn_ticks;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::tests::MockPacketSender;
    use crate::npc::spawn_group::{Formation, RouteType, WaypointConfig};
    use odin_models::npc_mob::NpcMob;
    use odin_models::position::Position;

    fn simple_config(max_alive: u32, respawn_ticks: u32) -> SpawnGroupConfig {
        SpawnGroupConfig {
            leader_template: NpcMob {
                name: "TestMob".to_string(),
                ..Default::default()
            },
            follower_template: None,
            min_group: 0,
            max_group: 0,
            formation: Formation::None,
            route_type: RouteType::Stationary,
            waypoints: vec![WaypointConfig {
                position: Position { x: 2100, y: 2100 },
                range: 0,
                wait_ticks: 0,
            }],
            respawn_ticks,
            max_alive,
        }
    }

    #[test]
    fn initial_spawn_adds_npcs_to_world() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let mut manager = SpawnManager::new(vec![simple_config(3, 0)]);

        manager.initial_spawn(&mut world, &sender);

        let npc_ids = world.npc_ids();
        assert_eq!(npc_ids.len(), 3);
        for id in &npc_ids {
            assert!(world.entity_exists(*id));
        }
    }

    #[test]
    fn tick_detects_dead_npcs() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let mut manager = SpawnManager::new(vec![simple_config(3, 100)]);

        manager.initial_spawn(&mut world, &sender);
        assert_eq!(manager.groups[0].active_npcs.len(), 3);

        let victim = manager.groups[0].active_npcs[0];
        world.remove_entity(victim).unwrap();

        manager.tick(&mut world, &sender);

        assert_eq!(manager.groups[0].active_npcs.len(), 2);
        assert!(!manager.groups[0].active_npcs.contains(&victim));
    }

    #[test]
    fn tick_respawns_after_countdown() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let mut manager = SpawnManager::new(vec![simple_config(1, 2)]);

        manager.initial_spawn(&mut world, &sender);
        assert_eq!(world.npc_ids().len(), 1);

        let original = manager.groups[0].active_npcs[0];
        world.remove_entity(original).unwrap();

        // tick 1: retain removes dead NPC, countdown 2 -> 1, no spawn
        manager.tick(&mut world, &sender);
        assert_eq!(world.npc_ids().len(), 0);

        // tick 2: countdown 1 -> 0, needs_spawn true, spawns
        manager.tick(&mut world, &sender);
        assert_eq!(world.npc_ids().len(), 1);
        assert!(world.entity_exists(manager.groups[0].active_npcs[0]));
    }

    #[test]
    fn release_mob_id_makes_id_reusable() {
        let mut manager = SpawnManager::new(vec![]);

        while manager.mob_id_allocator.allocate().is_some() {}
        assert!(manager.mob_id_allocator.allocate().is_none());

        manager.release_mob_id(42);
        let reused = manager.mob_id_allocator.allocate().unwrap();
        assert_eq!(reused, 42);
    }

    #[test]
    fn reload_replaces_configs() {
        let mut world = World::default();
        let sender = MockPacketSender::default();
        let mut manager = SpawnManager::new(vec![simple_config(2, 0)]);

        manager.initial_spawn(&mut world, &sender);
        let old_ids = world.npc_ids();
        assert_eq!(old_ids.len(), 2);

        let new_config = SpawnGroupConfig {
            leader_template: NpcMob {
                name: "NewMob".to_string(),
                ..Default::default()
            },
            ..simple_config(1, 0)
        };
        manager.reload(vec![new_config]);

        for id in &old_ids {
            assert!(world.entity_exists(*id));
        }
        assert_eq!(manager.groups.len(), 1);
        assert!(manager.groups[0].active_npcs.is_empty());
    }

    #[test]
    fn spawn_with_followers() {
        let mut world = World::default();
        let sender = MockPacketSender::default();

        let config = SpawnGroupConfig {
            follower_template: Some(NpcMob {
                name: "Follower".to_string(),
                ..Default::default()
            }),
            min_group: 2,
            max_group: 2,
            formation: Formation::Line,
            ..simple_config(3, 0)
        };
        let mut manager = SpawnManager::new(vec![config]);

        manager.initial_spawn(&mut world, &sender);

        let npc_ids = world.npc_ids();
        assert_eq!(npc_ids.len(), 3);
        assert_eq!(manager.groups[0].active_npcs.len(), 3);
    }

    #[test]
    fn spawn_followers_capped_by_max_alive() {
        let mut world = World::default();
        let sender = MockPacketSender::default();

        let config = SpawnGroupConfig {
            follower_template: Some(NpcMob {
                name: "Follower".to_string(),
                ..Default::default()
            }),
            min_group: 8,
            max_group: 8,
            formation: Formation::None,
            ..simple_config(5, 0)
        };
        let mut manager = SpawnManager::new(vec![config]);

        manager.initial_spawn(&mut world, &sender);

        assert_eq!(world.npc_ids().len(), 5);
        assert_eq!(manager.groups[0].active_npcs.len(), 5);
    }

    #[test]
    fn spawn_fills_to_max_alive() {
        let mut world = World::default();
        let sender = MockPacketSender::default();

        let config = SpawnGroupConfig {
            follower_template: Some(NpcMob {
                name: "Follower".to_string(),
                ..Default::default()
            }),
            min_group: 24,
            max_group: 24,
            formation: Formation::None,
            ..simple_config(25, 0)
        };
        let mut manager = SpawnManager::new(vec![config]);

        manager.initial_spawn(&mut world, &sender);

        assert_eq!(world.npc_ids().len(), 25);
        assert_eq!(manager.groups[0].active_npcs.len(), 25);
    }

    #[test]
    fn spawn_no_followers_when_max_alive_is_one() {
        let mut world = World::default();
        let sender = MockPacketSender::default();

        let config = SpawnGroupConfig {
            follower_template: Some(NpcMob {
                name: "Follower".to_string(),
                ..Default::default()
            }),
            min_group: 5,
            max_group: 5,
            formation: Formation::None,
            ..simple_config(1, 0)
        };
        let mut manager = SpawnManager::new(vec![config]);

        manager.initial_spawn(&mut world, &sender);

        assert_eq!(world.npc_ids().len(), 1);
        assert_eq!(manager.groups[0].active_npcs.len(), 1);
    }
}
