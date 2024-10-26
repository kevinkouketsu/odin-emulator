pub mod header;
pub mod string;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageIdentifier {
    Login,
    Token,
}
impl TryFrom<u16> for MessageIdentifier {
    type Error = InvalidMessageType;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0x784 => MessageIdentifier::Login,
            0xFDE => MessageIdentifier::Token,
            _ => return Err(InvalidMessageType(value)),
        })
    }
}

#[derive(Debug, Error)]
#[error("The type {0} has not been identified")]
pub struct InvalidMessageType(u16);
