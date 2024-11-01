use super::MessageSignal;
use crate::{
    messages::{client::numeric_token::NumericTokenRaw, ServerMessage},
    WritableResource, WritableResourceError,
};

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

#[derive(Default)]
pub struct IncorrectNumericToken;
impl WritableResource for IncorrectNumericToken {
    const IDENTIFIER: ServerMessage = ServerMessage::IncorrectNumericToken;
    type Output = MessageSignal<IncorrectNumericToken>;

    fn write(self) -> Result<Self::Output, WritableResourceError> {
        Ok(MessageSignal::default())
    }
}
