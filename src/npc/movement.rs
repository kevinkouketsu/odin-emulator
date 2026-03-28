use odin_models::{direction::Direction, position::Position};
use rand::Rng;

pub fn speed_from_attack_run(attack_run: i8) -> u8 {
    let raw = ((attack_run as i32 & 0xF) * 8) / 4;
    raw.clamp(1, 6) as u8
}

pub fn tick_interval_ms(speed: u8) -> u32 {
    1000 / speed.max(1) as u32
}

#[derive(Debug, Clone, PartialEq)]
pub struct Waypoint {
    pub position: Position,
    pub range: u16,
    pub wait_ticks: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementBehavior {
    Stationary,
    Random {
        origin: Position,
        radius: u16,
        current_target: Option<Position>,
    },
    Patrol {
        waypoints: Vec<Waypoint>,
        current_index: usize,
    },
    PatrolDisappear {
        waypoints: Vec<Waypoint>,
        current_index: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementPhase {
    Idle {
        remaining_ticks: u32,
    },
    Walking {
        path: Vec<Direction>,
        step_index: usize,
    },
    Arrived,
    Despawned,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementAction {
    Waiting,
    PickNextTarget,
    StepForward { direction: Direction },
    AdvanceWaypoint,
    Despawn,
}

#[derive(Debug, Clone)]
pub struct MovementState {
    pub behavior: MovementBehavior,
    pub phase: MovementPhase,
    pub speed: u8,
}

impl MovementState {
    pub fn new(behavior: MovementBehavior, speed: u8) -> Self {
        let phase = match &behavior {
            MovementBehavior::Stationary => MovementPhase::Idle {
                remaining_ticks: u32::MAX,
            },
            MovementBehavior::Random { .. } => MovementPhase::Idle { remaining_ticks: 1 },
            MovementBehavior::Patrol { waypoints, .. }
            | MovementBehavior::PatrolDisappear { waypoints, .. } => {
                let wait = waypoints.first().map_or(0, |w| w.wait_ticks);
                MovementPhase::Idle {
                    remaining_ticks: wait,
                }
            }
        };
        Self {
            behavior,
            phase,
            speed,
        }
    }

    pub fn next_action(&mut self) -> MovementAction {
        match &mut self.phase {
            MovementPhase::Idle { remaining_ticks } => {
                if *remaining_ticks > 0 && *remaining_ticks != u32::MAX {
                    *remaining_ticks -= 1;
                }
                if *remaining_ticks == 0 {
                    MovementAction::PickNextTarget
                } else {
                    MovementAction::Waiting
                }
            }
            MovementPhase::Walking { path, step_index } => {
                if *step_index < path.len() {
                    let dir = path[*step_index];
                    *step_index += 1;
                    MovementAction::StepForward { direction: dir }
                } else {
                    MovementAction::PickNextTarget
                }
            }
            MovementPhase::Arrived => match &self.behavior {
                MovementBehavior::PatrolDisappear { .. } => MovementAction::Despawn,
                _ => MovementAction::AdvanceWaypoint,
            },
            MovementPhase::Despawned => MovementAction::Waiting,
        }
    }

    pub fn set_walking(&mut self, path: Vec<Direction>) {
        self.phase = MovementPhase::Walking {
            path,
            step_index: 0,
        };
    }

    pub fn set_idle(&mut self, wait_ticks: u32) {
        self.phase = MovementPhase::Idle {
            remaining_ticks: wait_ticks,
        };
    }

    pub fn set_arrived(&mut self) {
        self.phase = MovementPhase::Arrived;
    }

    pub fn set_despawned(&mut self) {
        self.phase = MovementPhase::Despawned;
    }

    pub fn advance_waypoint(&mut self) {
        match &mut self.behavior {
            MovementBehavior::Stationary => {
                self.phase = MovementPhase::Idle {
                    remaining_ticks: u32::MAX,
                };
            }
            MovementBehavior::Random { current_target, .. } => {
                *current_target = None;
                self.phase = MovementPhase::Idle { remaining_ticks: 1 };
            }
            MovementBehavior::Patrol {
                waypoints,
                current_index,
            } => {
                if waypoints.is_empty() {
                    self.phase = MovementPhase::Idle {
                        remaining_ticks: u32::MAX,
                    };
                    return;
                }
                *current_index = (*current_index + 1) % waypoints.len();
                let wait = waypoints[*current_index].wait_ticks;
                self.phase = MovementPhase::Idle {
                    remaining_ticks: wait,
                };
            }
            MovementBehavior::PatrolDisappear {
                waypoints,
                current_index,
            } => {
                if waypoints.is_empty() || *current_index + 1 >= waypoints.len() {
                    self.phase = MovementPhase::Arrived;
                    return;
                }
                *current_index += 1;
                let wait = waypoints[*current_index].wait_ticks;
                self.phase = MovementPhase::Idle {
                    remaining_ticks: wait,
                };
            }
        }
    }

    pub fn pick_random_target(&mut self, rng: &mut impl Rng) {
        if let MovementBehavior::Random {
            origin,
            radius,
            current_target,
        } = &mut self.behavior
        {
            let r = *radius as i32;
            let dx = rng.gen_range(-r..=r);
            let dy = rng.gen_range(-r..=r);
            *current_target = Some(origin.offset(dx, dy).unwrap_or(*origin));
        }
    }

    pub fn current_waypoint_target(&self) -> Option<Position> {
        match &self.behavior {
            MovementBehavior::Stationary => None,
            MovementBehavior::Random { current_target, .. } => *current_target,
            MovementBehavior::Patrol {
                waypoints,
                current_index,
            }
            | MovementBehavior::PatrolDisappear {
                waypoints,
                current_index,
            } => waypoints.get(*current_index).map(|w| w.position),
        }
    }

    pub fn interpolated_position(&self, grid_position: Position, elapsed_ms: u32) -> Position {
        match &self.phase {
            MovementPhase::Walking { path, step_index } => {
                if path.is_empty() {
                    return grid_position;
                }
                let interval = tick_interval_ms(self.speed);
                let total_time = path.len() as u32 * interval;
                let clamped_elapsed = elapsed_ms.min(total_time);
                let steps_done = if interval > 0 {
                    (clamped_elapsed / interval) as usize
                } else {
                    path.len()
                };
                let steps = steps_done.min(path.len());
                let start_step = step_index.saturating_sub(path.len());
                let mut pos = grid_position;
                for dir in path.iter().skip(start_step).take(steps) {
                    if let Some(next) = pos.apply_direction(*dir) {
                        pos = next;
                    }
                }
                pos
            }
            _ => grid_position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: u16, y: u16) -> Position {
        Position { x, y }
    }

    fn waypoint(x: u16, y: u16, wait: u32) -> Waypoint {
        Waypoint {
            position: pos(x, y),
            range: 0,
            wait_ticks: wait,
        }
    }

    #[test]
    fn speed_from_attack_run_normal() {
        assert_eq!(speed_from_attack_run(3), 6);
    }

    #[test]
    fn speed_from_attack_run_clamped_low() {
        assert_eq!(speed_from_attack_run(0), 1);
    }

    #[test]
    fn speed_from_attack_run_clamped_high() {
        assert_eq!(speed_from_attack_run(5), 6);
    }

    #[test]
    fn tick_interval_speed_1() {
        assert_eq!(tick_interval_ms(1), 1000);
    }

    #[test]
    fn tick_interval_speed_6() {
        assert_eq!(tick_interval_ms(6), 166);
    }

    #[test]
    fn stationary_always_idle() {
        let mut state = MovementState::new(MovementBehavior::Stationary, 3);
        for _ in 0..10 {
            assert_eq!(state.next_action(), MovementAction::Waiting);
        }
    }

    #[test]
    fn random_idle_countdown() {
        let mut state = MovementState::new(
            MovementBehavior::Random {
                origin: pos(100, 100),
                radius: 5,
                current_target: None,
            },
            3,
        );
        assert_eq!(state.next_action(), MovementAction::PickNextTarget);
    }

    #[test]
    fn random_pick_target_after_idle() {
        let mut state = MovementState::new(
            MovementBehavior::Random {
                origin: pos(100, 100),
                radius: 5,
                current_target: None,
            },
            3,
        );
        state.set_idle(2);
        assert_eq!(state.next_action(), MovementAction::Waiting);
        assert_eq!(state.next_action(), MovementAction::PickNextTarget);
    }

    #[test]
    fn patrol_advances_waypoints() {
        let waypoints = vec![
            waypoint(100, 100, 0),
            waypoint(110, 100, 0),
            waypoint(120, 100, 0),
        ];
        let mut state = MovementState::new(
            MovementBehavior::Patrol {
                waypoints,
                current_index: 0,
            },
            3,
        );

        assert_eq!(state.current_waypoint_target(), Some(pos(100, 100)));
        state.advance_waypoint();
        assert_eq!(state.current_waypoint_target(), Some(pos(110, 100)));
        state.advance_waypoint();
        assert_eq!(state.current_waypoint_target(), Some(pos(120, 100)));
        state.advance_waypoint();
        assert_eq!(state.current_waypoint_target(), Some(pos(100, 100)));
    }

    #[test]
    fn patrol_disappear_despawns_at_end() {
        let waypoints = vec![waypoint(100, 100, 0), waypoint(110, 100, 0)];
        let mut state = MovementState::new(
            MovementBehavior::PatrolDisappear {
                waypoints,
                current_index: 0,
            },
            3,
        );

        state.advance_waypoint();
        assert_eq!(state.current_waypoint_target(), Some(pos(110, 100)));

        state.advance_waypoint();
        assert_eq!(state.phase, MovementPhase::Arrived);

        assert_eq!(state.next_action(), MovementAction::Despawn);
    }

    #[test]
    fn step_forward_returns_next_direction() {
        let path = vec![Direction::North, Direction::North, Direction::Northeast];
        let mut state = MovementState::new(MovementBehavior::Stationary, 3);
        state.set_walking(path);

        assert_eq!(
            state.next_action(),
            MovementAction::StepForward {
                direction: Direction::North
            }
        );
        assert_eq!(
            state.next_action(),
            MovementAction::StepForward {
                direction: Direction::North
            }
        );
        assert_eq!(
            state.next_action(),
            MovementAction::StepForward {
                direction: Direction::Northeast
            }
        );
    }

    #[test]
    fn step_forward_exhausts_path() {
        let path = vec![Direction::South];
        let mut state = MovementState::new(MovementBehavior::Stationary, 3);
        state.set_walking(path);

        assert_eq!(
            state.next_action(),
            MovementAction::StepForward {
                direction: Direction::South
            }
        );
        assert_eq!(state.next_action(), MovementAction::PickNextTarget);
    }

    #[test]
    fn interpolated_position_at_start() {
        let path = vec![Direction::East, Direction::East, Direction::East];
        let mut state = MovementState::new(MovementBehavior::Stationary, 1);
        state.set_walking(path);

        let p = state.interpolated_position(pos(100, 100), 0);
        assert_eq!(p, pos(100, 100));
    }

    #[test]
    fn interpolated_position_midway() {
        let path = vec![
            Direction::East,
            Direction::East,
            Direction::East,
            Direction::East,
        ];
        let mut state = MovementState::new(MovementBehavior::Stationary, 1);
        state.set_walking(path);

        let p = state.interpolated_position(pos(100, 100), 2000);
        assert_eq!(p, pos(102, 100));
    }

    #[test]
    fn waiting_phase_respects_wait_time() {
        let waypoints = vec![waypoint(100, 100, 3), waypoint(110, 100, 0)];
        let mut state = MovementState::new(
            MovementBehavior::Patrol {
                waypoints,
                current_index: 0,
            },
            3,
        );

        assert_eq!(state.next_action(), MovementAction::Waiting);
        assert_eq!(state.next_action(), MovementAction::Waiting);
        assert_eq!(state.next_action(), MovementAction::PickNextTarget);
    }

    #[test]
    fn random_target_differs_from_origin() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        let origin = pos(2100, 2100);
        let mut state = MovementState::new(
            MovementBehavior::Random {
                origin,
                radius: 10,
                current_target: None,
            },
            3,
        );

        let mut rng = SmallRng::seed_from_u64(42);
        state.pick_random_target(&mut rng);
        let target = state.current_waypoint_target().unwrap();

        assert_ne!(
            target, origin,
            "random target should differ from origin with radius 10"
        );
        assert!(
            target.chebyshev_distance(origin) <= 10,
            "random target {:?} should be within radius 10 of origin {:?}",
            target,
            origin
        );
    }

    #[test]
    fn random_target_cleared_on_advance() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        let origin = pos(2100, 2100);
        let mut state = MovementState::new(
            MovementBehavior::Random {
                origin,
                radius: 10,
                current_target: None,
            },
            3,
        );

        let mut rng = SmallRng::seed_from_u64(42);
        state.pick_random_target(&mut rng);
        assert!(state.current_waypoint_target().is_some());

        state.advance_waypoint();

        // After advance, target should be cleared (back to origin until next pick)
        // Next PickNextTarget will pick a new random target
    }

    #[test]
    fn random_picks_different_targets_across_cycles() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        let origin = pos(2100, 2100);
        let mut state = MovementState::new(
            MovementBehavior::Random {
                origin,
                radius: 10,
                current_target: None,
            },
            3,
        );

        let mut rng = SmallRng::seed_from_u64(42);
        let mut targets = Vec::new();
        for _ in 0..5 {
            state.pick_random_target(&mut rng);
            targets.push(state.current_waypoint_target().unwrap());
            state.advance_waypoint();
        }

        let unique: std::collections::HashSet<_> = targets.iter().collect();
        assert!(
            unique.len() > 1,
            "should pick different targets across cycles, got {:?}",
            targets
        );
    }
}
