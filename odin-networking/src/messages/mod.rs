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
}
impl TryFrom<u16> for ClientMessage {
    type Error = InvalidMessageType;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0x784 => ClientMessage::Login,
            0xFDE => ClientMessage::Token,
            _ => return Err(InvalidMessageType(value)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServerMessage {
    MessagePanel,
    FirstCharlist,
}
impl TryFrom<ServerMessage> for u16 {
    type Error = InvalidMessageType;

    fn try_from(value: ServerMessage) -> Result<Self, Self::Error> {
        Ok(match value {
            ServerMessage::MessagePanel => 0x101,
            ServerMessage::FirstCharlist => 0x10A,
        })
    }
}

#[derive(Debug, Error)]
#[error("The type {0} has not been identified")]
pub struct InvalidMessageType(u16);
