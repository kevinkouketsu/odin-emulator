use crate::{
    WritableResource, WritableResourceError,
    messages::{ServerMessage, client::action::ActionRaw, common::PositionRaw},
};
use odin_models::position::Position;

#[derive(Clone, Copy)]
pub struct ActionBroadcastData {
    pub mover_id: u16,
    pub last_pos: Position,
    pub move_type: u32,
    pub move_speed: u32,
    pub command: [u8; 24],
    pub destiny: Position,
}

impl ActionBroadcastData {
    fn to_raw(self) -> ActionRaw {
        ActionRaw {
            last_pos: PositionRaw {
                x: self.last_pos.x,
                y: self.last_pos.y,
            },
            move_type: self.move_type,
            move_speed: self.move_speed,
            command: self.command,
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
