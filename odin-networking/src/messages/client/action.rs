use crate::messages::common::PositionRaw;
use deku::prelude::*;

const MAX_ROUTE: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite)]
pub struct ActionRaw {
    pub last_pos: PositionRaw,
    pub move_type: u32,
    pub move_speed: u32,
    pub command: [u8; MAX_ROUTE],
    pub destiny: PositionRaw,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<ActionRaw>(), 40);
    }
}
