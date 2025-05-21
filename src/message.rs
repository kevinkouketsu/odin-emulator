use crate::handlers::login::{
    authentication::{Authentication, AuthenticationError},
    create_character::CreateCharacter,
    delete_character::DeleteCharacter,
    enter_world::EnterWorld,
    numeric_token::NumericToken,
};
use deku::prelude::*;
use odin_macros::HandlerDerive;
use odin_networking::{
    messages::{
        client::{
            create_character::CreateCharacterRaw, delete_character::DeleteCharacterRaw,
            enter_world::EnterWorldRaw, login::LoginMessageRaw, numeric_token::NumericTokenRaw,
        },
        header::Header,
        ClientMessage,
    },
    WritableResourceError,
};
use thiserror::Error;

/// Represents a client message that can be handled by the server.
/// Each variant corresponds to a specific message type and its associated handler.
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
    #[raw = "EnterWorldRaw"]
    EnterWorld(EnterWorld),
}

#[derive(Debug, Error)]
pub enum MessageError {
    #[error("Message type {0:?} is not yet implemented")]
    NotImplemented(Header),

    #[error("Unknown message type: {0:?}")]
    NotRecognized(Header),

    #[error("Invalid message structure: {0}")]
    InvalidStructure(#[from] DekuError),

    #[error("Failed to convert message to Rust type: {0}")]
    InvalidToRust(#[from] WritableResourceError),

    #[error(transparent)]
    AuthenticationError(#[from] AuthenticationError),
}
