use crate::map::EntityId;
use crate::npc::movement::{MovementBehavior, Waypoint};
use odin_models::npc_mob::NpcMob;
use odin_models::position::Position;
use rand::Rng;

pub const MAX_FOLLOWERS: usize = 12;
pub const MAX_WAYPOINTS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnMode {
    Auto { respawn_ticks: u32 },
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnGroupId {
    pub name: String,
    pub index: Option<u32>,
}

const FORMATION_NONE: [(i8, i8); MAX_FOLLOWERS] = [
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
    (0, 0),
];

const FORMATION_LINE: [(i8, i8); MAX_FOLLOWERS] = [
    (1, 0),
    (-1, 0),
    (2, 0),
    (-2, 0),
    (3, 0),
    (-3, 0),
    (4, 0),
    (-4, 0),
    (5, 0),
    (-5, 0),
    (6, 0),
    (-6, 0),
];

const FORMATION_WEDGE: [(i8, i8); MAX_FOLLOWERS] = [
    (1, 1),
    (-1, 1),
    (1, -1),
    (-1, -1),
    (2, 2),
    (-2, 2),
    (2, -2),
    (-2, -2),
    (3, 3),
    (-3, 3),
    (3, -3),
    (-3, -3),
];

const FORMATION_RING: [(i8, i8); MAX_FOLLOWERS] = [
    (1, 0),
    (0, 1),
    (-1, 0),
    (0, -1),
    (1, 1),
    (-1, 1),
    (1, -1),
    (-1, -1),
    (2, 0),
    (0, 2),
    (-2, 0),
    (0, -2),
];

const FORMATION_CROSS: [(i8, i8); MAX_FOLLOWERS] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (2, 0),
    (-2, 0),
    (0, 2),
    (0, -2),
    (3, 0),
    (-3, 0),
    (0, 3),
    (0, -3),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Formation {
    None,
    Line,
    Wedge,
    Ring,
    Cross,
}

impl Formation {
    pub fn offsets(&self) -> &[(i8, i8)] {
        match self {
            Formation::None => &FORMATION_NONE,
            Formation::Line => &FORMATION_LINE,
            Formation::Wedge => &FORMATION_WEDGE,
            Formation::Ring => &FORMATION_RING,
            Formation::Cross => &FORMATION_CROSS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteType {
    Stationary,
    Random { radius: u16 },
    WalkToEnd,
    WalkAndDespawn,
    PingPong,
    PingPongDespawn,
    Loop,
}

impl RouteType {
    pub fn to_behavior(&self, waypoints: Vec<Waypoint>) -> MovementBehavior {
        match self {
            RouteType::Stationary => MovementBehavior::Stationary,
            RouteType::Random { radius } => {
                let origin = waypoints
                    .first()
                    .map_or(Position::default(), |w| w.position);
                MovementBehavior::Random {
                    origin,
                    radius: *radius,
                    current_target: None,
                }
            }
            RouteType::Loop => MovementBehavior::Loop {
                waypoints,
                current_index: 0,
            },
            RouteType::WalkAndDespawn => MovementBehavior::WalkAndDespawn {
                waypoints,
                current_index: 0,
            },
            RouteType::WalkToEnd => MovementBehavior::WalkToEnd {
                waypoints,
                current_index: 0,
            },
            RouteType::PingPong => MovementBehavior::PingPong {
                waypoints,
                current_index: 0,
                forward: true,
            },
            RouteType::PingPongDespawn => MovementBehavior::PingPongDespawn {
                waypoints,
                current_index: 0,
                forward: true,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaypointConfig {
    pub position: Position,
    pub range: u16,
    pub wait_ticks: u32,
}

#[derive(Debug)]
pub struct SpawnGroupConfig {
    pub id: Option<SpawnGroupId>,
    pub leader_template: NpcMob,
    pub follower_template: Option<NpcMob>,
    pub min_group: u32,
    pub max_group: u32,
    pub formation: Formation,
    pub route_type: RouteType,
    pub waypoints: Vec<WaypointConfig>,
    pub spawn_mode: SpawnMode,
    pub max_alive: u32,
}

pub struct SpawnGroup {
    pub config: SpawnGroupConfig,
    pub active_npcs: Vec<EntityId>,
    pub respawn_countdown: u32,
}

impl SpawnGroup {
    pub fn new(config: SpawnGroupConfig) -> Self {
        let respawn_countdown = match &config.spawn_mode {
            SpawnMode::Auto { respawn_ticks } => *respawn_ticks,
            SpawnMode::Manual => 0,
        };
        Self {
            config,
            active_npcs: Vec::new(),
            respawn_countdown,
        }
    }

    pub fn needs_spawn(&self) -> bool {
        if matches!(self.config.spawn_mode, SpawnMode::Manual) {
            return false;
        }
        (self.active_npcs.len() as u32) < self.config.max_alive && self.respawn_countdown == 0
    }

    pub fn tick_respawn(&mut self) {
        if self.respawn_countdown > 0 {
            self.respawn_countdown -= 1;
        }
    }

    pub fn resolve_waypoints(&self, rng: &mut impl Rng) -> Vec<Waypoint> {
        self.config
            .waypoints
            .iter()
            .map(|wc| {
                let position = if wc.range == 0 {
                    wc.position
                } else {
                    let dx = rng.gen_range(-(wc.range as i32)..=(wc.range as i32));
                    let dy = rng.gen_range(-(wc.range as i32)..=(wc.range as i32));
                    wc.position.offset(dx, dy).unwrap_or(wc.position)
                };
                Waypoint {
                    position,
                    range: wc.range,
                    wait_ticks: wc.wait_ticks,
                }
            })
            .collect()
    }

    pub fn resolve_follower_waypoints(
        &self,
        leader_waypoints: &[Waypoint],
        follower_index: usize,
    ) -> Vec<Waypoint> {
        let offsets = self.config.formation.offsets();
        let (dx, dy) = offsets.get(follower_index).copied().unwrap_or((0, 0));

        leader_waypoints
            .iter()
            .map(|wp| {
                let position = wp
                    .position
                    .offset(dx as i32, dy as i32)
                    .unwrap_or(wp.position);
                Waypoint {
                    position,
                    range: wp.range,
                    wait_ticks: wp.wait_ticks,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn pos(x: u16, y: u16) -> Position {
        Position { x, y }
    }

    fn default_config() -> SpawnGroupConfig {
        SpawnGroupConfig {
            id: None,
            leader_template: NpcMob::default(),
            follower_template: None,
            min_group: 0,
            max_group: 0,
            formation: Formation::None,
            route_type: RouteType::Stationary,
            waypoints: Vec::new(),
            spawn_mode: SpawnMode::Auto { respawn_ticks: 0 },
            max_alive: 1,
        }
    }

    #[test]
    fn formation_none_all_zero_offsets() {
        let offsets = Formation::None.offsets();
        assert_eq!(offsets.len(), MAX_FOLLOWERS);
        for &(dx, dy) in offsets {
            assert_eq!((dx, dy), (0, 0));
        }
    }

    #[test]
    fn formation_line_offsets() {
        let offsets = Formation::Line.offsets();
        assert_eq!(offsets[0], (1, 0));
        assert_eq!(offsets[1], (-1, 0));
    }

    #[test]
    fn formation_ring_offsets() {
        let offsets = Formation::Ring.offsets();
        assert_eq!(offsets[0], (1, 0));
        assert_eq!(offsets[1], (0, 1));
        assert_eq!(offsets[2], (-1, 0));
        assert_eq!(offsets[3], (0, -1));
    }

    #[test]
    fn generate_positions_leader_only() {
        let config = SpawnGroupConfig {
            waypoints: vec![
                WaypointConfig {
                    position: pos(100, 100),
                    range: 0,
                    wait_ticks: 5,
                },
                WaypointConfig {
                    position: pos(200, 200),
                    range: 0,
                    wait_ticks: 10,
                },
            ],
            ..default_config()
        };
        let group = SpawnGroup::new(config);
        let mut rng = SmallRng::seed_from_u64(42);
        let waypoints = group.resolve_waypoints(&mut rng);
        assert_eq!(waypoints.len(), 2);
        assert_eq!(waypoints[0].position, pos(100, 100));
        assert_eq!(waypoints[0].wait_ticks, 5);
        assert_eq!(waypoints[1].position, pos(200, 200));
        assert_eq!(waypoints[1].wait_ticks, 10);
    }

    #[test]
    fn needs_spawn_when_all_dead() {
        let group = SpawnGroup::new(SpawnGroupConfig {
            max_alive: 3,
            ..default_config()
        });
        assert!(group.needs_spawn());
    }

    #[test]
    fn needs_spawn_at_max() {
        let mut group = SpawnGroup::new(SpawnGroupConfig {
            max_alive: 2,
            ..default_config()
        });
        group.active_npcs.push(EntityId::Mob(0));
        group.active_npcs.push(EntityId::Mob(1));
        assert!(!group.needs_spawn());
    }

    #[test]
    fn needs_spawn_countdown_not_ready() {
        let group = SpawnGroup::new(SpawnGroupConfig {
            max_alive: 5,
            spawn_mode: SpawnMode::Auto { respawn_ticks: 10 },
            ..default_config()
        });
        assert!(!group.needs_spawn());
    }

    #[test]
    fn needs_spawn_manual_always_false() {
        let group = SpawnGroup::new(SpawnGroupConfig {
            max_alive: 5,
            spawn_mode: SpawnMode::Manual,
            ..default_config()
        });
        assert!(!group.needs_spawn());
    }

    #[test]
    fn tick_respawn_decrements() {
        let mut group = SpawnGroup::new(SpawnGroupConfig {
            spawn_mode: SpawnMode::Auto { respawn_ticks: 3 },
            ..default_config()
        });
        assert_eq!(group.respawn_countdown, 3);
        group.tick_respawn();
        assert_eq!(group.respawn_countdown, 2);
    }

    #[test]
    fn resolve_waypoints_applies_range() {
        let config = SpawnGroupConfig {
            waypoints: vec![WaypointConfig {
                position: pos(100, 100),
                range: 5,
                wait_ticks: 0,
            }],
            ..default_config()
        };
        let group = SpawnGroup::new(config);
        let mut rng = SmallRng::seed_from_u64(42);
        let waypoints = group.resolve_waypoints(&mut rng);
        assert_eq!(waypoints.len(), 1);
        assert_ne!(waypoints[0].position, pos(100, 100));
        let dx = (waypoints[0].position.x as i32 - 100).abs();
        let dy = (waypoints[0].position.y as i32 - 100).abs();
        assert!(dx <= 5);
        assert!(dy <= 5);
    }

    #[test]
    fn resolve_waypoints_zero_range() {
        let config = SpawnGroupConfig {
            waypoints: vec![WaypointConfig {
                position: pos(100, 100),
                range: 0,
                wait_ticks: 0,
            }],
            ..default_config()
        };
        let group = SpawnGroup::new(config);
        let mut rng = SmallRng::seed_from_u64(42);
        let waypoints = group.resolve_waypoints(&mut rng);
        assert_eq!(waypoints[0].position, pos(100, 100));
    }

    #[test]
    fn resolve_follower_waypoints_applies_formation() {
        let config = SpawnGroupConfig {
            formation: Formation::Line,
            waypoints: vec![WaypointConfig {
                position: pos(100, 100),
                range: 0,
                wait_ticks: 3,
            }],
            ..default_config()
        };
        let group = SpawnGroup::new(config);
        let leader_waypoints = vec![Waypoint {
            position: pos(100, 100),
            range: 0,
            wait_ticks: 3,
        }];
        let follower_wps = group.resolve_follower_waypoints(&leader_waypoints, 0);
        assert_eq!(follower_wps.len(), 1);
        assert_eq!(follower_wps[0].position, pos(101, 100));
        assert_eq!(follower_wps[0].wait_ticks, 3);

        let follower_wps = group.resolve_follower_waypoints(&leader_waypoints, 1);
        assert_eq!(follower_wps[0].position, pos(99, 100));
    }

    #[test]
    fn route_type_to_behavior_stationary() {
        let behavior = RouteType::Stationary.to_behavior(Vec::new());
        assert_eq!(behavior, MovementBehavior::Stationary);
    }

    #[test]
    fn route_type_to_behavior_loop() {
        let waypoints = vec![
            Waypoint {
                position: pos(100, 100),
                range: 0,
                wait_ticks: 5,
            },
            Waypoint {
                position: pos(200, 200),
                range: 0,
                wait_ticks: 10,
            },
        ];
        let behavior = RouteType::Loop.to_behavior(waypoints.clone());
        assert_eq!(
            behavior,
            MovementBehavior::Loop {
                waypoints,
                current_index: 0,
            }
        );
    }

    #[test]
    fn route_type_to_behavior_walk_and_despawn() {
        let waypoints = vec![Waypoint {
            position: pos(100, 100),
            range: 0,
            wait_ticks: 0,
        }];
        let behavior = RouteType::WalkAndDespawn.to_behavior(waypoints.clone());
        assert_eq!(
            behavior,
            MovementBehavior::WalkAndDespawn {
                waypoints,
                current_index: 0,
            }
        );
    }

    #[test]
    fn route_type_to_behavior_walk_to_end() {
        let waypoints = vec![Waypoint {
            position: pos(100, 100),
            range: 0,
            wait_ticks: 0,
        }];
        let behavior = RouteType::WalkToEnd.to_behavior(waypoints.clone());
        assert_eq!(
            behavior,
            MovementBehavior::WalkToEnd {
                waypoints,
                current_index: 0,
            }
        );
    }

    #[test]
    fn route_type_to_behavior_ping_pong() {
        let waypoints = vec![
            Waypoint {
                position: pos(100, 100),
                range: 0,
                wait_ticks: 0,
            },
            Waypoint {
                position: pos(200, 200),
                range: 0,
                wait_ticks: 0,
            },
        ];
        let behavior = RouteType::PingPong.to_behavior(waypoints.clone());
        assert_eq!(
            behavior,
            MovementBehavior::PingPong {
                waypoints,
                current_index: 0,
                forward: true,
            }
        );
    }

    #[test]
    fn route_type_to_behavior_ping_pong_despawn() {
        let waypoints = vec![
            Waypoint {
                position: pos(100, 100),
                range: 0,
                wait_ticks: 0,
            },
            Waypoint {
                position: pos(200, 200),
                range: 0,
                wait_ticks: 0,
            },
        ];
        let behavior = RouteType::PingPongDespawn.to_behavior(waypoints.clone());
        assert_eq!(
            behavior,
            MovementBehavior::PingPongDespawn {
                waypoints,
                current_index: 0,
                forward: true,
            }
        );
    }
}
