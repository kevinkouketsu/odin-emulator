#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: u16,
    pub y: u16,
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
}
