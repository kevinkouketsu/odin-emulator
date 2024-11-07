use crate::handlers::{
    authentication::{Authentication, AuthenticationError},
    create_character::CreateCharacter,
    delete_character::DeleteCharacter,
    numeric_token::NumericToken,
};
use deku::prelude::*;
use odin_networking::{
    messages::{
        client::{
            create_character::CreateCharacterRaw, delete_character::DeleteCharacterRaw,
            login::LoginMessageRaw, numeric_token::NumericTokenRaw,
        },
        header::Header,
        ClientMessage,
    },
    WritableResourceError,
};
use thiserror::Error;

#[derive(Debug)]
pub enum Message {
    Login(Authentication),
    Token(NumericToken),
    CreateCharacter(CreateCharacter),
    DeleteCharacter(DeleteCharacter),
}
impl TryFrom<((&[u8], usize), Header)> for Message {
    type Error = MessageError;

    fn try_from((rest, header): ((&[u8], usize), Header)) -> Result<Self, Self::Error> {
        let message_type = ClientMessage::try_from(header.typ)
            .map_err(|_| MessageError::NotRecognized(header.clone()))?;

        Ok(match message_type {
            ClientMessage::Login => {
                Message::Login(LoginMessageRaw::from_bytes(rest)?.1.try_into()?)
            }
            ClientMessage::Token => {
                Message::Token(NumericTokenRaw::from_bytes(rest)?.1.try_into()?)
            }
            ClientMessage::CreateCharacter => {
                Message::CreateCharacter(CreateCharacterRaw::from_bytes(rest)?.1.try_into()?)
            }
            ClientMessage::DeleteCharacter => {
                Message::DeleteCharacter(DeleteCharacterRaw::from_bytes(rest)?.1.try_into()?)
            }
        })
    }
}

#[derive(Debug, Error)]
pub enum MessageError {
    #[error("The packet is not implemented yet: {0:?}")]
    NotImplemented(Header),

    #[error("Invalid packet, not recognized: {0:?}")]
    NotRecognized(Header),

    #[error("Invalid packet structure")]
    InvalidStructure(#[from] DekuError),

    #[error("Invalid conversion for rust type")]
    InvalidToRust(#[from] WritableResourceError),

    #[error(transparent)]
    AuthenticationError(#[from] AuthenticationError),
}
