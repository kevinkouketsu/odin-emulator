use crate::handlers::{
    authentication::{Authentication, AuthenticationError},
    create_character::CreateCharacter,
    delete_character::DeleteCharacter,
    numeric_token::NumericToken,
};
use deku::prelude::*;
use odin_macros::HandlerDerive;
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

#[derive(Debug, HandlerDerive)]
pub enum Message {
    #[raw = "LoginMessageRaw"]
    Login(Authentication),
    #[raw = "NumericTokenRaw"]
    Token(NumericToken),
    #[raw = "CreateCharacterRaw"]
    CreateCharacter(CreateCharacter),
    #[raw = "DeleteCharacterRaw"]
    DeleteCharacter(DeleteCharacter),
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
