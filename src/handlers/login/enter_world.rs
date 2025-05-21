use crate::session::SessionTrait;
use odin_networking::{messages::client::enter_world::EnterWorldRaw, WritableResourceError};

#[derive(Debug)]
pub struct EnterWorld {
    pub slot: u32,
    pub force: bool,
    // We don't support this for now, but it's here for future reference
    pub secret_code: String,
}
impl EnterWorld {
    pub async fn handle<S: SessionTrait>(&self, _session: &S) -> Result<(), WritableResourceError> {
        Ok(())
    }
}
impl TryFrom<EnterWorldRaw> for EnterWorld {
    type Error = WritableResourceError;

    fn try_from(value: EnterWorldRaw) -> Result<Self, Self::Error> {
        Ok(EnterWorld {
            slot: value.slot,
            force: value.force != 0,
            secret_code: value.secret_code.try_into()?,
        })
    }
}
