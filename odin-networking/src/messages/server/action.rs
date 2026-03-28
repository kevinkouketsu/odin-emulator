use crate::{
    WritableResource, WritableResourceError,
    messages::{ServerMessage, client::action::ActionRaw, common::PositionRaw},
};
use odin_models::{direction::Direction, position::Position};

pub const MAX_ROUTE: usize = 24;

#[derive(Clone, Copy)]
pub struct ActionBroadcastData {
    pub mover_id: u16,
    pub last_pos: Position,
    pub move_type: u32,
    pub move_speed: u32,
    pub route: [Option<Direction>; MAX_ROUTE],
    pub destiny: Position,
}

impl ActionBroadcastData {
    pub fn route_from_bytes(bytes: [u8; MAX_ROUTE]) -> [Option<Direction>; MAX_ROUTE] {
        let mut route = [None; MAX_ROUTE];
        for (i, &b) in bytes.iter().enumerate() {
            if b >= b'1' && b <= b'9' {
                route[i] = Direction::try_from(b - b'0').ok();
            }
        }
        route
    }

    pub fn route_from_directions(dirs: &[Direction]) -> [Option<Direction>; MAX_ROUTE] {
        let mut route = [None; MAX_ROUTE];
        for (i, dir) in dirs.iter().enumerate().take(MAX_ROUTE) {
            route[i] = Some(*dir);
        }
        route
    }

    fn to_raw(self) -> ActionRaw {
        let mut command = [0u8; MAX_ROUTE];
        for (i, dir) in self.route.iter().enumerate() {
            if let Some(d) = dir {
                command[i] = d.to_route_byte();
            }
        }
        ActionRaw {
            last_pos: PositionRaw {
                x: self.last_pos.x,
                y: self.last_pos.y,
            },
            move_type: self.move_type,
            move_speed: self.move_speed,
            command,
            destiny: PositionRaw {
                x: self.destiny.x,
                y: self.destiny.y,
            },
        }
    }
}

pub struct ActionWalkBroadcast(pub ActionBroadcastData);

impl WritableResource for ActionWalkBroadcast {
    const IDENTIFIER: ServerMessage = ServerMessage::Action;
    type Output = ActionRaw;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        Ok(self.0.to_raw())
    }

    fn client_id(&self) -> Option<u16> {
        Some(self.0.mover_id)
    }
}

pub struct ActionIllusionBroadcast(pub ActionBroadcastData);

impl WritableResource for ActionIllusionBroadcast {
    const IDENTIFIER: ServerMessage = ServerMessage::ActionIllusion;
    type Output = ActionRaw;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        Ok(self.0.to_raw())
    }

    fn client_id(&self) -> Option<u16> {
        Some(self.0.mover_id)
    }
}

pub struct ActionStopBroadcast(pub ActionBroadcastData);

impl WritableResource for ActionStopBroadcast {
    const IDENTIFIER: ServerMessage = ServerMessage::ActionStop;
    type Output = ActionRaw;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        Ok(self.0.to_raw())
    }

    fn client_id(&self) -> Option<u16> {
        Some(self.0.mover_id)
    }
}
