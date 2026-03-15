use super::MessageSignal;
use crate::{
    WritableResource, WritableResourceError,
    messages::{ServerMessage, client::numeric_token::NumericTokenRaw},
};
use odin_macros::MessageSignalDerive;

pub struct CorrectNumericToken {
    pub token: String,
    pub changing: bool,
}
impl WritableResource for CorrectNumericToken {
    const IDENTIFIER: ServerMessage = ServerMessage::CorrectNumericToken;
    type Output = NumericTokenRaw;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        Ok(NumericTokenRaw {
            token: self.token.try_into()?,
            state: self.changing as u32,
        })
    }
}

#[derive(Default, MessageSignalDerive)]
#[identifier = "ServerMessage::IncorrectNumericToken"]
pub struct IncorrectNumericToken;
