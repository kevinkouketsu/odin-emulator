use crate::position::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    Northwest = 1,
    North = 2,
    Northeast = 3,
    West = 4,
    East = 6,
    Southwest = 7,
    South = 8,
    Southeast = 9,
}

impl Direction {
    pub const ALL: [Direction; 8] = [
        Direction::Northwest,
        Direction::North,
        Direction::Northeast,
        Direction::West,
        Direction::East,
        Direction::Southwest,
        Direction::South,
        Direction::Southeast,
    ];

    pub fn dx(self) -> i32 {
        match self {
            Direction::Northwest | Direction::West | Direction::Southwest => -1,
            Direction::North | Direction::South => 0,
            Direction::Northeast | Direction::East | Direction::Southeast => 1,
        }
    }

    pub fn dy(self) -> i32 {
        match self {
            Direction::Northwest | Direction::North | Direction::Northeast => -1,
            Direction::West | Direction::East => 0,
            Direction::Southwest | Direction::South | Direction::Southeast => 1,
        }
    }

    pub fn to_route_byte(self) -> u8 {
        b'0' + self as u8
    }

    pub fn toward(from: Position, to: Position) -> Option<Self> {
        let dx = to.x as i32 - from.x as i32;
        let dy = to.y as i32 - from.y as i32;

        if dx == 0 && dy == 0 {
            return None;
        }

        let sx = dx.signum();
        let sy = dy.signum();

        Some(match (sx, sy) {
            (-1, -1) => Direction::Northwest,
            (0, -1) => Direction::North,
            (1, -1) => Direction::Northeast,
            (-1, 0) => Direction::West,
            (1, 0) => Direction::East,
            (-1, 1) => Direction::Southwest,
            (0, 1) => Direction::South,
            (1, 1) => Direction::Southeast,
            _ => unreachable!(),
        })
    }
}

impl TryFrom<u8> for Direction {
    type Error = InvalidDirection;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Direction::Northwest),
            2 => Ok(Direction::North),
            3 => Ok(Direction::Northeast),
            4 => Ok(Direction::West),
            6 => Ok(Direction::East),
            7 => Ok(Direction::Southwest),
            8 => Ok(Direction::South),
            9 => Ok(Direction::Southeast),
            _ => Err(InvalidDirection(value)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid direction value: {0}")]
pub struct InvalidDirection(u8);

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: u16, y: u16) -> Position {
        Position { x, y }
    }

    #[test]
    fn direction_dx_dy_correct() {
        assert_eq!(
            (Direction::Northwest.dx(), Direction::Northwest.dy()),
            (-1, -1)
        );
        assert_eq!((Direction::North.dx(), Direction::North.dy()), (0, -1));
        assert_eq!(
            (Direction::Northeast.dx(), Direction::Northeast.dy()),
            (1, -1)
        );
        assert_eq!((Direction::West.dx(), Direction::West.dy()), (-1, 0));
        assert_eq!((Direction::East.dx(), Direction::East.dy()), (1, 0));
        assert_eq!(
            (Direction::Southwest.dx(), Direction::Southwest.dy()),
            (-1, 1)
        );
        assert_eq!((Direction::South.dx(), Direction::South.dy()), (0, 1));
        assert_eq!(
            (Direction::Southeast.dx(), Direction::Southeast.dy()),
            (1, 1)
        );
    }

    #[test]
    fn toward_north() {
        assert_eq!(
            Direction::toward(pos(100, 105), pos(100, 100)),
            Some(Direction::North)
        );
    }

    #[test]
    fn toward_southeast() {
        assert_eq!(
            Direction::toward(pos(100, 100), pos(105, 105)),
            Some(Direction::Southeast)
        );
    }

    #[test]
    fn toward_same_position_is_none() {
        assert_eq!(Direction::toward(pos(100, 100), pos(100, 100)), None);
    }

    #[test]
    fn toward_diagonal_preference() {
        assert_eq!(
            Direction::toward(pos(100, 100), pos(103, 101)),
            Some(Direction::Southeast)
        );
    }

    #[test]
    fn try_from_valid_numpad() {
        for val in [1u8, 2, 3, 4, 6, 7, 8, 9] {
            assert!(Direction::try_from(val).is_ok());
        }
    }

    #[test]
    fn to_route_byte_is_ascii() {
        assert_eq!(Direction::North.to_route_byte(), b'2');
        assert_eq!(Direction::East.to_route_byte(), b'6');
        assert_eq!(Direction::Northwest.to_route_byte(), b'1');
        assert_eq!(Direction::Southeast.to_route_byte(), b'9');
        for dir in Direction::ALL {
            let byte = dir.to_route_byte();
            assert!(
                (b'1'..=b'9').contains(&byte),
                "route byte must be ASCII digit"
            );
        }
    }

    #[test]
    fn try_from_invalid_values() {
        assert!(Direction::try_from(0u8).is_err());
        assert!(Direction::try_from(5u8).is_err());
        assert!(Direction::try_from(10u8).is_err());
    }
}
