#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
