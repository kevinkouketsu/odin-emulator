use crate::direction::Direction;
use std::fmt::Display;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn chebyshev_distance(self, other: Position) -> u16 {
        let dx = (self.x as i32 - other.x as i32).unsigned_abs() as u16;
        let dy = (self.y as i32 - other.y as i32).unsigned_abs() as u16;
        dx.max(dy)
    }

    pub fn distance_to(self, other: Position) -> f64 {
        let dx = self.x as f64 - other.x as f64;
        let dy = self.y as f64 - other.y as f64;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn apply_direction(self, dir: Direction) -> Option<Position> {
        self.offset(dir.dx(), dir.dy())
    }

    pub fn offset(self, dx: i32, dy: i32) -> Option<Position> {
        let nx = self.x as i32 + dx;
        let ny = self.y as i32 + dy;
        if nx < 0 || nx > u16::MAX as i32 || ny < 0 || ny > u16::MAX as i32 {
            return None;
        }
        Some(Position {
            x: nx as u16,
            y: ny as u16,
        })
    }
}
impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}
impl From<(u16, u16)> for Position {
    fn from(value: (u16, u16)) -> Self {
        Position {
            x: value.0,
            y: value.1,
        }
    }
}
impl TryFrom<&str> for Position {
    type Error = FailParsePositionString;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = match value.starts_with('(') {
            true => &value[1..value.len() - 1],
            false => value,
        };

        let (x, y) = value
            .split_once(',')
            .ok_or_else(|| FailParsePositionString(value.to_string()))?;

        let x = x
            .trim()
            .parse::<u16>()
            .map_err(|err| FailParsePositionString(err.to_string()))?;
        let y = y
            .trim()
            .parse::<u16>()
            .map_err(|err| FailParsePositionString(err.to_string()))?;

        Ok(Position { x, y })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid position string: {0}")]
pub struct FailParsePositionString(String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_position() {
        assert_eq!(
            Position::try_from("2100, 2100").unwrap(),
            Position { x: 2100, y: 2100 }
        );
        assert_eq!(
            Position::try_from("(2100, 2100)").unwrap(),
            Position { x: 2100, y: 2100 }
        )
    }

    #[test]
    fn chebyshev_distance_cardinal() {
        let a = Position { x: 100, y: 100 };
        let b = Position { x: 100, y: 105 };
        assert_eq!(a.chebyshev_distance(b), 5);
    }

    #[test]
    fn chebyshev_distance_diagonal() {
        let a = Position { x: 100, y: 100 };
        let b = Position { x: 103, y: 105 };
        assert_eq!(a.chebyshev_distance(b), 5);
    }

    #[test]
    fn apply_direction_north() {
        let p = Position { x: 100, y: 100 };
        assert_eq!(
            p.apply_direction(Direction::North),
            Some(Position { x: 100, y: 99 })
        );
    }

    #[test]
    fn apply_direction_underflow() {
        let p = Position { x: 0, y: 0 };
        assert_eq!(p.apply_direction(Direction::North), None);
        assert_eq!(p.apply_direction(Direction::West), None);
        assert_eq!(p.apply_direction(Direction::Northwest), None);
    }

    #[test]
    fn offset_positive() {
        let p = Position { x: 100, y: 100 };
        assert_eq!(p.offset(5, -3), Some(Position { x: 105, y: 97 }));
    }

    #[test]
    fn offset_overflow() {
        let p = Position { x: 65535, y: 0 };
        assert_eq!(p.offset(1, 0), None);
    }
}
