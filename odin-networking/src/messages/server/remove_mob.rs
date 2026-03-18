use crate::{WritableResource, WritableResourceError, messages::ServerMessage};
use deku::prelude::*;

pub struct RemoveMob {
    pub mob_id: u16,
    // TODO: figure out what the remove type is, and if it can be an enum instead of an i32
    pub remove_type: i32,
}

impl WritableResource for RemoveMob {
    const IDENTIFIER: ServerMessage = ServerMessage::RemoveMob;
    type Output = RemoveMobRaw;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        Ok(RemoveMobRaw {
            remove_type: self.remove_type,
        })
    }

    fn client_id(&self) -> Option<u16> {
        Some(self.mob_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite)]
pub struct RemoveMobRaw {
    pub remove_type: i32,
}
