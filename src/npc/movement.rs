use odin_models::position::Position;
use rand::Rng;

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
    Loop {
        waypoints: Vec<Waypoint>,
        current_index: usize,
    },
    WalkAndDespawn {
        waypoints: Vec<Waypoint>,
        current_index: usize,
    },
    WalkToEnd {
        waypoints: Vec<Waypoint>,
        current_index: usize,
    },
    PingPong {
        waypoints: Vec<Waypoint>,
        current_index: usize,
        forward: bool,
    },
    PingPongDespawn {
        waypoints: Vec<Waypoint>,
        current_index: usize,
        forward: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementPhase {
    Idle,
    Waiting(u32),
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickAction {
    Wait,
    Move,
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
            MovementBehavior::Stationary => MovementPhase::Stopped,
            MovementBehavior::Random { .. } => MovementPhase::Idle,
            MovementBehavior::Loop { waypoints, .. }
            | MovementBehavior::WalkAndDespawn { waypoints, .. }
            | MovementBehavior::WalkToEnd { waypoints, .. }
            | MovementBehavior::PingPong { waypoints, .. }
            | MovementBehavior::PingPongDespawn { waypoints, .. } => {
                let wait = waypoints.first().map_or(0, |w| w.wait_ticks);
                if wait > 0 {
                    MovementPhase::Waiting(wait)
                } else {
                    MovementPhase::Idle
                }
            }
        };
        Self {
            behavior,
            phase,
            speed,
        }
    }

    pub fn tick(&mut self, at_target: bool) -> TickAction {
        match &mut self.phase {
            MovementPhase::Stopped => TickAction::Wait,
            MovementPhase::Waiting(n) => {
                if *n > 1 {
                    *n -= 1;
                    return TickAction::Wait;
                }
                self.advance_waypoint_internal()
            }
            MovementPhase::Idle => {
                if at_target {
                    let wait = self.current_waypoint_wait();
                    if wait > 0 {
                        self.phase = MovementPhase::Waiting(wait);
                        return TickAction::Wait;
                    }
                    self.advance_waypoint_internal()
                } else {
                    TickAction::Move
                }
            }
        }
    }

    fn advance_waypoint_internal(&mut self) -> TickAction {
        match &mut self.behavior {
            MovementBehavior::Stationary => {
                self.phase = MovementPhase::Stopped;
                TickAction::Wait
            }
            MovementBehavior::Random { current_target, .. } => {
                *current_target = None;
                self.phase = MovementPhase::Idle;
                TickAction::Wait
            }
            MovementBehavior::Loop {
                waypoints,
                current_index,
            } => {
                if waypoints.is_empty() {
                    self.phase = MovementPhase::Stopped;
                    return TickAction::Wait;
                }
                *current_index = (*current_index + 1) % waypoints.len();
                self.phase = MovementPhase::Idle;
                TickAction::Wait
            }
            MovementBehavior::WalkAndDespawn {
                waypoints,
                current_index,
            } => {
                if waypoints.is_empty() || *current_index + 1 >= waypoints.len() {
                    TickAction::Despawn
                } else {
                    *current_index += 1;
                    self.phase = MovementPhase::Idle;
                    TickAction::Wait
                }
            }
            MovementBehavior::WalkToEnd {
                waypoints,
                current_index,
            } => {
                if waypoints.is_empty() || *current_index + 1 >= waypoints.len() {
                    self.phase = MovementPhase::Stopped;
                    TickAction::Wait
                } else {
                    *current_index += 1;
                    self.phase = MovementPhase::Idle;
                    TickAction::Wait
                }
            }
            MovementBehavior::PingPong {
                waypoints,
                current_index,
                forward,
            } => {
                if waypoints.len() < 2 {
                    self.phase = MovementPhase::Stopped;
                    return TickAction::Wait;
                }
                if *forward {
                    if *current_index + 1 >= waypoints.len() {
                        *forward = false;
                        *current_index = waypoints.len() - 2;
                    } else {
                        *current_index += 1;
                    }
                } else if *current_index == 0 {
                    *forward = true;
                    *current_index = 1;
                } else {
                    *current_index -= 1;
                }
                self.phase = MovementPhase::Idle;
                TickAction::Wait
            }
            MovementBehavior::PingPongDespawn {
                waypoints,
                current_index,
                forward,
            } => {
                if waypoints.len() < 2 {
                    return TickAction::Despawn;
                }
                if *forward {
                    if *current_index + 1 >= waypoints.len() {
                        *forward = false;
                        *current_index = waypoints.len() - 2;
                    } else {
                        *current_index += 1;
                    }
                } else if *current_index == 0 {
                    return TickAction::Despawn;
                } else {
                    *current_index -= 1;
                }
                self.phase = MovementPhase::Idle;
                TickAction::Wait
            }
        }
    }

    pub fn current_waypoint_wait(&self) -> u32 {
        match &self.behavior {
            MovementBehavior::Stationary => 0,
            MovementBehavior::Random { .. } => 0,
            MovementBehavior::Loop {
                waypoints,
                current_index,
            }
            | MovementBehavior::WalkAndDespawn {
                waypoints,
                current_index,
            }
            | MovementBehavior::WalkToEnd {
                waypoints,
                current_index,
            }
            | MovementBehavior::PingPong {
                waypoints,
                current_index,
                ..
            }
            | MovementBehavior::PingPongDespawn {
                waypoints,
                current_index,
                ..
            } => waypoints.get(*current_index).map_or(0, |w| w.wait_ticks),
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
            MovementBehavior::Loop {
                waypoints,
                current_index,
            }
            | MovementBehavior::WalkAndDespawn {
                waypoints,
                current_index,
            }
            | MovementBehavior::WalkToEnd {
                waypoints,
                current_index,
            }
            | MovementBehavior::PingPong {
                waypoints,
                current_index,
                ..
            }
            | MovementBehavior::PingPongDespawn {
                waypoints,
                current_index,
                ..
            } => waypoints.get(*current_index).map(|w| w.position),
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

    // --- MovementState::new initial phase ---

    #[test]
    fn stationary_starts_stopped() {
        let state = MovementState::new(MovementBehavior::Stationary, 3);
        assert_eq!(state.phase, MovementPhase::Stopped);
    }

    #[test]
    fn loop_starts_waiting_with_initial_wait() {
        let state = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![waypoint(100, 100, 5), waypoint(110, 100, 0)],
                current_index: 0,
            },
            3,
        );
        assert_eq!(state.phase, MovementPhase::Waiting(5));
    }

    #[test]
    fn loop_starts_idle_when_no_wait() {
        let state = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![waypoint(100, 100, 0), waypoint(110, 100, 0)],
                current_index: 0,
            },
            3,
        );
        assert_eq!(state.phase, MovementPhase::Idle);
    }

    #[test]
    fn random_starts_idle() {
        let state = MovementState::new(
            MovementBehavior::Random {
                origin: pos(100, 100),
                radius: 5,
                current_target: None,
            },
            3,
        );
        assert_eq!(state.phase, MovementPhase::Idle);
    }

    #[test]
    fn walk_and_despawn_starts_with_wait() {
        let state = MovementState::new(
            MovementBehavior::WalkAndDespawn {
                waypoints: vec![waypoint(100, 100, 4), waypoint(110, 100, 0)],
                current_index: 0,
            },
            3,
        );
        assert_eq!(state.phase, MovementPhase::Waiting(4));
    }

    // --- tick(): Stopped ---

    #[test]
    fn stopped_always_returns_wait() {
        let mut state = MovementState::new(MovementBehavior::Stationary, 3);
        assert_eq!(state.tick(true), TickAction::Wait);
        assert_eq!(state.tick(false), TickAction::Wait);
        assert_eq!(state.phase, MovementPhase::Stopped);
    }

    // --- tick(): Waiting ---

    #[test]
    fn waiting_counts_down() {
        let mut state = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![waypoint(100, 100, 3), waypoint(110, 100, 0)],
                current_index: 0,
            },
            3,
        );
        assert_eq!(state.phase, MovementPhase::Waiting(3));

        assert_eq!(state.tick(true), TickAction::Wait);
        assert_eq!(state.phase, MovementPhase::Waiting(2));

        assert_eq!(state.tick(true), TickAction::Wait);
        assert_eq!(state.phase, MovementPhase::Waiting(1));
    }

    #[test]
    fn waiting_done_advances_waypoint() {
        let mut state = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![waypoint(100, 100, 1), waypoint(110, 100, 0)],
                current_index: 0,
            },
            3,
        );
        assert_eq!(state.phase, MovementPhase::Waiting(1));

        // Waiting(1) -> advance to index 1
        assert_eq!(state.tick(true), TickAction::Wait);
        assert_eq!(state.phase, MovementPhase::Idle);
        assert_eq!(state.current_waypoint_target(), Some(pos(110, 100)));
    }

    // --- tick(): Idle ---

    #[test]
    fn idle_at_target_with_wait_starts_waiting() {
        let mut state = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![waypoint(100, 100, 0), waypoint(110, 100, 5)],
                current_index: 1,
            },
            3,
        );
        // Phase is Idle (index 1 has wait=5 but new() reads index 0's wait=0 only for initial)
        // Actually new() reads waypoints.first() which is index 0 with wait=0, so Idle.
        // But current_index is 1, so current_waypoint_wait() returns 5.
        assert_eq!(state.phase, MovementPhase::Idle);

        assert_eq!(state.tick(true), TickAction::Wait);
        assert_eq!(state.phase, MovementPhase::Waiting(5));
    }

    #[test]
    fn idle_at_target_no_wait_advances() {
        let mut state = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![waypoint(100, 100, 0), waypoint(110, 100, 0)],
                current_index: 0,
            },
            3,
        );
        assert_eq!(state.phase, MovementPhase::Idle);

        // Idle + at_target + wait=0 -> advance to index 1
        assert_eq!(state.tick(true), TickAction::Wait);
        assert_eq!(state.current_waypoint_target(), Some(pos(110, 100)));
    }

    #[test]
    fn idle_not_at_target_returns_move() {
        let mut state = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![waypoint(100, 100, 0), waypoint(110, 100, 0)],
                current_index: 0,
            },
            3,
        );

        assert_eq!(state.tick(false), TickAction::Move);
        assert_eq!(state.phase, MovementPhase::Idle);
    }

    // --- advance: Loop ---

    #[test]
    fn loop_advance_wraps_index() {
        let mut state = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![
                    waypoint(100, 100, 0),
                    waypoint(110, 100, 0),
                    waypoint(120, 100, 0),
                ],
                current_index: 0,
            },
            3,
        );

        // Advance through all waypoints: 0->1->2->0
        state.tick(true); // advance 0->1
        assert_eq!(state.current_waypoint_target(), Some(pos(110, 100)));
        state.tick(true); // advance 1->2
        assert_eq!(state.current_waypoint_target(), Some(pos(120, 100)));
        state.tick(true); // advance 2->0 (wrap)
        assert_eq!(state.current_waypoint_target(), Some(pos(100, 100)));
    }

    // --- advance: WalkAndDespawn ---

    #[test]
    fn walk_and_despawn_returns_despawn_at_end() {
        let mut state = MovementState::new(
            MovementBehavior::WalkAndDespawn {
                waypoints: vec![waypoint(100, 100, 0), waypoint(110, 100, 0)],
                current_index: 1,
            },
            3,
        );

        // At final waypoint, wait=0 -> Despawn
        assert_eq!(state.tick(true), TickAction::Despawn);
    }

    #[test]
    fn walk_and_despawn_advances_before_end() {
        let mut state = MovementState::new(
            MovementBehavior::WalkAndDespawn {
                waypoints: vec![waypoint(100, 100, 0), waypoint(110, 100, 0)],
                current_index: 0,
            },
            3,
        );

        // At waypoint 0, wait=0 -> advance to 1
        assert_eq!(state.tick(true), TickAction::Wait);
        assert_eq!(state.current_waypoint_target(), Some(pos(110, 100)));
    }

    // --- advance: WalkToEnd ---

    #[test]
    fn walk_to_end_stops_at_end() {
        let mut state = MovementState::new(
            MovementBehavior::WalkToEnd {
                waypoints: vec![waypoint(100, 100, 0), waypoint(110, 100, 0)],
                current_index: 1,
            },
            3,
        );

        assert_eq!(state.tick(true), TickAction::Wait);
        assert_eq!(state.phase, MovementPhase::Stopped);
    }

    // --- advance: PingPong ---

    #[test]
    fn ping_pong_reverses_at_end() {
        let mut state = MovementState::new(
            MovementBehavior::PingPong {
                waypoints: vec![waypoint(0, 0, 0), waypoint(1, 0, 0), waypoint(2, 0, 0)],
                current_index: 2,
                forward: true,
            },
            3,
        );

        // At end, forward -> reverse to index 1
        state.tick(true);
        assert_eq!(state.current_waypoint_target(), Some(pos(1, 0)));
    }

    #[test]
    fn ping_pong_full_cycle() {
        let mut state = MovementState::new(
            MovementBehavior::PingPong {
                waypoints: vec![
                    waypoint(0, 0, 0),
                    waypoint(1, 0, 0),
                    waypoint(2, 0, 0),
                    waypoint(3, 0, 0),
                ],
                current_index: 0,
                forward: true,
            },
            3,
        );

        let expected = [1, 2, 3, 2, 1, 0, 1, 2, 3, 2, 1, 0];
        for &expected_x in &expected {
            state.tick(true); // advance
            assert_eq!(
                state.current_waypoint_target(),
                Some(pos(expected_x, 0)),
                "expected waypoint x={expected_x}"
            );
        }
    }

    // --- advance: PingPongDespawn ---

    #[test]
    fn ping_pong_despawn_despawns_at_start() {
        let mut state = MovementState::new(
            MovementBehavior::PingPongDespawn {
                waypoints: vec![
                    waypoint(100, 100, 0),
                    waypoint(110, 100, 0),
                    waypoint(120, 100, 0),
                ],
                current_index: 0,
                forward: false,
            },
            3,
        );

        assert_eq!(state.tick(true), TickAction::Despawn);
    }

    #[test]
    fn ping_pong_despawn_reverses_at_end() {
        let mut state = MovementState::new(
            MovementBehavior::PingPongDespawn {
                waypoints: vec![
                    waypoint(100, 100, 0),
                    waypoint(110, 100, 0),
                    waypoint(120, 100, 0),
                ],
                current_index: 2,
                forward: true,
            },
            3,
        );

        assert_eq!(state.tick(true), TickAction::Wait);
        assert_eq!(state.current_waypoint_target(), Some(pos(110, 100)));
    }

    // --- Random ---

    #[test]
    fn random_advance_clears_target() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        let mut state = MovementState::new(
            MovementBehavior::Random {
                origin: pos(100, 100),
                radius: 10,
                current_target: None,
            },
            3,
        );

        let mut rng = SmallRng::seed_from_u64(42);
        state.pick_random_target(&mut rng);
        assert!(state.current_waypoint_target().is_some());

        // Simulate arriving at target: tick(at_target=true) with wait=0 -> advance -> clears target
        state.tick(true);
        assert!(state.current_waypoint_target().is_none());
    }

    #[test]
    fn pick_random_target_sets_target() {
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

        assert_ne!(target, origin);
        assert!(target.chebyshev_distance(origin) <= 10);
    }

    // --- current_waypoint_target ---

    #[test]
    fn current_waypoint_target_returns_position() {
        let state = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![waypoint(100, 100, 0), waypoint(200, 200, 0)],
                current_index: 1,
            },
            3,
        );
        assert_eq!(state.current_waypoint_target(), Some(pos(200, 200)));
    }

    #[test]
    fn current_waypoint_target_none_for_stationary() {
        let state = MovementState::new(MovementBehavior::Stationary, 3);
        assert_eq!(state.current_waypoint_target(), None);
    }

    // --- current_waypoint_wait ---

    #[test]
    fn current_waypoint_wait_returns_wait() {
        let state = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![waypoint(100, 100, 7), waypoint(200, 200, 3)],
                current_index: 1,
            },
            3,
        );
        assert_eq!(state.current_waypoint_wait(), 3);
    }

    #[test]
    fn current_waypoint_wait_zero_for_random() {
        let state = MovementState::new(
            MovementBehavior::Random {
                origin: pos(100, 100),
                radius: 5,
                current_target: None,
            },
            3,
        );
        assert_eq!(state.current_waypoint_wait(), 0);
    }

    // --- Full flow: wait then move ---

    #[test]
    fn full_flow_wait_then_advance_then_move() {
        let mut state = MovementState::new(
            MovementBehavior::Loop {
                waypoints: vec![waypoint(100, 100, 2), waypoint(110, 100, 0)],
                current_index: 0,
            },
            3,
        );

        // Waiting(2) -> Waiting(1)
        assert_eq!(state.tick(true), TickAction::Wait);
        assert_eq!(state.phase, MovementPhase::Waiting(1));

        // Waiting(1) -> advance to index 1, Idle
        assert_eq!(state.tick(true), TickAction::Wait);
        assert_eq!(state.phase, MovementPhase::Idle);
        assert_eq!(state.current_waypoint_target(), Some(pos(110, 100)));

        // Idle + not at target -> Move
        assert_eq!(state.tick(false), TickAction::Move);
    }
}
