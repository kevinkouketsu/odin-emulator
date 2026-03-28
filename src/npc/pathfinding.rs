use odin_models::{direction::Direction, position::Position};

pub const MAX_PATH_STEPS: usize = 23;

pub trait Pathfinder {
    fn find_path(
        &self,
        from: Position,
        to: Position,
        max_steps: usize,
        is_passable: &dyn Fn(Position) -> bool,
    ) -> Vec<Direction>;
}

pub struct GreedyPathfinder;

impl Pathfinder for GreedyPathfinder {
    fn find_path(
        &self,
        from: Position,
        to: Position,
        max_steps: usize,
        is_passable: &dyn Fn(Position) -> bool,
    ) -> Vec<Direction> {
        let mut path = Vec::new();
        let mut current = from;

        for _ in 0..max_steps {
            let Some(best_dir) = Direction::toward(current, to) else {
                break;
            };

            if let Some(candidate) = current.apply_direction(best_dir)
                && is_passable(candidate)
            {
                path.push(best_dir);
                current = candidate;
                continue;
            }

            if let Some((dir, pos)) = find_alternative(current, to, is_passable) {
                path.push(dir);
                current = pos;
            } else {
                break;
            }
        }

        path
    }
}

fn find_alternative(
    current: Position,
    target: Position,
    is_passable: &dyn Fn(Position) -> bool,
) -> Option<(Direction, Position)> {
    let mut candidates: Vec<(Direction, Position)> = Direction::ALL
        .iter()
        .copied()
        .filter_map(|dir| {
            let pos = current.apply_direction(dir)?;
            is_passable(pos).then_some((dir, pos))
        })
        .collect();

    candidates.sort_by_key(|(_, pos)| pos.chebyshev_distance(target));
    candidates.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: u16, y: u16) -> Position {
        Position { x, y }
    }

    fn open_map() -> impl Fn(Position) -> bool {
        |_| true
    }

    #[test]
    fn greedy_straight_line_north() {
        let path =
            GreedyPathfinder.find_path(pos(100, 110), pos(100, 100), MAX_PATH_STEPS, &open_map());
        assert_eq!(path.len(), 10);
        assert!(path.iter().all(|d| *d == Direction::North));
    }

    #[test]
    fn greedy_diagonal() {
        let path =
            GreedyPathfinder.find_path(pos(100, 100), pos(105, 105), MAX_PATH_STEPS, &open_map());
        assert_eq!(path.len(), 5);
        assert!(path.iter().all(|d| *d == Direction::Southeast));
    }

    #[test]
    fn greedy_stops_at_target() {
        let path =
            GreedyPathfinder.find_path(pos(100, 100), pos(100, 102), MAX_PATH_STEPS, &open_map());
        assert_eq!(path.len(), 2);
        assert!(path.iter().all(|d| *d == Direction::South));
    }

    #[test]
    fn greedy_max_steps_caps() {
        let path = GreedyPathfinder.find_path(pos(100, 100), pos(100, 130), 5, &open_map());
        assert_eq!(path.len(), 5);
    }

    #[test]
    fn greedy_avoids_blocked_cell() {
        let blocked = pos(100, 99);
        let is_passable = move |p: Position| p != blocked;
        let path =
            GreedyPathfinder.find_path(pos(100, 100), pos(100, 90), MAX_PATH_STEPS, &is_passable);
        assert!(!path.is_empty());
        assert_ne!(path[0], Direction::North);
        assert!(path[0] == Direction::Northeast || path[0] == Direction::Northwest);
    }

    #[test]
    fn greedy_avoids_occupied_cell() {
        let occupied = pos(100, 99);
        let is_passable = move |p: Position| p != occupied;
        let path =
            GreedyPathfinder.find_path(pos(100, 100), pos(100, 95), MAX_PATH_STEPS, &is_passable);
        assert!(!path.is_empty());
        assert_ne!(path[0], Direction::North);
    }

    #[test]
    fn greedy_stuck_returns_partial() {
        let start = pos(100, 100);
        let is_passable = move |p: Position| p == start;
        let path = GreedyPathfinder.find_path(start, pos(100, 90), MAX_PATH_STEPS, &is_passable);
        assert!(path.is_empty());
    }

    #[test]
    fn greedy_already_at_target() {
        let path =
            GreedyPathfinder.find_path(pos(100, 100), pos(100, 100), MAX_PATH_STEPS, &open_map());
        assert!(path.is_empty());
    }

    #[test]
    fn greedy_steep_terrain_avoided() {
        let impassable = pos(101, 99);
        let is_passable = move |p: Position| p != impassable;
        let path =
            GreedyPathfinder.find_path(pos(100, 100), pos(102, 98), MAX_PATH_STEPS, &is_passable);
        assert!(!path.is_empty());
        let mut current = pos(100, 100);
        for dir in &path {
            current = current.apply_direction(*dir).unwrap();
            assert_ne!(current, impassable);
        }
        assert_eq!(current, pos(102, 98));
    }
}
