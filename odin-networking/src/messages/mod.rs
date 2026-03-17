pub mod client;
pub mod common;
pub mod header;
pub mod server;
pub mod string;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageDirection {
    Client(ClientMessage),
    Server(ServerMessage),
    ClientServer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientMessage {
    Login,
    Token,
    CreateCharacter,
    DeleteCharacter,
    EnterWorld,
    ApplyBonus,
}
impl TryFrom<u16> for ClientMessage {
    type Error = InvalidMessageType;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0x784 => ClientMessage::Login,
            0xFDE => ClientMessage::Token,
            0x20F => ClientMessage::CreateCharacter,
            0x211 => ClientMessage::DeleteCharacter,
            0x213 => ClientMessage::EnterWorld,
            0x277 => ClientMessage::ApplyBonus,
            _ => return Err(InvalidMessageType(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServerMessage {
    MessagePanel,
    FirstCharlist,
    CorrectNumericToken,
    IncorrectNumericToken,
    CreatedCharacter,
    DeleteCharacter,
    CharacterNameAlreadyExists,
    CreateMob,
    CharacterLogin,
    UpdateEtc,
}
impl TryFrom<ServerMessage> for u16 {
    type Error = InvalidMessageType;

    fn try_from(value: ServerMessage) -> Result<Self, Self::Error> {
        Ok(match value {
            ServerMessage::MessagePanel => 0x101,
            ServerMessage::FirstCharlist => 0x10A,
            ServerMessage::CorrectNumericToken => 0xFDE,
            ServerMessage::IncorrectNumericToken => 0xFDF,
            ServerMessage::CreatedCharacter => 0x110,
            ServerMessage::DeleteCharacter => 0x112,
            ServerMessage::CharacterNameAlreadyExists => 0x11A,
            ServerMessage::CreateMob => 0x364,
            ServerMessage::CharacterLogin => 0x114,
            ServerMessage::UpdateEtc => 0x337,
        })
    }
}

#[derive(Debug, Error)]
#[error("The type {0} has not been identified")]
pub struct InvalidMessageType(u16);
